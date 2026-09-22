use serde::{Deserialize, Serialize};

pub const PROVIDER_CONTRACT_VERSION: u32 = 1;

pub mod limits {
    pub const MAX_USER_MESSAGE_BYTES: usize = 16 * 1024;
    pub const MAX_MODEL_ID_BYTES: usize = 256;
    pub const MAX_DISPLAY_NAME_BYTES: usize = 200;
    pub const MAX_CONVERSATION_ID_BYTES: usize = 128;
    pub const MAX_STREAM_EVENT_BYTES: usize = 64 * 1024;
    pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    AuthenticationFailed,
    RateLimited,
    ProviderUnavailable,
    MalformedResponse,
    InvalidInput,
    RequestTooLarge,
    UnsupportedContractVersion,
    UnknownEvent,
    InvalidState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    UnsupportedContractVersion {
        supported: u32,
        received: u32,
    },
    TooLarge {
        field: &'static str,
        limit: usize,
        actual: usize,
    },
    Empty {
        field: &'static str,
    },
    InvalidState {
        state: RequestState,
        event: &'static str,
    },
}

impl ContractError {
    pub fn code(&self) -> ErrorCode {
        match self {
            ContractError::UnsupportedContractVersion { .. } => {
                ErrorCode::UnsupportedContractVersion
            }
            ContractError::TooLarge { .. } => ErrorCode::RequestTooLarge,
            ContractError::Empty { .. } => ErrorCode::InvalidInput,
            ContractError::InvalidState { .. } => ErrorCode::InvalidState,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ContractError::UnsupportedContractVersion { supported, received } => format!(
                "This BrainRoot build speaks contract version {supported}, but version {received} was requested."
            ),
            ContractError::TooLarge { field, limit, actual } => format!(
                "The {field} value is {actual} bytes, larger than the {limit}-byte limit."
            ),
            ContractError::Empty { field } => format!("The {field} value must not be empty."),
            ContractError::InvalidState { state, event } => format!(
                "The request cannot handle a {event} event while in the {state:?} state."
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelId(String);

impl ModelId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_text("model_id", &value, limits::MAX_MODEL_ID_BYTES)?;
        Ok(Self(value))
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        validate_text("model_id", &self.0, limits::MAX_MODEL_ID_BYTES)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConversationId(String);

impl ConversationId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_text("conversation_id", &value, limits::MAX_CONVERSATION_ID_BYTES)?;
        Ok(Self(value))
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        validate_text(
            "conversation_id",
            &self.0,
            limits::MAX_CONVERSATION_ID_BYTES,
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserMessage(String);

impl UserMessage {
    pub fn new(text: impl Into<String>) -> Result<Self, ContractError> {
        let text = text.into();
        validate_text("message", &text, limits::MAX_USER_MESSAGE_BYTES)?;
        Ok(Self(text))
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        validate_text("message", &self.0, limits::MAX_USER_MESSAGE_BYTES)
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}

fn validate_text(field: &'static str, value: &str, limit: usize) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::Empty { field });
    }
    if value.len() > limit {
        return Err(ContractError::TooLarge {
            field,
            limit,
            actual: value.len(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDescriptor {
    pub id: ModelId,
    pub display_name: String,
    pub supports_streaming: bool,
}

impl ModelDescriptor {
    pub fn new(
        id: ModelId,
        display_name: impl Into<String>,
        supports_streaming: bool,
    ) -> Result<Self, ContractError> {
        let display_name = display_name.into();
        validate_text(
            "display_name",
            &display_name,
            limits::MAX_DISPLAY_NAME_BYTES,
        )?;
        Ok(Self {
            id,
            display_name,
            supports_streaming,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRequest {
    pub contract_version: u32,
    pub conversation: ConversationId,
    pub model: ModelId,
    pub message: UserMessage,
}

impl ProviderRequest {
    pub fn new(conversation: ConversationId, model: ModelId, message: UserMessage) -> Self {
        Self {
            contract_version: PROVIDER_CONTRACT_VERSION,
            conversation,
            model,
            message,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.contract_version != PROVIDER_CONTRACT_VERSION {
            return Err(ContractError::UnsupportedContractVersion {
                supported: PROVIDER_CONTRACT_VERSION,
                received: self.contract_version,
            });
        }
        self.conversation.validate()?;
        self.model.validate()?;
        self.message.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationReason {
    UserRequested,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub conversation: ConversationId,
    pub stop_reason: StopReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cancellation {
    pub conversation: ConversationId,
    pub reason: CancellationReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Started,
    TextChunk { text: String },
    Completed { completion: Completion },
    Cancelled { cancellation: Cancellation },
    Failed { error: NormalizedError },
}

impl StreamEvent {
    pub fn kind(&self) -> &'static str {
        match self {
            StreamEvent::Started => "started",
            StreamEvent::TextChunk { .. } => "text_chunk",
            StreamEvent::Completed { .. } => "completed",
            StreamEvent::Cancelled { .. } => "cancelled",
            StreamEvent::Failed { .. } => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestState {
    Idle,
    Streaming,
    Terminal,
}

impl RequestState {
    pub fn next(self, event: &StreamEvent) -> Result<Self, ContractError> {
        match (self, event) {
            (RequestState::Idle, StreamEvent::Started) => Ok(RequestState::Streaming),
            (RequestState::Streaming, StreamEvent::TextChunk { .. }) => Ok(RequestState::Streaming),
            (
                RequestState::Streaming,
                StreamEvent::Completed { .. }
                | StreamEvent::Cancelled { .. }
                | StreamEvent::Failed { .. },
            ) => Ok(RequestState::Terminal),
            _ => Err(ContractError::InvalidState {
                state: self,
                event: event.kind(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conversation() -> ConversationId {
        ConversationId::new("conversation-1").expect("valid conversation")
    }

    fn model() -> ModelId {
        ModelId::new("model-1").expect("valid model")
    }

    fn completion() -> Completion {
        Completion {
            conversation: conversation(),
            stop_reason: StopReason::EndTurn,
        }
    }

    #[test]
    fn schema_version_is_accepted() {
        let request = ProviderRequest::new(
            conversation(),
            model(),
            UserMessage::new("Describe the change").expect("valid message"),
        );
        assert_eq!(request.contract_version, PROVIDER_CONTRACT_VERSION);
        assert_eq!(request.validate(), Ok(()));
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let mut request = ProviderRequest::new(
            conversation(),
            model(),
            UserMessage::new("Describe the change").expect("valid message"),
        );
        request.contract_version = PROVIDER_CONTRACT_VERSION + 1;

        assert_eq!(
            request.validate(),
            Err(ContractError::UnsupportedContractVersion {
                supported: PROVIDER_CONTRACT_VERSION,
                received: PROVIDER_CONTRACT_VERSION + 1,
            })
        );
    }

    #[test]
    fn unknown_stream_event_is_rejected() {
        let error = serde_json::from_str::<StreamEvent>("{\"type\":\"telemetry\"}")
            .expect_err("unknown event types must fail");
        assert!(error.to_string().contains("unknown variant"), "{error}");
    }

    #[test]
    fn missing_request_field_is_rejected() {
        let json =
            "{\"contract_version\":1,\"conversation\":\"conversation-1\",\"message\":\"hi\"}";
        serde_json::from_str::<ProviderRequest>(json).expect_err("a missing model must fail");
    }

    #[test]
    fn unknown_request_field_is_rejected() {
        let json = "{\"contract_version\":1,\"conversation\":\"conversation-1\",\"model\":\"model-1\",\"message\":\"hi\",\"extra\":true}";
        serde_json::from_str::<ProviderRequest>(json).expect_err("unknown fields must fail");
    }

    #[test]
    fn oversized_message_is_rejected() {
        let oversized = "a".repeat(limits::MAX_USER_MESSAGE_BYTES + 1);
        let error = UserMessage::new(oversized).expect_err("oversized messages must fail");
        assert_eq!(error.code(), ErrorCode::RequestTooLarge);
    }

    #[test]
    fn oversized_model_id_is_rejected() {
        let oversized = "m".repeat(limits::MAX_MODEL_ID_BYTES + 1);
        let error = ModelId::new(oversized).expect_err("oversized model IDs must fail");
        assert_eq!(error.code(), ErrorCode::RequestTooLarge);
    }

    #[test]
    fn empty_message_is_rejected() {
        let error = UserMessage::new("   ").expect_err("empty messages must fail");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
    }

    #[test]
    fn invalid_transition_from_idle_is_rejected() {
        let error = RequestState::Idle
            .next(&StreamEvent::TextChunk {
                text: "hello".to_string(),
            })
            .expect_err("text before started must fail");
        assert_eq!(error.code(), ErrorCode::InvalidState);
    }

    #[test]
    fn invalid_transition_from_terminal_is_rejected() {
        let error = RequestState::Terminal
            .next(&StreamEvent::Completed {
                completion: completion(),
            })
            .expect_err("a duplicate terminal event must fail");
        assert_eq!(error.code(), ErrorCode::InvalidState);
    }

    #[test]
    fn stream_progress_maps_to_terminal_state() {
        let mut state = RequestState::Idle;
        state = state.next(&StreamEvent::Started).expect("started");
        assert_eq!(state, RequestState::Streaming);
        state = state
            .next(&StreamEvent::TextChunk {
                text: "hello".to_string(),
            })
            .expect("chunk");
        assert_eq!(state, RequestState::Streaming);
        state = state
            .next(&StreamEvent::Completed {
                completion: completion(),
            })
            .expect("completed");
        assert_eq!(state, RequestState::Terminal);
    }

    #[test]
    fn stream_event_round_trip() {
        let event = StreamEvent::TextChunk {
            text: "hello".to_string(),
        };
        let json = serde_json::to_string(&event).expect("serializes");
        let decoded = serde_json::from_str::<StreamEvent>(&json).expect("deserializes");
        assert_eq!(decoded, event);
    }

    #[test]
    fn normalized_error_maps_contract_error() {
        let error = ContractError::TooLarge {
            field: "message",
            limit: limits::MAX_USER_MESSAGE_BYTES,
            actual: limits::MAX_USER_MESSAGE_BYTES + 1,
        };
        let normalized = NormalizedError {
            code: error.code(),
            message: error.message(),
        };
        assert_eq!(normalized.code, ErrorCode::RequestTooLarge);
        assert!(normalized.message.contains("limit"));

        let json = serde_json::to_string(&normalized).expect("serializes");
        assert!(json.contains("\"code\":\"request_too_large\""));
    }

    #[test]
    fn descriptor_requires_a_display_name() {
        let error = ModelDescriptor::new(model(), "  ", true).expect_err("empty names must fail");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
    }
}
