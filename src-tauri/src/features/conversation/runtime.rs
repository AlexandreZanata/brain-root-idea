//! Conversation runtime: one active request, one worker, cancellation.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[cfg(any(test, debug_assertions))]
use super::runner::FakeRunner;
use super::runner::{cancelled_event, is_terminal, normalized_error, LiveGoRunner, ProviderRunner};
use super::state::{ConversationEvent, ConversationState};
use super::wire::{ConversationAccepted, ConversationEnvelope, ConversationSendRequest};
use crate::provider::contract::{
    CancellationReason, ConversationId, ErrorCode, NormalizedError, ProviderRequest, StreamEvent,
    UserMessage, PROVIDER_CONTRACT_VERSION,
};
use crate::provider::credential::ProviderState;
use crate::provider::execution::CancellationToken;

struct SessionInner {
    conversation: ConversationId,
    state: Mutex<ConversationState>,
    active: AtomicBool,
    cancel: Mutex<Option<CancellationToken>>,
}

#[derive(Clone)]
pub struct ConversationSession {
    inner: Arc<SessionInner>,
    runner: Arc<dyn ProviderRunner>,
}

impl ConversationSession {
    pub fn live(provider: ProviderState) -> Self {
        Self::with_runner(Arc::new(LiveGoRunner::new(provider)))
    }

    #[cfg(any(test, debug_assertions))]
    pub fn debug_fake() -> Self {
        Self::with_runner(Arc::new(FakeRunner))
    }

    pub(super) fn with_runner(runner: Arc<dyn ProviderRunner>) -> Self {
        Self {
            inner: Arc::new(SessionInner {
                conversation: ConversationId::new(uuid::Uuid::new_v4().to_string())
                    .expect("UUID conversation is valid"),
                state: Mutex::new(ConversationState::Empty),
                active: AtomicBool::new(false),
                cancel: Mutex::new(None),
            }),
            runner,
        }
    }

    pub(super) fn start<F>(
        &self,
        input: ConversationSendRequest,
        emit: F,
    ) -> Result<(ConversationAccepted, JoinHandle<()>), NormalizedError>
    where
        F: FnMut(ConversationEnvelope) + Send + 'static,
    {
        if input.contract_version != PROVIDER_CONTRACT_VERSION {
            return Err(normalized_error(
                ErrorCode::UnsupportedContractVersion,
                &format!(
                    "This BrainRoot build speaks contract version {}, but version {} was requested.",
                    PROVIDER_CONTRACT_VERSION, input.contract_version
                ),
            ));
        }
        let message = UserMessage::new(input.message)
            .map_err(|error| normalized_error(error.code(), &error.message()))?;
        let request = ProviderRequest::new(
            self.inner.conversation.clone(),
            self.runner.model(),
            message,
        );
        request
            .validate()
            .map_err(|error| normalized_error(error.code(), &error.message()))?;

        {
            let mut state = self
                .inner
                .state
                .lock()
                .expect("conversation state lock is not poisoned");
            *state = state
                .next(&ConversationEvent::Submit)
                .map_err(|error| normalized_error(ErrorCode::InvalidState, &error.message()))?;
        }

        let token = CancellationToken::new();
        *self
            .inner
            .cancel
            .lock()
            .expect("cancel slot lock is not poisoned") = Some(token.clone());
        self.inner.active.store(true, Ordering::SeqCst);

        let accepted = ConversationAccepted {
            contract_version: PROVIDER_CONTRACT_VERSION,
            conversation: self.inner.conversation.clone(),
        };
        let inner = Arc::clone(&self.inner);
        let runner = Arc::clone(&self.runner);
        let handle = std::thread::spawn(move || run_request(inner, runner, request, token, emit));
        Ok((accepted, handle))
    }

