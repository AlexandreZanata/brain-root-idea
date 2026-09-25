//! Resource Governor (S01): one thread enforcing the sidecar idle budget.
//!
//! The 300–400MB sidecar must never sit idle-resident. A single background
//! thread wakes every `TICK`, asks the Agent Host to stop a sidecar idle
//! longer than `SIDECAR_IDLE_LIMIT`, and exits on `shutdown`. No polling per
//! resource, no network probe in the tick path (`stop_if_idle` reads only
//! timestamps). Preview/human-browser registration arrives in S02.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::agent_host::AgentHostState;

pub const TICK: Duration = Duration::from_secs(5);
pub const SIDECAR_IDLE_LIMIT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GovernorStatus {
    pub tick_secs: u64,
    pub sidecar_idle_secs: u64,
    pub sidecar_running: bool,
    pub auto_stops: u64,
}

#[derive(Debug, Default)]
struct Counters {
    auto_stops: u64,
}

#[derive(Debug)]
pub struct GovernorState {
    running: Arc<AtomicBool>,
    counters: std::sync::Mutex<Counters>,
}

impl Default for GovernorState {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            counters: std::sync::Mutex::default(),
        }
    }
}

impl GovernorState {
    /// Start the single enforcement thread. Idempotent; extra calls are no-ops.
    pub fn start(&self, app: AppHandle) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let running = Arc::clone(&self.running);
        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(TICK);
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let stopped = app
                    .state::<AgentHostState>()
                    .stop_if_idle(SIDECAR_IDLE_LIMIT);
                if stopped {
                    // Best effort; a poisoned counter must not kill enforcement.
                    // auto_stops is Mutex-guarded in status(); increment inline.
                    let _ = app.state::<GovernorState>().note_auto_stop();
                }
            }
        });
    }

    fn note_auto_stop(&self) {
        if let Ok(mut counters) = self.counters.lock() {
            counters.auto_stops = counters.auto_stops.saturating_add(1);
        }
    }

    /// Signal the thread to exit on its next tick (≤5s overrun).
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn status(&self, app: &AppHandle) -> GovernorStatus {
        let auto_stops = self
            .counters
            .lock()
            .map(|counters| counters.auto_stops)
            .unwrap_or(0);
        GovernorStatus {
            tick_secs: TICK.as_secs(),
            sidecar_idle_secs: SIDECAR_IDLE_LIMIT.as_secs(),
            sidecar_running: app.state::<AgentHostState>().has_sidecar(),
            auto_stops,
        }
    }
}

#[tauri::command]
pub fn governor_status(app: AppHandle, state: tauri::State<'_, GovernorState>) -> GovernorStatus {
    state.status(&app)
}

#[cfg(test)]
mod deferred {
    use super::*;

    #[test]
    fn budgets_are_sane() {
        assert!(SIDECAR_IDLE_LIMIT >= Duration::from_secs(30));
        assert!(TICK < SIDECAR_IDLE_LIMIT);
        let state = GovernorState::default();
        assert_eq!(state.counters.lock().unwrap().auto_stops, 0);
    }

    // T-R5-01 integration (deferred, opt-in): start sidecar, wait past the
    // idle limit with a shortened test-only tick, assert auto-stop + counter.
    // Too slow for any gate; runs only as a manual release probe.
}
