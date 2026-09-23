//! Provider runners for the conversation feature.

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::catalog::select_default_model;
use crate::provider::contract::{
    self, Cancellation, CancellationReason, ConversationId, ErrorCode, ModelId, NormalizedError,
    ProviderRequest, StreamEvent,
};
use crate::provider::credential::{CredentialError, CredentialStatus, ProviderState};
use crate::provider::discovery::{CatalogState, DiscoveryClient, ModelCache, UreqTransport};
use crate::provider::execution::{CancellationToken, DEFAULT_TOTAL_TIMEOUT};
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

const READ_BUFFER_BYTES: usize = 8 * 1024;

pub(super) trait ProviderRunner: Send + Sync {
    fn model(&self) -> ModelId;
    fn stream(
        &self,
        request: &ProviderRequest,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent),
    );
}

pub(super) struct LiveGoRunner {
    provider: ProviderState,
    transport: Arc<dyn GoTransport + Send + Sync>,
    catalog: Mutex<ModelCache>,
    started_at: Instant,
}

impl LiveGoRunner {
    pub(super) fn new(provider: ProviderState) -> Self {
        Self::with_transport(
            provider,
            Arc::new(UreqGoTransport::with_timeout(DEFAULT_TOTAL_TIMEOUT)),
        )
    }

    fn with_transport(
        provider: ProviderState,
        transport: Arc<dyn GoTransport + Send + Sync>,
    ) -> Self {
        Self {
            provider,
            transport,
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
                    "No OpenCode Go credential is configured. Add the key to the system credential store and try again.",
                ));
            }
            CredentialStatus::Unavailable => {
                return Err(normalized_error(
                    ErrorCode::ProviderUnavailable,
                    "The credential store is not available on this system. Prompts cannot be sent until a credential store is available.",
                ));
            }
        }

        let reference = self.provider.credential_reference().ok_or_else(|| {
            normalized_error(
                ErrorCode::AuthenticationFailed,
                "No OpenCode Go credential is configured. Add the key to the system credential store and try again.",
            )
        })?;
        self.provider.resolve(&reference).map_err(credential_error)
    }

    fn run_live(
        &self,
        request: &ProviderRequest,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent),
    ) -> Result<(), NormalizedError> {
        let credential = self.credential()?;
        if cancel.is_cancelled() {
            emit(cancelled_event(&request.conversation));
            return Ok(());
        }
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
                "No compatible OpenCode Go model is available right now. Try again later.",
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
        let mut reader = self
            .transport
            .post(config.endpoint(), &request_headers, &body)
            .map_err(go_error)?;
        let mut parser = ChatStreamParser::new(request.conversation.clone());
        let mut buffer = [0_u8; READ_BUFFER_BYTES];
        let mut received = 0usize;

        while !parser.is_finished() {
            if cancel.is_cancelled() {
                emit(cancelled_event(&request.conversation));
                return Ok(());
            }
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
                if cancel.is_cancelled() {
                    emit(cancelled_event(&request.conversation));
                    return Ok(());
                }
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

    fn stream(
        &self,
        request: &ProviderRequest,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent),
    ) {
        if let Err(error) = self.run_live(request, cancel, emit) {
            emit(StreamEvent::Failed { error });
        }
    }
}

#[cfg(any(test, debug_assertions))]
pub(super) struct FakeRunner;

#[cfg(any(test, debug_assertions))]
impl ProviderRunner for FakeRunner {
    fn model(&self) -> ModelId {
        ModelId::new("fake-streaming").expect("static fake model is valid")
    }

