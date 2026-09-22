//! Provider runners for the conversation feature.

use std::io::Read;
use std::sync::Mutex;
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
    catalog: Mutex<ModelCache>,
    started_at: Instant,
}

impl LiveGoRunner {
    pub(super) fn new(provider: ProviderState) -> Self {
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
