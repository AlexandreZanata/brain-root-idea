//! One validated OpenCode Go protocol path: chat/completions.
//!
//! Configuration is validated data, never presentation code: the endpoint must
//! be the documented `https://opencode.ai/zen/go/v1/chat/completions`, the
//! model comes from the contract, `x-opencode-session` is a random nonsecret
//! UUID per conversation, and the SSE stream is parsed into neutral events.
//! Status and transport failures are classified by [`GoFailure`]. There is no
//! fallback to another model, protocol, endpoint, or balance.

use std::io::Read;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::contract::{
    Completion, ConversationId, ErrorCode, ModelId, NormalizedError, StopReason, StreamEvent,
    UserMessage,
};
use super::credential::{AuthorizationHeader, CredentialValue};
use super::discovery::BRAINROOT_USER_AGENT;
use super::failure::GoFailure;

pub const GO_CHAT_COMPLETIONS_ENDPOINT: &str = "https://opencode.ai/zen/go/v1/chat/completions";
pub const DEFAULT_MODEL_ID: &str = "glm-5.3-flash";
pub const SESSION_HEADER: &str = "x-opencode-session";
pub const MAX_STREAM_LINE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoConfigError {
    UnsupportedEndpoint { endpoint: String },
    EmptySession,
}

impl GoConfigError {
    pub fn code(&self) -> ErrorCode {
        ErrorCode::InvalidInput
    }

