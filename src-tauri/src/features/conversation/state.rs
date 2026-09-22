//! Explicit conversation state for the MVP-0 Linux model loop.
//!
//! One pure state machine turns user intents and the frozen neutral
//! [`StreamEvent`]s into the seven product states the B04 UI renders:
//! `EMPTY`, `READY`, `SENDING`, `STREAMING`, `CANCELLING`, `SUCCEEDED`, and
//! `FAILED`. The reducer stores no prompt or response text: the caller owns
//! history, bounds it, and forwards events only while a transition is legal.
//! A cancelled request returns to `READY`; completion after cancel is rejected.

use crate::provider::contract::StreamEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    Empty,
    Ready,
    Sending,
    Streaming,
    Cancelling,
    Succeeded,
    Failed,
}

impl ConversationState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Ready => "ready",
            Self::Sending => "sending",
            Self::Streaming => "streaming",
            Self::Cancelling => "cancelling",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    /// Applies one event, returning the next state or a typed rejection.
    pub fn next(self, event: &ConversationEvent) -> Result<Self, TransitionError> {
        use ConversationEvent as Event;
        use ConversationState as State;

        match (self, event) {
            (State::Empty | State::Ready | State::Succeeded | State::Failed, Event::Submit) => {
                Ok(State::Sending)
            }
            (State::Sending, Event::Stream(StreamEvent::Started)) => Ok(State::Streaming),
            (State::Sending | State::Streaming, Event::Stream(StreamEvent::TextChunk { .. })) => {
                Ok(State::Streaming)
            }
            (State::Sending | State::Streaming, Event::Stream(StreamEvent::Completed { .. })) => {
                Ok(State::Succeeded)
            }
            (State::Sending | State::Streaming, Event::Stream(StreamEvent::Failed { .. })) => {
                Ok(State::Failed)
            }
            (State::Sending | State::Streaming, Event::Stream(StreamEvent::Cancelled { .. })) => {
                Ok(State::Ready)
            }
            (State::Sending | State::Streaming, Event::Cancel) => Ok(State::Cancelling),
            (State::Cancelling, Event::Stream(StreamEvent::Cancelled { .. })) => Ok(State::Ready),
            _ => Err(TransitionError {
                state: self,
                event: event.kind(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationEvent {
    /// The user submitted a new prompt; the caller validates and bounds it.
    Submit,
    /// The user asked to cancel the active request.
    Cancel,
    /// One neutral provider event from the frozen contract.
    Stream(StreamEvent),
}

impl ConversationEvent {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Cancel => "cancel",
            Self::Stream(event) => event.kind(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionError {
    pub state: ConversationState,
    pub event: &'static str,
}

impl TransitionError {
    pub fn message(&self) -> String {
        format!(
            "The conversation cannot handle a {} event while it is {}.",
            self.event,
            self.state.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::contract::{
        Cancellation, CancellationReason, Completion, ConversationId, ErrorCode, NormalizedError,
        StopReason,
    };

    fn conversation() -> ConversationId {
        ConversationId::new("conversation-1").expect("valid conversation")
    }

    fn completed() -> ConversationEvent {
        ConversationEvent::Stream(StreamEvent::Completed {
            completion: Completion {
                conversation: conversation(),
                stop_reason: StopReason::EndTurn,
            },
        })
    }

    fn cancelled() -> ConversationEvent {
        ConversationEvent::Stream(StreamEvent::Cancelled {
            cancellation: Cancellation {
                conversation: conversation(),
                reason: CancellationReason::UserRequested,
            },
        })
    }

    fn failed() -> ConversationEvent {
        ConversationEvent::Stream(StreamEvent::Failed {
            error: NormalizedError {
                code: ErrorCode::ProviderUnavailable,
                message: "The provider is temporarily unavailable.".to_string(),
            },
        })
    }

    fn chunk() -> ConversationEvent {
        ConversationEvent::Stream(StreamEvent::TextChunk {
            text: "hidden payload".to_string(),
        })
    }

    fn started() -> ConversationEvent {
        ConversationEvent::Stream(StreamEvent::Started)
    }

    fn step(state: ConversationState, event: ConversationEvent) -> ConversationState {
        state.next(&event).expect("legal transition")
    }

    fn rejected(state: ConversationState, event: ConversationEvent) -> TransitionError {
        state.next(&event).expect_err("illegal transition")
    }

    #[test]
    fn submit_from_settled_states_starts_a_request() {
        for state in [
            ConversationState::Empty,
            ConversationState::Ready,
            ConversationState::Succeeded,
            ConversationState::Failed,
        ] {
            assert_eq!(
                step(state, ConversationEvent::Submit),
                ConversationState::Sending,
                "{state:?} must accept a new prompt"
            );
        }
    }

    #[test]
    fn streaming_path_reaches_succeeded() {
        let mut state = step(ConversationState::Empty, ConversationEvent::Submit);
        state = step(state, started());
        assert_eq!(state, ConversationState::Streaming);
        state = step(state, chunk());
        assert_eq!(state, ConversationState::Streaming);
        state = step(state, chunk());
        state = step(state, completed());
        assert_eq!(state, ConversationState::Succeeded);
    }

    #[test]
    fn chunk_without_start_is_tolerated() {
        let state = step(ConversationState::Empty, ConversationEvent::Submit);

        assert_eq!(step(state, chunk()), ConversationState::Streaming);
    }

    #[test]
    fn failed_terminal_maps_to_failed() {
        let sending = step(ConversationState::Ready, ConversationEvent::Submit);
        assert_eq!(step(sending, failed()), ConversationState::Failed);

        let streaming = step(sending, started());
        assert_eq!(step(streaming, failed()), ConversationState::Failed);
    }

    #[test]
    fn cancel_reaches_cancelling_and_returns_to_ready() {
        let sending = step(ConversationState::Empty, ConversationEvent::Submit);
        let cancelling = step(sending, ConversationEvent::Cancel);
        assert_eq!(cancelling, ConversationState::Cancelling);
        assert_eq!(step(cancelling, cancelled()), ConversationState::Ready);

        let streaming = step(sending, started());
        let cancelling = step(streaming, ConversationEvent::Cancel);
        assert_eq!(step(cancelling, cancelled()), ConversationState::Ready);
    }

    #[test]
    fn cancellation_terminal_without_intent_returns_to_ready() {
        let streaming = step(
            step(ConversationState::Empty, ConversationEvent::Submit),
            started(),
        );

        assert_eq!(step(streaming, cancelled()), ConversationState::Ready);
    }

    #[test]
    fn double_submit_is_rejected() {
        for state in [
            ConversationState::Sending,
            ConversationState::Streaming,
            ConversationState::Cancelling,
        ] {
            let error = rejected(state, ConversationEvent::Submit);
            assert_eq!(error.state, state);
            assert_eq!(error.event, "submit");
        }
    }

    #[test]
    fn late_chunks_are_rejected() {
        for state in [
            ConversationState::Ready,
            ConversationState::Succeeded,
            ConversationState::Failed,
            ConversationState::Cancelling,
        ] {
            assert_eq!(rejected(state, chunk()).event, "text_chunk");
        }
    }

    #[test]
    fn duplicate_terminals_are_rejected() {
        assert_eq!(
            rejected(ConversationState::Succeeded, completed()).event,
            "completed"
        );
        assert_eq!(
            rejected(ConversationState::Failed, failed()).event,
            "failed"
        );
        assert_eq!(
            rejected(ConversationState::Ready, cancelled()).event,
            "cancelled"
        );
    }

    #[test]
    fn completion_or_failure_after_cancel_is_rejected() {
        for event in [started(), chunk(), completed(), failed()] {
            let kind = event.kind();
            let error = rejected(ConversationState::Cancelling, event);
            assert_eq!(error.event, kind);
            assert_eq!(error.state, ConversationState::Cancelling);
        }
    }

    #[test]
    fn cancel_outside_an_active_request_is_rejected() {
        for state in [
            ConversationState::Empty,
            ConversationState::Ready,
            ConversationState::Succeeded,
            ConversationState::Failed,
        ] {
            assert_eq!(rejected(state, ConversationEvent::Cancel).event, "cancel");
        }
    }

    #[test]
    fn duplicate_start_is_rejected() {
        let streaming = step(
            step(ConversationState::Empty, ConversationEvent::Submit),
            started(),
        );

        assert_eq!(rejected(streaming, started()).event, "started");
    }

    #[test]
    fn early_events_before_a_submit_are_rejected() {
        for event in [started(), chunk(), completed(), failed(), cancelled()] {
            let kind = event.kind();
            assert_eq!(rejected(ConversationState::Empty, event).event, kind);
        }
    }

    #[test]
    fn errors_are_plain_language_and_payload_free() {
        let error = rejected(ConversationState::Streaming, ConversationEvent::Submit);

        assert!(error.message().contains("streaming"));
        assert!(error.message().contains("submit"));
        assert!(!error.message().contains("hidden payload"));

        let error = rejected(ConversationState::Cancelling, failed());
        assert!(!error.message().contains("temporarily unavailable"));
    }
}
