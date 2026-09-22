//! Core-owned conversation session and Tauri IPC for B04.
//!
//! The frontend sends one bounded, provider-neutral message. A worker exists
//! only while that request is active and emits the frozen neutral
//! [`StreamEvent`] contract. The normal runner is OpenCode Go; the fake runner
//! is injected only by tests or explicit debug configuration.

use std::io::Read;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::conversation::{ConversationEvent, ConversationState};
use crate::provider::contract::{
    self, ConversationId, ErrorCode, ModelId, NormalizedError, ProviderRequest, StreamEvent,
    UserMessage, PROVIDER_CONTRACT_VERSION,
};
use crate::provider::credential::{CredentialError, CredentialStatus, ProviderState};
use crate::provider::discovery::{
    CatalogState, DiscoveredModel, DiscoveryClient, ModelCache, ProtocolEndpoint, UreqTransport,
};
use crate::provider::execution::DEFAULT_TOTAL_TIMEOUT;
use crate::provider::failure::GoFailure;
use crate::provider::go::{
    headers, ChatStreamParser, GoConfig, GoTransport, SessionId, UreqGoTransport, DEFAULT_MODEL_ID,
    GO_CHAT_COMPLETIONS_ENDPOINT,
};
#[cfg(any(test, debug_assertions))]
use crate::provider::{
    execution::{ExecutionLimits, RequestOwner},
    fake::{FakeProvider, FakeScenario},
    normalize::StreamNormalizer,
};