    pub fn message(&self) -> String {
        match self {
            GoConfigError::UnsupportedEndpoint { endpoint } => format!(
                "The provider endpoint {endpoint} is not the supported OpenCode Go chat endpoint."
            ),
            GoConfigError::EmptySession => {
                "The conversation session identifier must not be empty.".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoConfig {
    endpoint: String,
    model: ModelId,
}

impl GoConfig {
    pub fn new(endpoint: impl Into<String>, model: ModelId) -> Result<Self, GoConfigError> {
        let endpoint = endpoint.into();
        if endpoint != GO_CHAT_COMPLETIONS_ENDPOINT {
            return Err(GoConfigError::UnsupportedEndpoint { endpoint });
        }
        Ok(Self { endpoint, model })
    }

    pub fn default_go() -> Self {
        Self {
            endpoint: GO_CHAT_COMPLETIONS_ENDPOINT.to_string(),
            model: ModelId::new(DEFAULT_MODEL_ID).expect("static default model is valid"),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn model(&self) -> &ModelId {
        &self.model
    }

    pub fn build_request_body(&self, message: &UserMessage) -> Result<String, GoFailure> {
        let request = ChatRequest {
            model: self.model.as_str(),
            messages: vec![ChatMessage {
                role: "user",
                content: message.text(),
            }],
            stream: true,
        };
        serde_json::to_string(&request).map_err(|_| GoFailure::MalformedResponse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionId(String);

impl SessionId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from_value(value: impl Into<String>) -> Result<Self, GoConfigError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(GoConfigError::EmptySession);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn headers(credential: &CredentialValue, session: &SessionId) -> Vec<(String, String)> {
    vec![
        (
            "Authorization".to_string(),
            AuthorizationHeader::bearer(credential)
                .expose_to_core()
                .to_string(),
        ),
        ("User-Agent".to_string(), BRAINROOT_USER_AGENT.to_string()),
        (SESSION_HEADER.to_string(), session.as_str().to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
    ]
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

pub trait GoTransport {
    fn post(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &str,
    ) -> Result<Box<dyn Read + Send>, GoFailure>;
}

/// The live Go transport. Unbounded by default; [`UreqGoTransport::with_timeout`]
/// bounds the entire call, including reading the response body.
pub struct UreqGoTransport {
    timeout: Option<Duration>,
}

impl UreqGoTransport {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
        }
    }

    fn agent(&self) -> ureq::Agent {
        let mut config = ureq::Agent::config_builder();
        if let Some(timeout) = self.timeout {
            config = config.timeout_global(Some(timeout));
        }
        ureq::Agent::new_with_config(config.build())
    }
}

impl Default for UreqGoTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl GoTransport for UreqGoTransport {
    fn post(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &str,
    ) -> Result<Box<dyn Read + Send>, GoFailure> {
        let mut request = self
            .agent()
            .post(url)
            .header("User-Agent", BRAINROOT_USER_AGENT);
        for (name, value) in headers {
            request = request.header(name.as_str(), value.as_str());
        }

        match request.send(body) {
            Ok(response) => Ok(Box::new(response.into_body().into_reader())),
            Err(ureq::Error::StatusCode(status)) => Err(GoFailure::from_status(status)),
            Err(ureq::Error::Timeout(_)) => Err(GoFailure::Timeout),
            Err(_) => Err(GoFailure::NetworkUnavailable),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    error: Option<StreamErrorPayload>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamErrorPayload {
    #[serde(default)]
    #[allow(dead_code)]
    message: Option<String>,
}

pub struct ChatStreamParser {
    conversation: ConversationId,
    buffer: Vec<u8>,
    started: bool,
    finished: bool,
}

impl ChatStreamParser {
    pub fn new(conversation: ConversationId) -> Self {
        Self {
            conversation,
            buffer: Vec::new(),
            started: false,
            finished: false,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Feeds raw bytes; complete `data:` lines become neutral events.
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<StreamEvent>, GoFailure> {
        if self.finished {
            return Ok(Vec::new());
        }
        if self.buffer.len() + chunk.len() > MAX_STREAM_LINE_BYTES {
            return Err(GoFailure::MalformedResponse);
        }
        self.buffer.extend_from_slice(chunk);

        let mut events = Vec::new();
        while let Some(position) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=position).collect();
            let line = String::from_utf8_lossy(&line[..line.len() - 1])
                .trim()
                .to_string();
            self.process_line(&line, &mut events)?;
            if self.finished {
                break;
            }
        }
        Ok(events)
    }

    /// Completes the request when the stream ends without `[DONE]`.
    pub fn finish(&mut self) -> Vec<StreamEvent> {
        if self.finished {
            return Vec::new();
        }
        let mut events = Vec::new();
        if !self.buffer.is_empty() {
            let line = String::from_utf8_lossy(&self.buffer).trim().to_string();
            self.buffer.clear();
            let _ = self.process_line(&line, &mut events);
        }
        self.complete(&mut events);
        events
    }

    fn process_line(&mut self, line: &str, events: &mut Vec<StreamEvent>) -> Result<(), GoFailure> {
        if self.finished || line.is_empty() || line.starts_with(':') {
            return Ok(());
        }
        let Some(payload) = line.strip_prefix("data:") else {
            return Ok(());
        };
        let payload = payload.trim();
        if payload == "[DONE]" {
            self.complete(events);
            return Ok(());
        }

        let chunk: StreamChunk =
            serde_json::from_str(payload).map_err(|_| GoFailure::MalformedResponse)?;
        if !self.started {
            self.started = true;
            events.push(StreamEvent::Started);
        }
        if chunk.error.is_some() {
            self.finished = true;
            events.push(StreamEvent::Failed {
                error: NormalizedError {
                    code: ErrorCode::ProviderUnavailable,
                    message: "The provider reported an error while streaming.".to_string(),
                },
            });
            return Ok(());
        }

        let content: String = chunk
            .choices
            .iter()
            .filter_map(|choice| choice.delta.as_ref())
            .filter_map(|delta| delta.content.as_deref())
            .collect();
        if !content.is_empty() {
            events.push(StreamEvent::TextChunk { text: content });
        }
        Ok(())
    }

    fn complete(&mut self, events: &mut Vec<StreamEvent>) {
        if self.finished {
            return;
        }
        self.finished = true;
        events.push(StreamEvent::Completed {
            completion: Completion {
                conversation: self.conversation.clone(),
                stop_reason: StopReason::EndTurn,
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FAKE_SECRET: &str = "fake-secret-value";

    fn conversation() -> ConversationId {
        ConversationId::new("conversation-1").expect("valid conversation")
    }

    fn credential() -> CredentialValue {
        CredentialValue::new(FAKE_SECRET).expect("valid credential")
    }

    fn session() -> SessionId {
        SessionId::from_value("session-fixed-for-tests").expect("valid session")
    }

    fn message() -> UserMessage {
        UserMessage::new("Describe a small change").expect("valid message")
    }

    fn kinds(events: &[StreamEvent]) -> Vec<&'static str> {
        events
            .iter()
            .map(|event| match event {
                StreamEvent::Started => "started",
                StreamEvent::TextChunk { .. } => "text_chunk",
                StreamEvent::Completed { .. } => "completed",
                StreamEvent::Cancelled { .. } => "cancelled",
                StreamEvent::Failed { .. } => "failed",
            })
            .collect()
    }

    #[test]
    fn default_config_uses_the_documented_endpoint_and_model() {
        let config = GoConfig::default_go();

        assert_eq!(
            config.endpoint(),
            "https://opencode.ai/zen/go/v1/chat/completions"
        );
        assert_eq!(config.model().as_str(), DEFAULT_MODEL_ID);
    }

    #[test]
    fn endpoint_validation_rejects_unsupported_values() {
        for endpoint in [
            "http://opencode.ai/zen/go/v1/chat/completions",
            "https://example.com/v1/chat/completions",
            "https://opencode.ai/zen/go/v1/messages",
            "https://opencode.ai/zen/go/v1/responses",
        ] {
            let error = GoConfig::new(endpoint, ModelId::new("m").expect("valid"))
                .expect_err("unsupported endpoint");
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            assert!(!error.message().is_empty());
        }
    }

    #[test]
    fn model_override_is_accepted() {
        let config = GoConfig::new(
            GO_CHAT_COMPLETIONS_ENDPOINT,
            ModelId::new("glm-5.3").expect("valid"),
        )
        .expect("supported endpoint");

        assert_eq!(config.model().as_str(), "glm-5.3");
    }

    #[test]
    fn session_ids_are_unique_and_nonsecret() {
        let first = SessionId::generate();
        let second = SessionId::generate();

        assert_ne!(first, second);
        assert!(!first.as_str().is_empty());
        assert!(!first.as_str().contains(FAKE_SECRET));
        assert!(SessionId::from_value("  ").is_err());
    }

    #[test]
    fn headers_carry_authorization_user_agent_and_session_once() {
        let headers = headers(&credential(), &session());

        let count = |name: &str| headers.iter().filter(|(key, _)| key == name).count();
        assert_eq!(count("Authorization"), 1);
        assert_eq!(count("User-Agent"), 1);
        assert_eq!(count(SESSION_HEADER), 1);
        assert_eq!(count("Content-Type"), 1);

        let authorization = headers
            .iter()
            .find(|(name, _)| name == "Authorization")
            .map(|(_, value)| value)
            .expect("authorization");
        assert_eq!(authorization, &format!("Bearer {FAKE_SECRET}"));
        let user_agent = headers
            .iter()
            .find(|(name, _)| name == "User-Agent")
            .map(|(_, value)| value)
            .expect("user agent");
        assert_eq!(user_agent, BRAINROOT_USER_AGENT);
    }

    #[test]
    fn request_body_has_the_documented_shape() {
        let body = GoConfig::default_go()
            .build_request_body(&message())
            .expect("body builds");
        let value: Value = serde_json::from_str(&body).expect("valid json");

        assert_eq!(value["model"], DEFAULT_MODEL_ID);
        assert_eq!(value["stream"], true);
        assert_eq!(value["messages"][0]["role"], "user");
        assert_eq!(value["messages"][0]["content"], "Describe a small change");
        assert!(!body.contains(FAKE_SECRET));
    }

    #[test]
    fn parser_reassembles_split_chunks_and_utf8() {
        let frame = "data: {\"choices\":[{\"delta\":{\"content\":\"café\"}}]}\n";
        let mut parser = ChatStreamParser::new(conversation());
        let mut events = Vec::new();
        for byte in frame.as_bytes() {
            events.extend(
                parser
                    .push(std::slice::from_ref(byte))
                    .expect("byte accepted"),
            );
        }

        assert_eq!(kinds(&events), vec!["started", "text_chunk"]);
        assert!(matches!(&events[1], StreamEvent::TextChunk { text } if text == "café"));
    }

    #[test]
    fn parser_maps_delta_to_text_and_done_to_completed() {
        let stream = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\", \"}}]}\n",
            "data: [DONE]\n"
        );
        let mut parser = ChatStreamParser::new(conversation());
        let events = parser.push(stream.as_bytes()).expect("stream parses");

        assert_eq!(
            kinds(&events),
            vec!["started", "text_chunk", "text_chunk", "completed"]
        );
        assert!(parser.is_finished());
    }

    #[test]
    fn parser_ignores_comments_blank_lines_and_unknown_fields() {
        let stream = concat!(
            ": keep-alive\n",
            "\n",
            "event: message\n",
            "data: {\"unknown\":true,\"choices\":[{\"delta\":{\"content\":\"hi\"},\"extra\":1}]}\n",
            "data: [DONE]\n"
        );
        let mut parser = ChatStreamParser::new(conversation());
        let events = parser.push(stream.as_bytes()).expect("stream parses");

        assert_eq!(kinds(&events), vec!["started", "text_chunk", "completed"]);
    }

    #[test]
    fn parser_maps_provider_error_to_failed() {
        let stream = "data: {\"error\":{\"message\":\"provider blew up\"}}\n";
        let mut parser = ChatStreamParser::new(conversation());
        let events = parser.push(stream.as_bytes()).expect("stream parses");

        assert_eq!(kinds(&events), vec!["started", "failed"]);
        let StreamEvent::Failed { error } = &events[1] else {
            panic!("expected failed");
        };
        assert_eq!(error.code, ErrorCode::ProviderUnavailable);
        assert!(!error.message.contains("blew up"));
    }

    #[test]
    fn parser_rejects_malformed_json() {
        let stream = "data: {not json}\n";
        let mut parser = ChatStreamParser::new(conversation());

        assert_eq!(
            parser.push(stream.as_bytes()),
            Err(GoFailure::MalformedResponse)
        );
    }

    #[test]
    fn parser_ignores_events_after_the_terminal() {
        let stream = concat!(
            "data: [DONE]\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"late\"}}]}\n"
        );
        let mut parser = ChatStreamParser::new(conversation());
        let events = parser.push(stream.as_bytes()).expect("stream parses");

        assert_eq!(kinds(&events), vec!["completed"]);
        assert!(parser.push(b"data: [DONE]\n").expect("no error").is_empty());
    }

    #[test]
    fn finish_completes_an_unterminated_stream() {
        let stream = "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n";
        let mut parser = ChatStreamParser::new(conversation());
        let events = parser.push(stream.as_bytes()).expect("stream parses");
        assert_eq!(kinds(&events), vec!["started", "text_chunk"]);

        let finished = parser.finish();
        assert_eq!(kinds(&finished), vec!["completed"]);
        assert!(parser.finish().is_empty());
    }
}
