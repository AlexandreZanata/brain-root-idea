//! Deterministic, bounded provider execution.
//!
//! Pull-based: the caller feeds elapsed time with [`RequestOwner::advance`] and
//! there is no thread, timer, async runtime, or wall-clock read. One owner
//! bounds one request with connect/read/total timeouts and a response-size
//! limit, emits exactly one terminal event, and releases its script afterwards.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;

use super::contract::{self, CancellationReason};
use super::fake::FakeWireStep;

pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutKind {
    Connect,
    Read,
    Total,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalReason {
    Completed,
    Cancelled(CancellationReason),
    TimedOut(TimeoutKind),
    ResponseTooLarge { limit: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionEvent {
    Frame(String),
    Terminal(TerminalReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidLimits { field: &'static str },
}

impl ExecutionError {
    pub fn message(&self) -> String {
        match self {
            ExecutionError::InvalidLimits { field } => {
                format!("The {field} limit must be greater than zero.")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub connect: Duration,
    pub read: Duration,
    pub total: Duration,
    pub max_response_bytes: usize,
}

impl ExecutionLimits {
    pub fn new(
        connect: Duration,
        read: Duration,
        total: Duration,
        max_response_bytes: usize,
    ) -> Result<Self, ExecutionError> {
        if connect.is_zero() {
            return Err(ExecutionError::InvalidLimits { field: "connect" });
        }
        if read.is_zero() {
            return Err(ExecutionError::InvalidLimits { field: "read" });
        }
        if total.is_zero() {
            return Err(ExecutionError::InvalidLimits { field: "total" });
        }
        if max_response_bytes == 0 {
            return Err(ExecutionError::InvalidLimits {
                field: "max_response_bytes",
            });
        }
        Ok(Self {
            connect,
            read,
            total,
            max_response_bytes,
        })
    }
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            connect: DEFAULT_CONNECT_TIMEOUT,
            read: DEFAULT_READ_TIMEOUT,
            total: DEFAULT_TOTAL_TIMEOUT,
            max_response_bytes: contract::limits::MAX_RESPONSE_BYTES,
        }
    }
}

/// Shared cancellation handle. Single-threaded callers only need `cancel`,
/// `reason`, and `is_cancelled`; the token is thread-safe on purpose so a
/// later UI thread can cancel without owning the request.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    reason: Arc<Mutex<Option<CancellationReason>>>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self, reason: CancellationReason) {
        let mut slot = self
            .reason
            .lock()
            .expect("cancellation token lock is not poisoned");
        if slot.is_none() {
            *slot = Some(reason);
        }
    }

    pub fn reason(&self) -> Option<CancellationReason> {
        *self
            .reason
            .lock()
            .expect("cancellation token lock is not poisoned")
    }

    pub fn is_cancelled(&self) -> bool {
        self.reason().is_some()
    }
}

#[derive(Debug)]
pub struct RequestOwner {
    script: VecDeque<FakeWireStep>,
    limits: ExecutionLimits,
    token: CancellationToken,
    credit: Duration,
    total: Duration,
    since_last_frame: Duration,
    received_frame: bool,
    bytes: usize,
    terminal: Option<TerminalReason>,
}

impl RequestOwner {
    pub fn start(
        script: Vec<FakeWireStep>,
        limits: ExecutionLimits,
        token: CancellationToken,
    ) -> Self {
        Self {
            script: VecDeque::from(script),
            limits,
            token,
            credit: Duration::ZERO,
            total: Duration::ZERO,
            since_last_frame: Duration::ZERO,
            received_frame: false,
            bytes: 0,
            terminal: None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal.is_some()
    }

    pub fn pending_steps(&self) -> usize {
        self.script.len()
    }

    /// Cancels immediately and returns the single terminal event, or `None`
    /// when the request already terminated.
    pub fn cancel(&mut self, reason: CancellationReason) -> Option<ExecutionEvent> {
        self.token.cancel(reason);
        self.finish(TerminalReason::Cancelled(reason))
    }

    /// Feeds elapsed time and returns the events produced by this step.
    pub fn advance(&mut self, elapsed: Duration) -> Vec<ExecutionEvent> {
        if self.terminal.is_some() {
            return Vec::new();
        }
        if let Some(reason) = self.token.reason() {
            return self
                .finish(TerminalReason::Cancelled(reason))
                .into_iter()
                .collect();
        }

        self.total += elapsed;
        self.since_last_frame += elapsed;
        self.credit += elapsed;

        let mut events = Vec::new();
        loop {
            if self.terminal.is_some() {
                break;
            }
            match self.script.front() {
                Some(FakeWireStep::Frame(_)) => {
                    let Some(FakeWireStep::Frame(payload)) = self.script.pop_front() else {
                        break;
                    };
                    self.since_last_frame = Duration::ZERO;
                    self.received_frame = true;
                    self.bytes = self.bytes.saturating_add(payload.len());
                    let is_end = frame_kind(&payload).as_deref() == Some("end");
                    events.push(ExecutionEvent::Frame(payload));
                    if self.bytes > self.limits.max_response_bytes {
                        if let Some(event) = self.finish(TerminalReason::ResponseTooLarge {
                            limit: self.limits.max_response_bytes,
                        }) {
                            events.push(event);
                        }
                    } else if is_end {
                        if let Some(event) = self.finish(TerminalReason::Completed) {
                            events.push(event);
                        }
                    }
                }
                Some(FakeWireStep::Delay(delay)) => {
                    let delay = *delay;
                    if self.credit >= delay {
                        self.credit -= delay;
                        self.script.pop_front();
                    } else {
                        break;
                    }
                }
                Some(FakeWireStep::Hang) | None => break,
            }
        }

        if self.terminal.is_none() {
            if !self.received_frame {
                if self.total >= self.limits.connect {
                    if let Some(event) = self.finish(TerminalReason::TimedOut(TimeoutKind::Connect))
                    {
                        events.push(event);
                    }
                }
            } else if self.total >= self.limits.total {
                if let Some(event) = self.finish(TerminalReason::TimedOut(TimeoutKind::Total)) {
                    events.push(event);
                }
            } else if self.since_last_frame >= self.limits.read {
                if let Some(event) = self.finish(TerminalReason::TimedOut(TimeoutKind::Read)) {
                    events.push(event);
                }
            } else if self.script.is_empty() {
                if let Some(event) = self.finish(TerminalReason::Completed) {
                    events.push(event);
                }
            }
        }

        events
    }

    fn finish(&mut self, reason: TerminalReason) -> Option<ExecutionEvent> {
        if self.terminal.is_some() {
            return None;
        }
        self.terminal = Some(reason.clone());
        self.script.clear();
        self.credit = Duration::ZERO;
        Some(ExecutionEvent::Terminal(reason))
    }
}

fn frame_kind(payload: &str) -> Option<String> {
    serde_json::from_str::<Value>(payload)
        .ok()
        .and_then(|value| {
            value
                .get("kind")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::fake::{FakeProvider, FakeScenario};
    use std::time::Duration;

    fn limits(connect: u64, read: u64, total: u64) -> ExecutionLimits {
        ExecutionLimits::new(
            Duration::from_secs(connect),
            Duration::from_secs(read),
            Duration::from_secs(total),
            contract::limits::MAX_RESPONSE_BYTES,
        )
        .expect("valid limits")
    }

    fn owner(scenario: FakeScenario) -> RequestOwner {
        RequestOwner::start(
            FakeProvider.script(scenario),
            limits(5, 10, 120),
            CancellationToken::new(),
        )
    }

    fn frames(events: &[ExecutionEvent]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match event {
                ExecutionEvent::Frame(payload) => Some(payload.clone()),
                ExecutionEvent::Terminal(_) => None,
            })
            .collect()
    }

    fn terminal(events: &[ExecutionEvent]) -> Option<TerminalReason> {
        events.iter().find_map(|event| match event {
            ExecutionEvent::Terminal(reason) => Some(reason.clone()),
            ExecutionEvent::Frame(_) => None,
        })
    }

    #[test]
    fn success_runs_to_completion() {
        let mut owner = owner(FakeScenario::Success);
        let events = owner.advance(Duration::ZERO);
        let terminal_events: Vec<&ExecutionEvent> = events
            .iter()
            .filter(|event| matches!(event, ExecutionEvent::Terminal(_)))
            .collect();

        assert_eq!(terminal_events.len(), 1);
        assert_eq!(terminal(&events), Some(TerminalReason::Completed));
        assert_eq!(frames(&events).len(), 5);
        assert!(owner.is_terminal());
        assert_eq!(owner.pending_steps(), 0);
    }

    #[test]
    fn connect_timeout_fires_before_the_first_frame() {
        let mut owner = RequestOwner::start(
            vec![
                FakeWireStep::Delay(Duration::from_secs(20)),
                FakeWireStep::Frame(r#"{"kind":"chunk","text":"late"}"#.to_string()),
            ],
            limits(5, 30, 120),
            CancellationToken::new(),
        );

        let events = owner.advance(Duration::from_secs(6));
        assert_eq!(
            terminal(&events),
            Some(TerminalReason::TimedOut(TimeoutKind::Connect))
        );
        assert!(frames(&events).is_empty());
        assert_eq!(owner.pending_steps(), 0);
    }

    #[test]
    fn read_timeout_fires_between_events() {
        let mut owner = RequestOwner::start(
            vec![
                FakeWireStep::Frame(r#"{"kind":"start"}"#.to_string()),
                FakeWireStep::Delay(Duration::from_secs(20)),
                FakeWireStep::Frame(r#"{"kind":"end"}"#.to_string()),
            ],
            limits(5, 10, 120),
            CancellationToken::new(),
        );

        let first = owner.advance(Duration::from_secs(1));
        assert_eq!(frames(&first).len(), 1);
        assert_eq!(terminal(&first), None);

        let second = owner.advance(Duration::from_secs(11));
        assert_eq!(
            terminal(&second),
            Some(TerminalReason::TimedOut(TimeoutKind::Read))
        );
    }

    #[test]
    fn total_timeout_bounds_a_never_ending_script() {
        let mut owner = owner(FakeScenario::NeverEnding);
        let mut terminal_reason = None;
        for _ in 0..3 {
            let events = owner.advance(Duration::from_secs(60));
            if let Some(reason) = terminal(&events) {
                terminal_reason = Some(reason);
                break;
            }
        }

        assert_eq!(
            terminal_reason,
            Some(TerminalReason::TimedOut(TimeoutKind::Total))
        );
        assert!(owner.is_terminal());
    }

    #[test]
    fn cancellation_emits_exactly_one_terminal() {
        let mut owner = owner(FakeScenario::NeverEnding);
        let _ = owner.advance(Duration::from_secs(1));
        assert!(!owner.is_terminal());

        let event = owner.cancel(CancellationReason::UserRequested);
        assert_eq!(
            event,
            Some(ExecutionEvent::Terminal(TerminalReason::Cancelled(
                CancellationReason::UserRequested
            )))
        );
        assert!(owner.advance(Duration::from_secs(1)).is_empty());
        assert_eq!(owner.pending_steps(), 0);
    }

    #[test]
    fn token_cancellation_is_detected_on_advance() {
        let token = CancellationToken::new();
        let mut owner = RequestOwner::start(
            FakeProvider.script(FakeScenario::Success),
            limits(5, 10, 120),
            token.clone(),
        );

        token.cancel(CancellationReason::Timeout);
        let events = owner.advance(Duration::ZERO);

        assert_eq!(
            terminal(&events),
            Some(TerminalReason::Cancelled(CancellationReason::Timeout))
        );
        assert!(owner.advance(Duration::ZERO).is_empty());
    }

    #[test]
    fn late_cancel_after_terminal_is_a_noop() {
        let mut owner = owner(FakeScenario::Success);
        let _ = owner.advance(Duration::ZERO);
        assert!(owner.is_terminal());

        assert_eq!(owner.cancel(CancellationReason::UserRequested), None);
        assert!(owner.advance(Duration::from_secs(1)).is_empty());
    }

    #[test]
    fn response_size_limit_terminates() {
        let mut owner = RequestOwner::start(
            FakeProvider.script(FakeScenario::Success),
            ExecutionLimits::new(
                Duration::from_secs(5),
                Duration::from_secs(10),
                Duration::from_secs(120),
                20,
            )
            .expect("valid limits"),
            CancellationToken::new(),
        );

        let events = owner.advance(Duration::ZERO);
        assert_eq!(
            terminal(&events),
            Some(TerminalReason::ResponseTooLarge { limit: 20 })
        );
    }

    #[test]
    fn zero_limits_are_rejected() {
        let error = ExecutionLimits::new(
            Duration::ZERO,
            Duration::from_secs(10),
            Duration::from_secs(120),
            1024,
        )
        .expect_err("zero connect limit must fail");
        assert_eq!(error, ExecutionError::InvalidLimits { field: "connect" });

        let error = ExecutionLimits::new(
            Duration::from_secs(5),
            Duration::from_secs(10),
            Duration::from_secs(120),
            0,
        )
        .expect_err("zero response limit must fail");
        assert_eq!(
            error,
            ExecutionError::InvalidLimits {
                field: "max_response_bytes"
            }
        );
    }

    #[test]
    fn script_exhaustion_completes() {
        let mut owner = RequestOwner::start(
            vec![FakeWireStep::Frame(r#"{"kind":"start"}"#.to_string())],
            limits(5, 10, 120),
            CancellationToken::new(),
        );

        let events = owner.advance(Duration::ZERO);
        assert_eq!(terminal(&events), Some(TerminalReason::Completed));
    }

    #[test]
    fn delayed_stream_needs_enough_accumulated_time() {
        let mut owner = RequestOwner::start(
            vec![
                FakeWireStep::Frame(r#"{"kind":"start"}"#.to_string()),
                FakeWireStep::Delay(Duration::from_secs(5)),
                FakeWireStep::Frame(r#"{"kind":"chunk","text":"ok"}"#.to_string()),
                FakeWireStep::Frame(r#"{"kind":"end"}"#.to_string()),
            ],
            limits(5, 30, 120),
            CancellationToken::new(),
        );

        let first = owner.advance(Duration::from_secs(2));
        assert_eq!(frames(&first).len(), 1);
        assert_eq!(terminal(&first), None);

        let second = owner.advance(Duration::from_secs(3));
        assert_eq!(frames(&second).len(), 2);
        assert_eq!(terminal(&second), Some(TerminalReason::Completed));
    }

    #[test]
    fn finished_owner_releases_the_script_and_ignores_time() {
        let mut owner = owner(FakeScenario::Success);
        let _ = owner.advance(Duration::ZERO);

        assert_eq!(owner.pending_steps(), 0);
        assert!(owner.advance(Duration::from_secs(3600)).is_empty());
        assert!(owner.is_terminal());
    }
}