    /// Moves the conversation to `CANCELLING` immediately and signals the
    /// active request. A second cancel or a cancel without an active request
    /// is rejected.
    pub(crate) fn cancel(&self) -> Result<(), NormalizedError> {
        {
            let mut state = self
                .inner
                .state
                .lock()
                .expect("conversation state lock is not poisoned");
            let next = state.next(&ConversationEvent::Cancel).map_err(|_| {
                normalized_error(
                    ErrorCode::InvalidState,
                    "There is no active request to cancel.",
                )
            })?;
            *state = next;
        }

        let token = self
            .inner
            .cancel
            .lock()
            .expect("cancel slot lock is not poisoned")
            .clone();
        if let Some(token) = token {
            token.cancel(CancellationReason::UserRequested);
        }
        Ok(())
    }

    pub(super) fn conversation_id(&self) -> ConversationId {
        self.inner.conversation.clone()
    }

    #[cfg(test)]
    pub(super) fn model_id(&self) -> crate::provider::contract::ModelId {
        self.runner.model()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.inner.active.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(super) fn state(&self) -> ConversationState {
        *self
            .inner
            .state
            .lock()
            .expect("conversation state lock is not poisoned")
    }
}

fn run_request<F>(
    inner: Arc<SessionInner>,
    runner: Arc<dyn ProviderRunner>,
    request: ProviderRequest,
    cancel: CancellationToken,
    mut emit: F,
) where
    F: FnMut(ConversationEnvelope),
{
    let mut terminal = false;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        runner.stream(&request, &cancel, &mut |event| {
            if terminal {
                return;
            }
            if apply_stream_event(&inner, &event) {
                terminal = is_terminal(&event);
                emit(ConversationEnvelope {
                    contract_version: PROVIDER_CONTRACT_VERSION,
                    conversation: inner.conversation.clone(),
                    event,
                });
            }
        });
    }));

    if outcome.is_err() || !terminal {
        let event = if current_state(&inner) == ConversationState::Cancelling {
            cancelled_event(&inner.conversation)
        } else {
            StreamEvent::Failed {
                error: normalized_error(
                    ErrorCode::ProviderUnavailable,
                    "The provider request ended before a complete answer was received.",
                ),
            }
        };
        if apply_stream_event(&inner, &event) {
            emit(ConversationEnvelope {
                contract_version: PROVIDER_CONTRACT_VERSION,
                conversation: inner.conversation.clone(),
                event,
            });
        }
    }

    inner.active.store(false, Ordering::SeqCst);
    *inner
        .cancel
        .lock()
        .expect("cancel slot lock is not poisoned") = None;
}

fn current_state(inner: &SessionInner) -> ConversationState {
    *inner
        .state
        .lock()
        .expect("conversation state lock is not poisoned")
}

fn apply_stream_event(inner: &SessionInner, event: &StreamEvent) -> bool {
    if !event_matches_conversation(event, &inner.conversation) {
        return false;
    }
    let mut state = inner
        .state
        .lock()
        .expect("conversation state lock is not poisoned");
    let Ok(next) = state.next(&ConversationEvent::Stream(event.clone())) else {
        return false;
    };
    *state = next;
    true
}

