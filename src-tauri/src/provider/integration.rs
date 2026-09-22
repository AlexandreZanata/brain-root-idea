//! Provider contract integration test (test-only).
//!
//! Drives the fake provider end to end the way the future UI intent will:
//! credential-status pre-flight, discovery, a validated request, streamed
//! chunks to completion, cancellation mid-stream, and a plain-language failure.
//! Time is fed explicitly and every loop is bounded; nothing runs in the
//! background.

use std::time::Duration;

use super::contract::{
    CancellationReason, ConversationId, ErrorCode, ModelId, ProviderRequest, StreamEvent,
    UserMessage,
};
use super::credential::{CredentialReference, CredentialStatus, CredentialValue, ProviderState};
use super::execution::{CancellationToken, ExecutionLimits, RequestOwner};
use super::fake::{FakeProvider, FakeScenario};
use super::normalize::StreamNormalizer;

const STEP: Duration = Duration::from_millis(10);
const MAX_STEPS: usize = 100;

/// One deterministic run of a scripted request, mirroring the core sequence the
/// future UI intent will call.
struct TestUiIntent {
    owner: RequestOwner,
    normalizer: StreamNormalizer,
}

impl TestUiIntent {
    fn send(scenario: FakeScenario, model: &ModelId) -> Self {
        let conversation =
            ConversationId::new("conversation-integration").expect("valid conversation");
        let request = ProviderRequest::new(
            conversation.clone(),
            model.clone(),
            UserMessage::new("Describe a small change").expect("valid message"),
        );
        request
            .validate()
            .expect("the request satisfies the contract");

        let owner = RequestOwner::start(
            FakeProvider.script(scenario),
            ExecutionLimits::default(),
            CancellationToken::new(),
        );
        Self {
            owner,
            normalizer: StreamNormalizer::new(conversation),
        }
    }

    fn step(&mut self) -> Vec<StreamEvent> {
        for event in self.owner.advance(STEP) {
            if self.normalizer.is_finished() {
                // The first terminal wins; late executor terminals are ignored,
                // exactly as the UI will ignore late provider events.
                break;
            }
            self.normalizer
                .push_execution(event)
                .expect("the normalizer accepts");
        }
        self.normalizer.drain()
    }

    fn run_until_terminal(&mut self) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        for _ in 0..MAX_STEPS {
            events.extend(self.step());
            if self.owner.is_terminal() {
                break;
            }
        }
        events
    }

    fn cancel(&mut self, reason: CancellationReason) -> Vec<StreamEvent> {
        if let Some(event) = self.owner.cancel(reason) {
            self.normalizer
                .push_execution(event)
                .expect("the normalizer accepts");
        }
        self.normalizer.drain()
    }
}

fn terminal_kinds(events: &[StreamEvent]) -> Vec<&'static str> {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::Completed { .. } => Some("completed"),
            StreamEvent::Cancelled { .. } => Some("cancelled"),
            StreamEvent::Failed { .. } => Some("failed"),
            _ => None,
        })
        .collect()
}

fn text_of(events: &[StreamEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::TextChunk { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn streaming_model() -> ModelId {
    ModelId::new("fake-streaming").expect("static model is valid")
}

#[test]
fn discovery_request_stream_and_complete() {
    let state = ProviderState::new();
    assert_eq!(state.credential_status(), CredentialStatus::NotConfigured);
    let reference = CredentialReference::new("default").expect("valid reference");
    state.set_credential(
        reference.clone(),
        CredentialValue::new("fake-secret-value").expect("valid value"),
    );
    assert_eq!(state.credential_status(), CredentialStatus::Configured);

    let models = FakeProvider.models();
    assert_eq!(models.len(), 2);
    assert!(models.iter().all(|model| model.supports_streaming));
    let model = models[0].id.clone();

    let mut intent = TestUiIntent::send(FakeScenario::Success, &model);
    let events = intent.run_until_terminal();

    assert_eq!(text_of(&events), "Hello, BrainRoot");
    assert_eq!(terminal_kinds(&events), vec!["completed"]);
    assert!(intent.owner.is_terminal());
    assert_eq!(intent.owner.pending_steps(), 0);
    assert!(intent.normalizer.is_finished());
    assert!(intent.step().is_empty());
}

#[test]
fn cancel_mid_stream_yields_one_cancelled_terminal() {
    let mut intent = TestUiIntent::send(FakeScenario::NeverEnding, &streaming_model());

    let streamed = intent.run_until_terminal();
    assert!(terminal_kinds(&streamed).is_empty());
    assert!(!streamed.is_empty());
    assert!(!intent.owner.is_terminal());

    let cancelled = intent.cancel(CancellationReason::UserRequested);
    assert_eq!(terminal_kinds(&cancelled), vec!["cancelled"]);
    assert!(intent.owner.is_terminal());
    assert_eq!(intent.owner.pending_steps(), 0);
    assert!(intent.step().is_empty());
    assert!(intent.cancel(CancellationReason::UserRequested).is_empty());
}

#[test]
fn idle_intent_produces_nothing_until_advanced() {
    let mut intent = TestUiIntent::send(FakeScenario::Success, &streaming_model());

    assert!(!intent.owner.is_terminal());
    assert!(!intent.normalizer.is_finished());
    assert_eq!(intent.normalizer.pending_len(), 0);
    assert!(intent.owner.pending_steps() > 0);
    assert!(intent.normalizer.drain().is_empty());
}

#[test]
fn failure_scenario_is_plain_language() {
    let mut intent = TestUiIntent::send(FakeScenario::AuthFailure, &streaming_model());

    let events = intent.run_until_terminal();
    assert_eq!(terminal_kinds(&events), vec!["failed"]);
    let StreamEvent::Failed { error } = &events[0] else {
        panic!("expected a failed event");
    };
    assert_eq!(error.code, ErrorCode::AuthenticationFailed);
    assert!(error.message.contains("credential"));
    assert!(intent.owner.is_terminal());
    assert_eq!(intent.owner.pending_steps(), 0);
}
