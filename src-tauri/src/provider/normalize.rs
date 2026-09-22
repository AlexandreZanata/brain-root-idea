//! Reassemble wire bytes into neutral stream events.
//!
//! Frames are self-delimiting JSON values: the decoder buffers raw bytes and
//! extracts complete values with `serde_json`'s streaming deserializer, so
//! adjacent frames and chunks split inside a multi-byte UTF-8 character work.
//! Provider-supplied text is never surfaced; only BrainRoot's plain-language
//! messages cross the boundary.

use std::collections::VecDeque;

use serde_json::Value;

use super::contract::{
    limits, Cancellation, Completion, ConversationId, ErrorCode, NormalizedError, StopReason,
    StreamEvent,
};
use super::execution::{ExecutionEvent, TerminalReason, TimeoutKind};

pub const MAX_PENDING_EVENTS: usize = 64;
pub const MAX_WIRE_BUFFER_BYTES: usize = limits::MAX_STREAM_EVENT_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizeError {
    BufferTooLarge { limit: usize },
    QueueFull { limit: usize },
    AlreadyTerminal,
}

impl NormalizeError {
    pub fn message(&self) -> String {
        match self {
            NormalizeError::BufferTooLarge { limit } => {
                format!("The provider sent a frame larger than the {limit}-byte limit.")
            }
            NormalizeError::QueueFull { limit } => {
                format!("The pending event queue is full ({limit} events); drain it before feeding more.")
            }
            NormalizeError::AlreadyTerminal => {
                "The request already reached a terminal event.".to_string()
            }
        }
    }
}

#[derive(Debug)]
pub struct StreamNormalizer {
    conversation: ConversationId,
    buffer: Vec<u8>,
    pending: VecDeque<StreamEvent>,
    finished: bool,
}