fn event_matches_conversation(event: &StreamEvent, expected: &ConversationId) -> bool {
    match event {
        StreamEvent::Completed { completion } => &completion.conversation == expected,
        StreamEvent::Cancelled { cancellation } => &cancellation.conversation == expected,
        StreamEvent::Started | StreamEvent::TextChunk { .. } | StreamEvent::Failed { .. } => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::contract::{self, Completion, ModelId, StopReason};
    use std::sync::{mpsc, Barrier};

    fn input(message: &str) -> ConversationSendRequest {
        ConversationSendRequest {
            contract_version: PROVIDER_CONTRACT_VERSION,
            message: message.to_string(),
        }
    }

    #[test]
    fn fake_stream_emits_only_neutral_events_and_releases_ownership() {
        let session = ConversationSession::debug_fake();
        let (sender, receiver) = mpsc::channel();
        let (accepted, worker) = session
            .start(input("Build a small example"), move |event| {
                sender.send(event).expect("receiver remains open");
            })
            .expect("request accepted");
        worker.join().expect("worker completes");
        let events: Vec<_> = receiver.try_iter().collect();

        assert_eq!(accepted.contract_version, PROVIDER_CONTRACT_VERSION);
        assert_eq!(accepted.conversation, session.inner.conversation);
        assert_eq!(
            events
                .iter()
                .filter_map(|envelope| match &envelope.event {
                    StreamEvent::TextChunk { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<String>(),
            "Hello, BrainRoot"
        );
        assert!(matches!(
            events.last().map(|envelope| &envelope.event),
            Some(StreamEvent::Completed { .. })
        ));
        assert_eq!(session.state(), ConversationState::Succeeded);
        assert!(!session.is_active());

        let (_, second_worker) = session
            .start(input("Build another example"), |_| {})
            .expect("terminal state releases ownership");
        second_worker.join().expect("second worker completes");
    }

    struct BlockingRunner {
        entered: mpsc::Sender<()>,
        release: Arc<Barrier>,
    }

    impl ProviderRunner for BlockingRunner {
        fn model(&self) -> ModelId {
            ModelId::new("blocking-test").expect("valid model")
        }

        fn stream(
            &self,
            request: &ProviderRequest,
            cancel: &CancellationToken,
            emit: &mut dyn FnMut(StreamEvent),
        ) {
            emit(StreamEvent::Started);
            self.entered.send(()).expect("test receiver remains open");
            self.release.wait();
            if cancel.is_cancelled() {
                emit(cancelled_event(&request.conversation));
            } else {
                emit(StreamEvent::Completed {
                    completion: Completion {
                        conversation: request.conversation.clone(),
                        stop_reason: StopReason::EndTurn,
                    },
                });
            }
        }
    }

    struct IgnoringCancelRunner {
        entered: mpsc::Sender<()>,
        release: Arc<Barrier>,
    }

    impl ProviderRunner for IgnoringCancelRunner {
        fn model(&self) -> ModelId {
            ModelId::new("ignoring-cancel-test").expect("valid model")
        }

        fn stream(
            &self,
            request: &ProviderRequest,
            _cancel: &CancellationToken,
            emit: &mut dyn FnMut(StreamEvent),
        ) {
            emit(StreamEvent::Started);
            self.entered.send(()).expect("test receiver remains open");
            self.release.wait();
            emit(StreamEvent::Completed {
                completion: Completion {
                    conversation: request.conversation.clone(),
                    stop_reason: StopReason::EndTurn,
                },
            });
        }
    }

    #[test]
    fn duplicate_submit_is_rejected_while_worker_owns_conversation() {
        let (entered_sender, entered_receiver) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));
        let session = ConversationSession::with_runner(Arc::new(BlockingRunner {
            entered: entered_sender,
            release: Arc::clone(&release),
        }));
        let (_, worker) = session
            .start(input("First"), |_| {})
            .expect("first accepted");
        entered_receiver.recv().expect("worker started");

        let error = session
            .start(input("Duplicate"), |_| {})
            .expect_err("duplicate rejected");
        assert_eq!(error.code, ErrorCode::InvalidState);

        release.wait();
        worker.join().expect("worker completes");
        assert_eq!(session.state(), ConversationState::Succeeded);
        assert!(!session.is_active());
    }

    #[test]
    fn cancel_moves_to_cancelling_and_emits_one_cancelled_terminal() {
        let (entered_sender, entered_receiver) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));
        let session = ConversationSession::with_runner(Arc::new(BlockingRunner {
            entered: entered_sender,
            release: Arc::clone(&release),
        }));
        let (sender, receiver) = mpsc::channel();
        let (_, worker) = session
            .start(input("Cancel me"), move |envelope| {
                sender.send(envelope).expect("receiver remains open");
            })
            .expect("request accepted");
        entered_receiver.recv().expect("worker started");

        session.cancel().expect("cancel accepted");
        assert_eq!(session.state(), ConversationState::Cancelling);
        assert!(session.is_active());

        release.wait();
        worker.join().expect("worker completes");
        let events: Vec<_> = receiver.try_iter().collect();

        assert_eq!(
            events
                .iter()
                .filter(|envelope| matches!(&envelope.event, StreamEvent::Cancelled { .. }))
                .count(),
            1
        );
        assert!(matches!(
            events.last().map(|envelope| &envelope.event),
            Some(StreamEvent::Cancelled { .. })
        ));
        assert_eq!(session.state(), ConversationState::Ready);
        assert!(!session.is_active());

        let (_, next_worker) = session
            .start(input("Next request"), |_| {})
            .expect("cancel releases ownership");
        release.wait();
        next_worker.join().expect("next worker completes");
    }

    #[test]
    fn late_completion_after_cancel_never_emits_completed_or_failed() {
        let (entered_sender, entered_receiver) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));
        let session = ConversationSession::with_runner(Arc::new(IgnoringCancelRunner {
            entered: entered_sender,
            release: Arc::clone(&release),
        }));
        let (sender, receiver) = mpsc::channel();
        let (_, worker) = session
            .start(input("Ignore the cancel"), move |envelope| {
                sender.send(envelope).expect("receiver remains open");
            })
            .expect("request accepted");
        entered_receiver.recv().expect("worker started");
        session.cancel().expect("cancel accepted");

        release.wait();
        worker.join().expect("worker completes");
        let events: Vec<_> = receiver.try_iter().collect();

        assert!(events.iter().all(|envelope| !matches!(
            &envelope.event,
            StreamEvent::Completed { .. } | StreamEvent::Failed { .. }
        )));
        assert_eq!(
            events
                .iter()
                .filter(|envelope| matches!(&envelope.event, StreamEvent::Cancelled { .. }))
                .count(),
            1
        );
        assert_eq!(session.state(), ConversationState::Ready);
        assert!(!session.is_active());
    }

    #[test]
    fn cancel_without_an_active_request_is_rejected() {
        let session = ConversationSession::debug_fake();
        let error = session.cancel().expect_err("nothing to cancel");
        assert_eq!(error.code, ErrorCode::InvalidState);

        let (_, worker) = session
            .start(input("Finish first"), |_| {})
            .expect("request accepted");
        worker.join().expect("worker completes");
        let error = session.cancel().expect_err("no active request");
        assert_eq!(error.code, ErrorCode::InvalidState);
    }

    #[test]
    fn second_cancel_while_cancelling_is_rejected() {
        let (entered_sender, entered_receiver) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));
        let session = ConversationSession::with_runner(Arc::new(BlockingRunner {
            entered: entered_sender,
            release: Arc::clone(&release),
        }));
        let (_, worker) = session
            .start(input("Cancel twice"), |_| {})
            .expect("request accepted");
        entered_receiver.recv().expect("worker started");

        session.cancel().expect("first cancel accepted");
        let error = session.cancel().expect_err("second cancel rejected");
        assert_eq!(error.code, ErrorCode::InvalidState);

        release.wait();
        worker.join().expect("worker completes");
        assert!(!session.is_active());
    }

    #[test]
    fn invalid_contract_and_messages_are_rejected_before_a_worker_starts() {
        let session = ConversationSession::debug_fake();
        let error = session
            .start(
                ConversationSendRequest {
                    contract_version: PROVIDER_CONTRACT_VERSION + 1,
                    message: "hello".to_string(),
                },
                |_| {},
            )
            .expect_err("version rejected");
        assert_eq!(error.code, ErrorCode::UnsupportedContractVersion);

        let error = session
            .start(input("   "), |_| {})
            .expect_err("empty rejected");
        assert_eq!(error.code, ErrorCode::InvalidInput);

        let oversized = "x".repeat(contract::limits::MAX_USER_MESSAGE_BYTES + 1);
        let error = session
            .start(input(&oversized), |_| {})
            .expect_err("oversized rejected");
        assert_eq!(error.code, ErrorCode::RequestTooLarge);
        assert_eq!(session.state(), ConversationState::Empty);
    }
}