pub const CONVERSATION_EVENT_NAME: &str = "conversation_event";
const READ_BUFFER_BYTES: usize = 8 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConversationSendRequest {
    pub contract_version: u32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConversationAccepted {
    pub contract_version: u32,
    pub conversation: ConversationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConversationEnvelope {
    pub contract_version: u32,
    pub conversation: ConversationId,
    pub event: StreamEvent,
}

trait ProviderRunner: Send + Sync {
    fn model(&self) -> ModelId;
    fn stream(&self, request: &ProviderRequest, emit: &mut dyn FnMut(StreamEvent));
}

struct LiveGoRunner {
    provider: ProviderState,
    catalog: Mutex<ModelCache>,
    started_at: Instant,
}

impl LiveGoRunner {
    fn new(provider: ProviderState) -> Self {
        Self {
            provider,
            catalog: Mutex::new(ModelCache::new()),
            started_at: Instant::now(),
        }
    }

    fn credential(&self) -> Result<crate::provider::credential::CredentialValue, NormalizedError> {
        match self.provider.credential_status() {
            CredentialStatus::Configured => {}
            CredentialStatus::NotConfigured => {
                return Err(normalized_error(
                    ErrorCode::AuthenticationFailed,
                    "No OpenCode Go credential is configured.",
                ));
            }
            CredentialStatus::Unavailable => {
                return Err(normalized_error(
                    ErrorCode::ProviderUnavailable,
                    "The credential store is not available on this system.",
                ));
            }
        }

        let reference = self.provider.credential_reference().ok_or_else(|| {
            normalized_error(
                ErrorCode::AuthenticationFailed,
                "No OpenCode Go credential is configured.",
            )
        })?;
        self.provider.resolve(&reference).map_err(credential_error)
    }

    fn run_live(
        &self,
        request: &ProviderRequest,
        emit: &mut dyn FnMut(StreamEvent),
    ) -> Result<(), NormalizedError> {
        let credential = self.credential()?;
        let now = self.started_at.elapsed();
        let cached = self
            .catalog
            .lock()
            .expect("model cache lock is not poisoned")
            .state(now);
        let catalog = match cached {
            CatalogState::Fresh(models) => models,
            CatalogState::Stale(_) | CatalogState::Empty => {
                let models = DiscoveryClient::new(UreqTransport::with_timeout(
                    crate::provider::execution::DEFAULT_CONNECT_TIMEOUT,
                ))
                .fetch(&credential)
                .map_err(go_error)?;
                self.catalog
                    .lock()
                    .expect("model cache lock is not poisoned")
                    .store(now, models.clone());
                models
            }
        };
        let model = select_default_model(&catalog).ok_or_else(|| {
            normalized_error(
                ErrorCode::InvalidInput,
                "The default model is not available on the supported OpenCode Go endpoint.",
            )
        })?;

        let config = GoConfig::new(GO_CHAT_COMPLETIONS_ENDPOINT, model)
            .map_err(|error| normalized_error(error.code(), &error.message()))?;
        let body = config
            .build_request_body(&request.message)
            .map_err(go_error)?;
        let session = SessionId::from_value(request.conversation.as_str())
            .map_err(|error| normalized_error(error.code(), &error.message()))?;
        let request_headers = headers(&credential, &session);
        let mut reader = UreqGoTransport::with_timeout(DEFAULT_TOTAL_TIMEOUT)
            .post(config.endpoint(), &request_headers, &body)
            .map_err(go_error)?;
        let mut parser = ChatStreamParser::new(request.conversation.clone());
        let mut buffer = [0_u8; READ_BUFFER_BYTES];
        let mut received = 0usize;

        while !parser.is_finished() {
            let count = reader.read(&mut buffer).map_err(|_| {
                normalized_error(
                    ErrorCode::ProviderUnavailable,
                    "The provider connection ended unexpectedly.",
                )
            })?;
            if count == 0 {
                break;
            }
            received = received.saturating_add(count);
            if received > contract::limits::MAX_RESPONSE_BYTES {
                return Err(normalized_error(
                    ErrorCode::ResponseTooLarge,
                    "The provider's response was larger than BrainRoot allows.",
                ));
            }
            for event in parser.push(&buffer[..count]).map_err(go_error)? {
                emit(event);
            }
        }

        for event in parser.finish() {
            emit(event);
        }
        Ok(())
    }
}

impl ProviderRunner for LiveGoRunner {
    fn model(&self) -> ModelId {
        ModelId::new(DEFAULT_MODEL_ID).expect("static default model is valid")
    }

    fn stream(&self, request: &ProviderRequest, emit: &mut dyn FnMut(StreamEvent)) {
        if let Err(error) = self.run_live(request, emit) {
            emit(StreamEvent::Failed { error });
        }
    }
}

#[cfg(any(test, debug_assertions))]
struct FakeRunner;

#[cfg(any(test, debug_assertions))]
impl ProviderRunner for FakeRunner {
    fn model(&self) -> ModelId {
        ModelId::new("fake-streaming").expect("static fake model is valid")
    }

    fn stream(&self, request: &ProviderRequest, emit: &mut dyn FnMut(StreamEvent)) {
        let mut owner = RequestOwner::start(
            FakeProvider.script(FakeScenario::Success),
            ExecutionLimits::default(),
            crate::provider::execution::CancellationToken::new(),
        );
        let mut normalizer = StreamNormalizer::new(request.conversation.clone());
        for execution_event in owner.advance(std::time::Duration::ZERO) {
            if normalizer.is_finished() {
                break;
            }
            if normalizer.push_execution(execution_event).is_err() {
                emit(StreamEvent::Failed {
                    error: normalized_error(
                        ErrorCode::MalformedResponse,
                        "The deterministic provider returned an unreadable response.",
                    ),
                });
                return;
            }
            for event in normalizer.drain() {
                emit(event);
            }
        }
    }
}

struct SessionInner {
    conversation: ConversationId,
    state: Mutex<ConversationState>,
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

    fn with_runner(runner: Arc<dyn ProviderRunner>) -> Self {
        Self {
            inner: Arc::new(SessionInner {
                conversation: ConversationId::new(uuid::Uuid::new_v4().to_string())
                    .expect("UUID conversation is valid"),
                state: Mutex::new(ConversationState::Empty),
            }),
            runner,
        }
    }

    fn start<F>(
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

        let accepted = ConversationAccepted {
            contract_version: PROVIDER_CONTRACT_VERSION,
            conversation: self.inner.conversation.clone(),
        };
        let inner = Arc::clone(&self.inner);
        let runner = Arc::clone(&self.runner);
        let handle = std::thread::spawn(move || run_request(inner, runner, request, emit));
        Ok((accepted, handle))
    }

    #[cfg(test)]
    fn state(&self) -> ConversationState {
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
    mut emit: F,
) where
    F: FnMut(ConversationEnvelope),
{
    let mut terminal = false;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        runner.stream(&request, &mut |event| {
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
        let event = StreamEvent::Failed {
            error: normalized_error(
                ErrorCode::ProviderUnavailable,
                "The provider request ended before a complete answer was received.",
            ),
        };
        if apply_stream_event(&inner, &event) {
            emit(ConversationEnvelope {
                contract_version: PROVIDER_CONTRACT_VERSION,
                conversation: inner.conversation.clone(),
                event,
            });
        }
    }
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

fn is_terminal(event: &StreamEvent) -> bool {
    matches!(
        event,
        StreamEvent::Completed { .. } | StreamEvent::Cancelled { .. } | StreamEvent::Failed { .. }
    )
}

fn select_default_model(models: &[DiscoveredModel]) -> Option<ModelId> {
    models
        .iter()
        .find(|model| {
            model.id.as_str() == DEFAULT_MODEL_ID
                && model.endpoint == ProtocolEndpoint::ChatCompletions
        })
        .map(|model| model.id.clone())
}

fn credential_error(error: CredentialError) -> NormalizedError {
    match error {
        CredentialError::BackendUnavailable => normalized_error(
            ErrorCode::ProviderUnavailable,
            "The credential store is not available on this system.",
        ),
        CredentialError::NotConfigured | CredentialError::UnknownReference => normalized_error(
            ErrorCode::AuthenticationFailed,
            "No OpenCode Go credential is configured.",
        ),
        CredentialError::Empty => normalized_error(
            ErrorCode::InvalidInput,
            "The configured credential is empty.",
        ),
    }
}

fn go_error(error: GoFailure) -> NormalizedError {
    normalized_error(error.code(), &error.message())
}

fn normalized_error(code: ErrorCode, message: &str) -> NormalizedError {
    NormalizedError {
        code,
        message: message.to_string(),
    }
}

#[tauri::command]
pub fn conversation_send(
    request: ConversationSendRequest,
    app: AppHandle,
    state: tauri::State<'_, ConversationSession>,
) -> Result<ConversationAccepted, NormalizedError> {
    let (accepted, _worker) = state.start(request, move |envelope| {
        let _ = app.emit(CONVERSATION_EVENT_NAME, envelope);
    })?;
    Ok(accepted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::contract::{Completion, StopReason};
    use crate::provider::discovery::{ModelPrivacy, PrivacyDisclosure};
    use std::sync::{mpsc, Barrier};

    fn input(message: &str) -> ConversationSendRequest {
        ConversationSendRequest {
            contract_version: PROVIDER_CONTRACT_VERSION,
            message: message.to_string(),
        }
    }

    fn model(id: &str, endpoint: ProtocolEndpoint) -> DiscoveredModel {
        DiscoveredModel {
            id: ModelId::new(id).expect("valid model"),
            display_name: id.to_string(),
            endpoint,
            privacy: ModelPrivacy {
                training: PrivacyDisclosure::Unknown,
                retention: PrivacyDisclosure::Unknown,
            },
        }
    }

    #[test]
    fn default_model_requires_exact_id_and_supported_endpoint() {
        let catalog = vec![
            model(DEFAULT_MODEL_ID, ProtocolEndpoint::Messages),
            model("another-model", ProtocolEndpoint::ChatCompletions),
        ];
        assert_eq!(select_default_model(&catalog), None);

        let mut compatible = catalog;
        compatible.push(model(DEFAULT_MODEL_ID, ProtocolEndpoint::ChatCompletions));
        assert_eq!(
            select_default_model(&compatible)
                .expect("default is compatible")
                .as_str(),
            DEFAULT_MODEL_ID
        );
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

        fn stream(&self, request: &ProviderRequest, emit: &mut dyn FnMut(StreamEvent)) {
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

    #[test]
    fn envelope_serialization_is_versioned_and_provider_neutral() {
        let envelope = ConversationEnvelope {
            contract_version: PROVIDER_CONTRACT_VERSION,
            conversation: ConversationId::new("conversation-test").expect("valid"),
            event: StreamEvent::TextChunk {
                text: "safe text".to_string(),
            },
        };
        let value = serde_json::to_value(envelope).expect("serializes");

        assert_eq!(value["contractVersion"], PROVIDER_CONTRACT_VERSION);
        assert_eq!(value["conversation"], "conversation-test");
        assert_eq!(value["event"]["type"], "text_chunk");
        assert_eq!(value["event"]["text"], "safe text");
        let rendered = value.to_string();
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("credential"));
        assert!(!rendered.contains("opencode"));
    }
}