impl StreamNormalizer {
    pub fn new(conversation: ConversationId) -> Self {
        Self {
            conversation,
            buffer: Vec::new(),
            pending: VecDeque::new(),
            finished: false,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Feeds raw wire bytes. On `QueueFull` nothing is consumed, so the caller
    /// can drain and retry the same chunk.
    pub fn push(&mut self, chunk: &[u8]) -> Result<(), NormalizeError> {
        if self.finished {
            return Err(NormalizeError::AlreadyTerminal);
        }
        let original_len = self.buffer.len();
        if original_len + chunk.len() > MAX_WIRE_BUFFER_BYTES {
            return Err(NormalizeError::BufferTooLarge {
                limit: MAX_WIRE_BUFFER_BYTES,
            });
        }
        self.buffer.extend_from_slice(chunk);

        let mut mapped = Vec::new();
        let mut consumed = 0usize;
        let mut malformed = false;
        {
            let mut stream =
                serde_json::Deserializer::from_slice(&self.buffer).into_iter::<Value>();
            loop {
                match stream.next() {
                    Some(Ok(value)) => {
                        consumed = stream.byte_offset();
                        mapped.push(map_frame(&value, &self.conversation));
                    }
                    Some(Err(error)) if error.is_eof() => break,
                    Some(Err(_)) => {
                        malformed = true;
                        break;
                    }
                    None => break,
                }
            }
        }

        if malformed {
            mapped.push(failed(
                ErrorCode::MalformedResponse,
                "The provider returned an unreadable response.",
            ));
        }

        if self.pending.len() + mapped.len() > MAX_PENDING_EVENTS {
            self.buffer.truncate(original_len);
            return Err(NormalizeError::QueueFull {
                limit: MAX_PENDING_EVENTS,
            });
        }

        self.buffer.drain(..consumed);
        if malformed {
            self.buffer.clear();
        }
        for event in mapped {
            self.enqueue(event);
        }
        Ok(())
    }

    /// Feeds an executor event: frames go through the byte decoder and terminal
    /// reasons become the single terminal stream event.
    pub fn push_execution(&mut self, event: ExecutionEvent) -> Result<(), NormalizeError> {
        match event {
            ExecutionEvent::Frame(payload) => self.push(payload.as_bytes()),
            ExecutionEvent::Terminal(reason) => {
                if self.finished {
                    return Err(NormalizeError::AlreadyTerminal);
                }
                if self.pending.len() + 1 > MAX_PENDING_EVENTS {
                    return Err(NormalizeError::QueueFull {
                        limit: MAX_PENDING_EVENTS,
                    });
                }
                self.enqueue(terminal_event(reason, &self.conversation));
                Ok(())
            }
        }
    }

    pub fn drain(&mut self) -> Vec<StreamEvent> {
        self.pending.drain(..).collect()
    }

    fn enqueue(&mut self, event: StreamEvent) {
        if is_terminal_event(&event) {
            self.finished = true;
        }
        self.pending.push_back(event);
    }
}

fn is_terminal_event(event: &StreamEvent) -> bool {
    matches!(
        event,
        StreamEvent::Completed { .. } | StreamEvent::Cancelled { .. } | StreamEvent::Failed { .. }
    )
}

fn map_frame(value: &Value, conversation: &ConversationId) -> StreamEvent {
    let Some(object) = value.as_object() else {
        return failed(
            ErrorCode::MalformedResponse,
            "The provider returned an unreadable response.",
        );
    };

    match object.get("kind").and_then(Value::as_str) {
        Some("start") => StreamEvent::Started,
        Some("chunk") => match object.get("text").and_then(Value::as_str) {
            Some(text) => StreamEvent::TextChunk {
                text: text.to_string(),
            },
            None => failed(
                ErrorCode::MalformedResponse,
                "The provider returned an unreadable response.",
            ),
        },
        Some("end") => StreamEvent::Completed {
            completion: Completion {
                conversation: conversation.clone(),
                stop_reason: StopReason::EndTurn,
            },
        },
        Some("error") => {
            let code = object
                .get("code")
                .and_then(Value::as_str)
                .map(error_code_from_wire)
                .unwrap_or(ErrorCode::MalformedResponse);
            failed(code, &error_message(code))
        }
        Some(_) => failed(
            ErrorCode::UnknownEvent,
            "The provider sent an event BrainRoot does not understand.",
        ),
        None => failed(
            ErrorCode::MalformedResponse,
            "The provider returned an unreadable response.",
        ),
    }
}

fn terminal_event(reason: TerminalReason, conversation: &ConversationId) -> StreamEvent {
    match reason {
        TerminalReason::Completed => StreamEvent::Completed {
            completion: Completion {
                conversation: conversation.clone(),
                stop_reason: StopReason::EndTurn,
            },
        },
        TerminalReason::Cancelled(reason) => StreamEvent::Cancelled {
            cancellation: Cancellation {
                conversation: conversation.clone(),
                reason,
            },
        },
        TerminalReason::TimedOut(kind) => failed(ErrorCode::TimedOut, timeout_message(kind)),
        TerminalReason::ResponseTooLarge { limit } => failed(
            ErrorCode::ResponseTooLarge,
            &format!("The provider's response was larger than BrainRoot allows ({limit} bytes)."),
        ),
    }
}

fn failed(code: ErrorCode, message: &str) -> StreamEvent {
    StreamEvent::Failed {
        error: NormalizedError {
            code,
            message: message.to_string(),
        },
    }
}

fn error_code_from_wire(code: &str) -> ErrorCode {
    match code {
        "authentication_failed" => ErrorCode::AuthenticationFailed,
        "rate_limited" => ErrorCode::RateLimited,
        "provider_unavailable" => ErrorCode::ProviderUnavailable,
        _ => ErrorCode::MalformedResponse,
    }
}

fn error_message(code: ErrorCode) -> String {
    match code {
        ErrorCode::AuthenticationFailed => "The provider rejected the credential.".to_string(),
        ErrorCode::RateLimited => {
            "The provider temporarily limited requests. Try again shortly.".to_string()
        }
        ErrorCode::ProviderUnavailable => "The provider is temporarily unavailable.".to_string(),
        ErrorCode::MalformedResponse => "The provider returned an unreadable response.".to_string(),
        ErrorCode::UnknownEvent => {
            "The provider sent an event BrainRoot does not understand.".to_string()
        }
        ErrorCode::TimedOut => "The provider took too long to respond.".to_string(),
        ErrorCode::ResponseTooLarge => {
            "The provider's response was larger than BrainRoot allows.".to_string()
        }
        ErrorCode::InvalidInput
        | ErrorCode::RequestTooLarge
        | ErrorCode::UnsupportedContractVersion
        | ErrorCode::InvalidState => "The request could not be completed.".to_string(),
    }
}

fn timeout_message(kind: TimeoutKind) -> &'static str {
    match kind {
        TimeoutKind::Connect => "The provider did not answer in time.",
        TimeoutKind::Read => "The provider stopped sending the answer.",
        TimeoutKind::Total => "The answer took longer than BrainRoot allows.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::contract::CancellationReason;
    use crate::provider::execution::{ExecutionEvent, TerminalReason, TimeoutKind};
    use crate::provider::fake::{FakeProvider, FakeScenario, FakeWireStep};

    fn normalizer() -> StreamNormalizer {
        StreamNormalizer::new(ConversationId::new("conversation-1").expect("valid id"))
    }

    #[test]
    fn success_wire_frames_map_to_neutral_events() {
        let mut normalizer = normalizer();
        for step in FakeProvider.script(FakeScenario::Success) {
            if let FakeWireStep::Frame(payload) = step {
                normalizer.push(payload.as_bytes()).expect("frame accepted");
            }
        }

        let events = normalizer.drain();
        assert_eq!(events.len(), 5);
        assert!(matches!(&events[0], StreamEvent::Started));
        assert!(matches!(&events[1], StreamEvent::TextChunk { text } if text == "Hello"));
        assert!(matches!(&events[4], StreamEvent::Completed { .. }));
        assert!(normalizer.is_finished());
    }

    #[test]
    fn frame_split_inside_a_multibyte_character_is_reassembled() {
        let mut normalizer = normalizer();
        let frame = r#"{"kind":"chunk","text":"café"}"#;
        for byte in frame.as_bytes() {
            normalizer
                .push(std::slice::from_ref(byte))
                .expect("byte accepted");
        }

        let events = normalizer.drain();
        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], StreamEvent::TextChunk { text } if text == "café"));
    }

    #[test]
    fn adjacent_frames_without_a_separator_are_parsed() {
        let mut normalizer = normalizer();
        normalizer
            .push(br#"{"kind":"start"}{"kind":"chunk","text":"hi"}"#)
            .expect("frames accepted");

        let events = normalizer.drain();
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[1], StreamEvent::TextChunk { text } if text == "hi"));
    }

    #[test]
    fn malformed_json_terminates_with_malformed_response() {
        let mut normalizer = normalizer();
        normalizer
            .push(br#"{"kind":"chunk",}"#)
            .expect("input accepted");

        let events = normalizer.drain();
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], StreamEvent::Failed { error } if error.code == ErrorCode::MalformedResponse)
        );
        assert!(normalizer.is_finished());
    }

    #[test]
    fn unknown_kind_terminates_with_unknown_event() {
        let mut normalizer = normalizer();
        normalizer
            .push(br#"{"kind":"telemetry"}"#)
            .expect("input accepted");

        let events = normalizer.drain();
        assert!(
            matches!(&events[0], StreamEvent::Failed { error } if error.code == ErrorCode::UnknownEvent)
        );
    }

    #[test]
    fn error_frames_use_plain_language_and_hide_provider_text() {
        let mut normalizer = normalizer();
        normalizer
            .push(br#"{"kind":"error","code":"authentication_failed","message":"sk-secret rejected"}"#)
            .expect("input accepted");

        let events = normalizer.drain();
        let StreamEvent::Failed { error } = &events[0] else {
            panic!("expected a failed event");
        };
        assert_eq!(error.code, ErrorCode::AuthenticationFailed);
        assert!(error.message.contains("credential"));
        assert!(!error.message.contains("sk-secret"));
    }

    #[test]
    fn duplicate_terminal_is_rejected() {
        let mut normalizer = normalizer();
        normalizer
            .push_execution(ExecutionEvent::Terminal(TerminalReason::Completed))
            .expect("first terminal");

        assert_eq!(
            normalizer.push_execution(ExecutionEvent::Terminal(TerminalReason::Completed)),
            Err(NormalizeError::AlreadyTerminal)
        );
        assert_eq!(normalizer.drain().len(), 1);
    }

    #[test]
    fn cancellation_race_keeps_the_first_terminal() {
        let mut normalizer = normalizer();
        normalizer.push(br#"{"kind":"start"}"#).expect("accepted");
        normalizer
            .push_execution(ExecutionEvent::Terminal(TerminalReason::Cancelled(
                CancellationReason::UserRequested,
            )))
            .expect("cancel accepted");

        assert_eq!(
            normalizer.push_execution(ExecutionEvent::Terminal(TerminalReason::Completed)),
            Err(NormalizeError::AlreadyTerminal)
        );
        let events = normalizer.drain();
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[1], StreamEvent::Cancelled { .. }));
    }

    #[test]
    fn backpressure_rejects_when_the_queue_is_full() {
        let mut normalizer = normalizer();
        for index in 0..MAX_PENDING_EVENTS {
            let frame = format!(r#"{{"kind":"chunk","text":"chunk-{index}"}}"#);
            normalizer.push(frame.as_bytes()).expect("within bound");
        }

        let overflow = br#"{"kind":"chunk","text":"overflow"}"#;
        assert_eq!(
            normalizer.push(overflow),
            Err(NormalizeError::QueueFull {
                limit: MAX_PENDING_EVENTS
            })
        );
        assert_eq!(normalizer.drain().len(), MAX_PENDING_EVENTS);

        normalizer.push(overflow).expect("accepted after draining");
        assert_eq!(normalizer.drain().len(), 1);
    }

    #[test]
    fn oversized_partial_frame_is_rejected() {
        let mut normalizer = normalizer();
        let oversized = vec![b'a'; MAX_WIRE_BUFFER_BYTES + 1];

        assert_eq!(
            normalizer.push(&oversized),
            Err(NormalizeError::BufferTooLarge {
                limit: MAX_WIRE_BUFFER_BYTES
            })
        );
    }

    #[test]
    fn timeout_terminals_map_to_the_timed_out_code() {
        let mut normalizer = normalizer();
        normalizer
            .push_execution(ExecutionEvent::Terminal(TerminalReason::TimedOut(
                TimeoutKind::Read,
            )))
            .expect("terminal");

        let events = normalizer.drain();
        assert!(
            matches!(&events[0], StreamEvent::Failed { error } if error.code == ErrorCode::TimedOut && error.message.contains("stopped sending"))
        );
    }

    #[test]
    fn response_too_large_maps_to_its_code() {
        let mut normalizer = normalizer();
        normalizer
            .push_execution(ExecutionEvent::Terminal(TerminalReason::ResponseTooLarge {
                limit: 20,
            }))
            .expect("terminal");

        let events = normalizer.drain();
        assert!(
            matches!(&events[0], StreamEvent::Failed { error } if error.code == ErrorCode::ResponseTooLarge && error.message.contains("20"))
        );
    }

    #[test]
    fn wire_end_frame_terminates_the_stream() {
        let mut normalizer = normalizer();
        normalizer
            .push(br#"{"kind":"start"}{"kind":"end"}"#)
            .expect("accepted");

        let events = normalizer.drain();
        assert!(matches!(&events[1], StreamEvent::Completed { .. }));
        assert_eq!(
            normalizer.push(br#"{"kind":"chunk","text":"late"}"#),
            Err(NormalizeError::AlreadyTerminal)
        );
        assert_eq!(normalizer.pending_len(), 0);
    }

    #[test]
    fn no_events_after_a_terminal() {
        let mut normalizer = normalizer();
        normalizer
            .push_execution(ExecutionEvent::Terminal(TerminalReason::Cancelled(
                CancellationReason::UserRequested,
            )))
            .expect("terminal");
        let _ = normalizer.drain();

        assert!(normalizer.is_finished());
        assert_eq!(normalizer.pending_len(), 0);
        assert_eq!(
            normalizer.push(br#"{"kind":"start"}"#),
            Err(NormalizeError::AlreadyTerminal)
        );
    }
}