    fn stream(
        &self,
        request: &ProviderRequest,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(StreamEvent),
    ) {
        let mut owner = RequestOwner::start(
            FakeProvider.script(FakeScenario::Success),
            ExecutionLimits::default(),
            cancel.clone(),
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

pub(super) fn credential_error(error: CredentialError) -> NormalizedError {
    match error {
        CredentialError::BackendUnavailable => normalized_error(
            ErrorCode::ProviderUnavailable,
            "The credential store is not available on this system. Prompts cannot be sent until a credential store is available.",
        ),
        CredentialError::NotConfigured | CredentialError::UnknownReference => normalized_error(
            ErrorCode::AuthenticationFailed,
            "No OpenCode Go credential is configured. Add the key to the system credential store and try again.",
        ),
        CredentialError::Empty => normalized_error(
            ErrorCode::InvalidInput,
            "The configured credential is empty.",
        ),
    }
}

pub(super) fn go_error(error: GoFailure) -> NormalizedError {
    normalized_error(error.code(), &error.message())
}

pub(super) fn normalized_error(code: ErrorCode, message: &str) -> NormalizedError {
    NormalizedError {
        code,
        message: message.to_string(),
    }
}

pub(super) fn cancelled_event(conversation: &ConversationId) -> StreamEvent {
    StreamEvent::Cancelled {
        cancellation: Cancellation {
            conversation: conversation.clone(),
            reason: CancellationReason::UserRequested,
        },
    }
}

pub(super) fn is_terminal(event: &StreamEvent) -> bool {
    matches!(
        event,
        StreamEvent::Completed { .. } | StreamEvent::Cancelled { .. } | StreamEvent::Failed { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::credential::{CredentialReference, CredentialStore, CredentialValue};

    struct UnavailableStore;

    impl CredentialStore for UnavailableStore {
        fn status(&self) -> CredentialStatus {
            CredentialStatus::Unavailable
        }

        fn reference(&self) -> Option<CredentialReference> {
            None
        }

        fn resolve(
            &self,
            _reference: &CredentialReference,
        ) -> Result<CredentialValue, CredentialError> {
            Err(CredentialError::BackendUnavailable)
        }

        fn set(
            &self,
            _reference: CredentialReference,
            _value: CredentialValue,
        ) -> Result<(), CredentialError> {
            Err(CredentialError::BackendUnavailable)
        }
    }

    #[test]
    fn unconfigured_credential_explains_the_next_action() {
        let runner = LiveGoRunner::new(ProviderState::new());

        let error = runner
            .credential()
            .expect_err("no credential is configured");
        assert_eq!(error.code, ErrorCode::AuthenticationFailed);
        assert!(error
            .message
            .contains("Add the key to the system credential store"));
    }

    #[test]
    fn unavailable_credential_store_explains_the_next_action() {
        let runner = LiveGoRunner::new(ProviderState::with_store(UnavailableStore));

        let error = runner.credential().expect_err("the store is unavailable");
        assert_eq!(error.code, ErrorCode::ProviderUnavailable);
        assert!(error.message.contains("Prompts cannot be sent"));
    }

    struct FakeTransport {
        body: Result<Vec<u8>, GoFailure>,
    }

    impl GoTransport for FakeTransport {
        fn post(
            &self,
            _url: &str,
            _headers: &[(String, String)],
            _body: &str,
        ) -> Result<Box<dyn Read + Send>, GoFailure> {
            match &self.body {
                Ok(body) => Ok(Box::new(std::io::Cursor::new(body.clone()))),
                Err(failure) => Err(*failure),
            }
        }
    }

    fn configured_runner(body: Result<Vec<u8>, GoFailure>) -> LiveGoRunner {
        let provider = ProviderState::new();
        let reference = CredentialReference::new("default").expect("valid reference");
        provider
            .set_credential(
                reference,
                CredentialValue::new("fake-secret-value").expect("valid credential"),
            )
            .expect("session store accepts");

        let runner = LiveGoRunner::with_transport(provider, Arc::new(FakeTransport { body }));
        runner
            .catalog
            .lock()
            .expect("catalog lock is not poisoned")
            .store(
                std::time::Duration::ZERO,
                vec![crate::provider::discovery::DiscoveredModel {
                    id: ModelId::new(DEFAULT_MODEL_ID).expect("static model is valid"),
                    display_name: DEFAULT_MODEL_ID.to_string(),
                    endpoint: None,
                    privacy: crate::provider::discovery::ModelPrivacy::unknown(),
                }],
            );
        runner
    }

    fn live_request() -> ProviderRequest {
        ProviderRequest::new(
            ConversationId::new("conversation-security").expect("valid conversation"),
            ModelId::new(DEFAULT_MODEL_ID).expect("static model is valid"),
            crate::provider::contract::UserMessage::new("security check").expect("valid message"),
        )
    }

    #[test]
    fn oversized_live_response_is_rejected_offline() {
        let chunk = "data: {\"choices\":[{\"delta\":{\"content\":\"0123456789abcdef\"}}]}\n\n";
        let mut body = String::new();
        while body.len() <= contract::limits::MAX_RESPONSE_BYTES + chunk.len() {
            body.push_str(chunk);
        }
        let runner = configured_runner(Ok(body.into_bytes()));

        let error = runner
            .run_live(&live_request(), &CancellationToken::new(), &mut |_| {})
            .expect_err("oversized responses are capped");
        assert_eq!(error.code, ErrorCode::ResponseTooLarge);
    }

    #[test]
    fn transport_failure_maps_to_its_neutral_code_offline() {
        let runner = configured_runner(Err(GoFailure::from_status(401)));

        let error = runner
            .run_live(&live_request(), &CancellationToken::new(), &mut |_| {})
            .expect_err("transport failures are mapped");
        assert_eq!(error.code, ErrorCode::AuthenticationFailed);
    }

    #[test]
    fn malformed_live_frame_is_rejected_offline() {
        let runner = configured_runner(Ok(b"data: {not json}\n".to_vec()));

        let error = runner
            .run_live(&live_request(), &CancellationToken::new(), &mut |_| {})
            .expect_err("malformed frames are rejected");
        assert_eq!(error.code, ErrorCode::MalformedResponse);
    }
}
