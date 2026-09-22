//! Deterministic MVP-0 fake-provider end-to-end journey (test-only).
//!
//! Drives the product session API through the journey the plan records:
//! configured fake → discover → prompt → stream → complete → second prompt →
//! cancel → close, plus a neutral technical error, asserting user states and
//! that every request releases its worker. The app UI still has no approved
//! automation, so the process-level launch/readiness/close lives in
//! `scripts/smoke-linux.sh`, which `scripts/e2e-linux.sh` runs when a display
//! is available.

use std::sync::{mpsc, Arc, Barrier};

use super::runner::ProviderRunner;
use super::runtime::ConversationSession;
use super::state::ConversationState;
use super::wire::ConversationSendRequest;
use crate::provider::contract::{
    Cancellation, CancellationReason, Completion, ErrorCode, ModelId, NormalizedError,
    ProviderRequest, StopReason, StreamEvent, PROVIDER_CONTRACT_VERSION,
};
use crate::provider::execution::CancellationToken;

fn input(message: &str) -> ConversationSendRequest {
    ConversationSendRequest {
        contract_version: PROVIDER_CONTRACT_VERSION,
        message: message.to_string(),
    }
}

fn streamed_text(events: &[super::wire::ConversationEnvelope]) -> String {
    events
        .iter()
        .filter_map(|envelope| match &envelope.event {
            StreamEvent::TextChunk { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

struct StoppableRunner {
    entered: mpsc::Sender<()>,
    release: Arc<Barrier>,
}

impl ProviderRunner for StoppableRunner {
    fn model(&self) -> ModelId {
        ModelId::new("stoppable-e2e").expect("valid model")
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
            emit(StreamEvent::Cancelled {
                cancellation: Cancellation {
                    conversation: request.conversation.clone(),
                    reason: CancellationReason::UserRequested,
                },
            });
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

struct FailingRunner;

impl ProviderRunner for FailingRunner {
    fn model(&self) -> ModelId {
        ModelId::new("failing-e2e").expect("valid model")
    }

    fn stream(
        &self,
        _request: &ProviderRequest,
        _cancel: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent),
    ) {
        emit(StreamEvent::Started);
        emit(StreamEvent::Failed {
            error: NormalizedError {
                code: ErrorCode::ProviderUnavailable,
                message: "The provider is temporarily unavailable. Try again shortly.".to_string(),
            },
        });
    }
}

#[test]
fn fake_journey_completes_then_cancels_and_releases() {
    let session = ConversationSession::debug_fake();
    assert!(!session.is_active());
    assert_eq!(session.model_id().as_str(), "fake-streaming");
    assert_eq!(session.state(), ConversationState::Empty);

    let (sender, receiver) = mpsc::channel();
    let (accepted, worker) = session
        .start(input("First prompt"), move |envelope| {
            sender.send(envelope).expect("receiver remains open");
        })
        .expect("prompt one is accepted");
    assert_eq!(accepted.contract_version, PROVIDER_CONTRACT_VERSION);
    worker.join().expect("prompt one worker completes");

    let events: Vec<_> = receiver.try_iter().collect();
    assert_eq!(streamed_text(&events), "Hello, BrainRoot");
    assert!(matches!(
        events.last().map(|envelope| &envelope.event),
        Some(StreamEvent::Completed { .. })
    ));
    assert_eq!(session.state(), ConversationState::Succeeded);
    assert!(!session.is_active());

    let (entered_sender, entered_receiver) = mpsc::channel();
    let release = Arc::new(Barrier::new(2));
    let cancellable = ConversationSession::with_runner(Arc::new(StoppableRunner {
        entered: entered_sender,
        release: Arc::clone(&release),
    }));
    let (sender, receiver) = mpsc::channel();
    let (_, worker) = cancellable
        .start(input("Second prompt"), move |envelope| {
            sender.send(envelope).expect("receiver remains open");
        })
        .expect("prompt two is accepted");
    entered_receiver.recv().expect("second worker started");
    assert!(cancellable.is_active());

    cancellable.cancel().expect("cancel is accepted");
    assert_eq!(cancellable.state(), ConversationState::Cancelling);
    release.wait();
    worker.join().expect("second worker completes");

    let events: Vec<_> = receiver.try_iter().collect();
    assert_eq!(
        events
            .iter()
            .filter(|envelope| matches!(&envelope.event, StreamEvent::Cancelled { .. }))
            .count(),
        1
    );
    assert_eq!(cancellable.state(), ConversationState::Ready);
    assert!(!cancellable.is_active());
}

#[test]
fn technical_errors_are_neutral_and_cleanup_holds() {
    let session = ConversationSession::with_runner(Arc::new(FailingRunner));
    let (sender, receiver) = mpsc::channel();
    let (_, worker) = session
        .start(input("Break it"), move |envelope| {
            sender.send(envelope).expect("receiver remains open");
        })
        .expect("request is accepted");
    worker.join().expect("worker completes");

    let events: Vec<_> = receiver.try_iter().collect();
    assert_eq!(
        events
            .iter()
            .filter(|envelope| matches!(&envelope.event, StreamEvent::Failed { .. }))
            .count(),
        1
    );
    let Some(StreamEvent::Failed { error }) = events.last().map(|envelope| &envelope.event) else {
        panic!("expected a failed terminal");
    };
    assert_eq!(error.code, ErrorCode::ProviderUnavailable);
    assert!(error.message.contains("Try again shortly"));
    assert_eq!(session.state(), ConversationState::Failed);
    assert!(!session.is_active());
}
