//! OpenCode Go model discovery.
//!
//! Fetches the documented models endpoint through an injectable transport,
//! keeps only the protocol families the contract records, and caches nonsecret
//! metadata with an explicit timestamp/expiry. No clock is read here: callers
//! pass `at`, mirroring the deterministic architecture of the provider core.
//! Failure conditions go through the shared [`GoFailure`] classification.
//!
//! Per-model privacy disclosure is captured only when the payload states it,
//! bounded by [`MAX_PRIVACY_DISCLOSURE_BYTES`], and stays explicitly unknown
//! otherwise.
//!
//! The live response shape is verified by the opt-in smoke test (B03-S06);
//! this module parses a minimal documented shape tolerantly and ignores
//! unknown fields, so a richer payload does not break discovery.

use std::time::Duration;

use super::contract::ModelId;
use super::credential::{AuthorizationHeader, CredentialValue};
use super::failure::GoFailure;

pub const MODELS_URL: &str = "https://opencode.ai/zen/go/v1/models";
pub const BRAINROOT_USER_AGENT: &str = concat!("brainroot/", env!("CARGO_PKG_VERSION"));
pub const CACHE_TTL: Duration = Duration::from_secs(15 * 60);
pub const MAX_PRIVACY_DISCLOSURE_BYTES: usize = 200;

/// Protocol families the OpenCode Go contract records. The mapping from a
/// model to one of them is external and mutable; discovery only classifies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolEndpoint {
    Responses,
    ChatCompletions,
    Messages,
}

impl ProtocolEndpoint {
    pub fn from_url(url: &str) -> Option<Self> {
        let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
        if path.ends_with("/responses") {
            Some(Self::Responses)
        } else if path.ends_with("/chat/completions") {
            Some(Self::ChatCompletions)
        } else if path.ends_with("/messages") {
            Some(Self::Messages)
        } else {
            None
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::ChatCompletions => "chat_completions",
            Self::Messages => "messages",
        }
    }
}

/// Per-model privacy policy stated by the models payload. Values are the
/// provider's own bounded text; a missing, mistyped, empty, or oversized field
/// stays [`PrivacyDisclosure::Unknown`] instead of being guessed or truncated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivacyDisclosure {
    Stated(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPrivacy {
    pub training: PrivacyDisclosure,
    pub retention: PrivacyDisclosure,
}

impl ModelPrivacy {
    pub fn unknown() -> Self {
        Self {
            training: PrivacyDisclosure::Unknown,
            retention: PrivacyDisclosure::Unknown,
        }
    }

    fn from_entry(object: &serde_json::Map<String, serde_json::Value>) -> Self {
        let Some(privacy) = object.get("privacy").and_then(|value| value.as_object()) else {
            return Self::unknown();
        };
        Self {
            training: disclosure(privacy.get("training")),
            retention: disclosure(privacy.get("retention")),
        }
    }
}

fn disclosure(value: Option<&serde_json::Value>) -> PrivacyDisclosure {
    let Some(text) = value.and_then(|value| value.as_str()) else {
        return PrivacyDisclosure::Unknown;
    };
    let text = text.trim();
    if text.is_empty() || text.len() > MAX_PRIVACY_DISCLOSURE_BYTES {
        return PrivacyDisclosure::Unknown;
    }
    PrivacyDisclosure::Stated(text.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredModel {
    pub id: ModelId,
    pub display_name: String,
    pub endpoint: ProtocolEndpoint,
    pub privacy: ModelPrivacy,
}

/// Minimal HTTP surface discovery needs. The real implementation is
/// [`UreqTransport`]; tests supply a fake so they never touch the network.
pub trait HttpTransport {
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<String, GoFailure>;
}

/// The live discovery transport. Unbounded by default; [`UreqTransport::with_timeout`]
/// bounds the entire call, including reading the response body.
pub struct UreqTransport {
    timeout: Option<Duration>,
}

impl UreqTransport {
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

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpTransport for UreqTransport {
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<String, GoFailure> {
        let mut request = self
            .agent()
            .get(url)
            .header("User-Agent", BRAINROOT_USER_AGENT);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }

        match request.call() {
            Ok(mut response) => response
                .body_mut()
                .read_to_string()
                .map_err(|_| GoFailure::MalformedResponse),
            Err(ureq::Error::StatusCode(status)) => Err(GoFailure::from_status(status)),
            Err(ureq::Error::Timeout(_)) => Err(GoFailure::Timeout),
            Err(_) => Err(GoFailure::NetworkUnavailable),
        }
    }
}

pub struct DiscoveryClient<T: HttpTransport> {
    transport: T,
}

impl<T: HttpTransport> DiscoveryClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Fetches the models endpoint with the credential; the header value is
    /// built internally and never logged.
    pub fn fetch(&self, credential: &CredentialValue) -> Result<Vec<DiscoveredModel>, GoFailure> {
        let authorization = AuthorizationHeader::bearer(credential);
        let body = self.transport.get(
            MODELS_URL,
            &[("Authorization", authorization.expose_to_core())],
        )?;
        parse_models(&body)
    }
}

fn parse_models(body: &str) -> Result<Vec<DiscoveredModel>, GoFailure> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| GoFailure::MalformedResponse)?;

    let entries = match value {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::Object(mut object) => {
            let data = object.remove("data").ok_or(GoFailure::MalformedResponse)?;
            data.as_array()
                .cloned()
                .ok_or(GoFailure::MalformedResponse)?
        }
        _ => return Err(GoFailure::MalformedResponse),
    };

    let mut models = Vec::new();
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let Some(id_value) = object.get("id").and_then(|id| id.as_str()) else {
            continue;
        };
        let Some(endpoint_value) = object.get("endpoint").and_then(|url| url.as_str()) else {
            continue;
        };
        let Some(endpoint) = ProtocolEndpoint::from_url(endpoint_value) else {
            continue;
        };
        let Ok(id) = ModelId::new(id_value) else {
            continue;
        };
        let display_name = object
            .get("name")
            .and_then(|name| name.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| id.as_str().to_string());
        models.push(DiscoveredModel {
            id,
            display_name,
            endpoint,
            privacy: ModelPrivacy::from_entry(object),
        });
    }
    Ok(models)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogState {
    Fresh(Vec<DiscoveredModel>),
    Stale(Vec<DiscoveredModel>),
    Empty,
}

#[derive(Debug, Default)]
pub struct ModelCache {
    stored_at: Option<Duration>,
    models: Vec<DiscoveredModel>,
}

impl ModelCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores nonsecret metadata with the moment it was fetched.
    pub fn store(&mut self, at: Duration, models: Vec<DiscoveredModel>) {
        self.stored_at = Some(at);
        self.models = models;
    }

    pub fn clear(&mut self) {
        self.stored_at = None;
        self.models.clear();
    }

    pub fn state(&self, at: Duration) -> CatalogState {
        let Some(stored_at) = self.stored_at else {
            return CatalogState::Empty;
        };
        if self.models.is_empty() {
            return CatalogState::Empty;
        }
        let age = at.saturating_sub(stored_at);
        if age <= CACHE_TTL {
            CatalogState::Fresh(self.models.clone())
        } else {
            CatalogState::Stale(self.models.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::contract::ErrorCode;
    use std::cell::RefCell;

    const FIXTURE: &str = r#"{
        "data": [
            {
                "id": "glm-5.3",
                "name": "GLM 5.3",
                "endpoint": "https://opencode.ai/zen/go/v1/chat/completions",
                "context": 200000,
                "limit": { "output": 65536 },
                "vendor": { "family": "glm" },
                "privacy": { "training": "Not used", "retention": "0 days" }
            },
            {
                "id": "minimax-m3",
                "endpoint": "https://opencode.ai/zen/go/v1/messages"
            },
            {
                "id": "gpt-5.6-luna",
                "name": "GPT 5.6 Luna",
                "endpoint": "https://opencode.ai/zen/go/v1/responses"
            },
            {
                "id": "unsupported",
                "name": "Unsupported",
                "endpoint": "https://example.com/v2/complete"
            }
        ]
    }"#;

    type RecordedRequest = (String, Vec<(String, String)>);

    struct FakeTransport {
        response: Result<String, GoFailure>,
        requests: RefCell<Vec<RecordedRequest>>,
    }

    impl FakeTransport {
        fn returning(body: &str) -> Self {
            Self {
                response: Ok(body.to_string()),
                requests: RefCell::new(Vec::new()),
            }
        }

        fn failing(error: GoFailure) -> Self {
            Self {
                response: Err(error),
                requests: RefCell::new(Vec::new()),
            }
        }
    }

    impl HttpTransport for FakeTransport {
        fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<String, GoFailure> {
            self.requests.borrow_mut().push((
                url.to_string(),
                headers
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.to_string()))
                    .collect(),
            ));
            self.response.clone()
        }
    }

    fn credential() -> CredentialValue {
        CredentialValue::new("fake-secret-value").expect("valid credential")
    }

    #[test]
    fn parses_models_and_ignores_unknown_fields() {
        let client = DiscoveryClient::new(FakeTransport::returning(FIXTURE));
        let models = client.fetch(&credential()).expect("fixture parses");

        assert_eq!(models.len(), 3);
        assert_eq!(models[0].id.as_str(), "glm-5.3");
        assert_eq!(models[0].display_name, "GLM 5.3");
        assert_eq!(models[0].endpoint, ProtocolEndpoint::ChatCompletions);
        assert_eq!(
            models[0].privacy,
            ModelPrivacy {
                training: PrivacyDisclosure::Stated("Not used".to_string()),
                retention: PrivacyDisclosure::Stated("0 days".to_string()),
            }
        );
    }

    #[test]
    fn filters_unknown_endpoints_and_falls_back_to_id() {
        let client = DiscoveryClient::new(FakeTransport::returning(FIXTURE));
        let models = client.fetch(&credential()).expect("fixture parses");

        assert!(models
            .iter()
            .all(|model| model.id.as_str() != "unsupported"));
        let minimax = models
            .iter()
            .find(|model| model.id.as_str() == "minimax-m3")
            .expect("minimax present");
        assert_eq!(minimax.display_name, "minimax-m3");
        assert_eq!(minimax.endpoint, ProtocolEndpoint::Messages);
        assert_eq!(minimax.privacy, ModelPrivacy::unknown());
    }

    #[test]
    fn absent_or_partial_privacy_stays_explicitly_unknown() {
        let body = r#"{"data":[
            {"id":"a","endpoint":"https://opencode.ai/zen/go/v1/chat/completions"},
            {"id":"b","endpoint":"https://opencode.ai/zen/go/v1/chat/completions",
             "privacy":{"training":"Not used"}}
        ]}"#;
        let client = DiscoveryClient::new(FakeTransport::returning(body));
        let models = client.fetch(&credential()).expect("fixture parses");

        assert_eq!(models[0].privacy, ModelPrivacy::unknown());
        assert_eq!(
            models[1].privacy.training,
            PrivacyDisclosure::Stated("Not used".to_string())
        );
        assert_eq!(models[1].privacy.retention, PrivacyDisclosure::Unknown);
    }

    #[test]
    fn mistyped_empty_or_oversized_privacy_is_unknown() {
        let oversized = "p".repeat(MAX_PRIVACY_DISCLOSURE_BYTES + 1);
        let body = format!(
            r#"{{"data":[
                {{"id":"a","endpoint":"https://opencode.ai/zen/go/v1/chat/completions",
                  "privacy":{{"training":7,"retention":"  ","extra":true}}}},
                {{"id":"b","endpoint":"https://opencode.ai/zen/go/v1/chat/completions",
                  "privacy":{{"training":"{oversized}"}}}},
                {{"id":"c","endpoint":"https://opencode.ai/zen/go/v1/chat/completions",
                  "privacy":"not an object"}}
            ]}}"#
        );
        let client = DiscoveryClient::new(FakeTransport::returning(&body));
        let models = client.fetch(&credential()).expect("fixture parses");

        assert!(models
            .iter()
            .all(|model| model.privacy == ModelPrivacy::unknown()));
    }

    #[test]
    fn accepts_a_bare_array_shape() {
        let body =
            r#"[{"id":"glm-5.3","endpoint":"https://opencode.ai/zen/go/v1/chat/completions"}]"#;
        let client = DiscoveryClient::new(FakeTransport::returning(body));
        let models = client.fetch(&credential()).expect("array parses");

        assert_eq!(models.len(), 1);
    }

    #[test]
    fn unauthorized_maps_to_a_typed_error() {
        let client = DiscoveryClient::new(FakeTransport::failing(GoFailure::from_status(401)));
        let error = client.fetch(&credential()).expect_err("unauthorized");

        assert_eq!(error.code(), ErrorCode::AuthenticationFailed);
        assert!(error.message().contains("rejected the credential"));
    }

    #[test]
    fn rate_limit_timeout_and_network_failures_stay_distinct() {
        let limited = DiscoveryClient::new(FakeTransport::failing(GoFailure::from_status(429)));
        let error = limited.fetch(&credential()).expect_err("rate limited");
        assert_eq!(error.code(), ErrorCode::RateLimited);
        assert!(error.message().contains("usage limit"));

        let timed_out = DiscoveryClient::new(FakeTransport::failing(GoFailure::Timeout));
        assert_eq!(
            timed_out.fetch(&credential()).expect_err("timeout").code(),
            ErrorCode::TimedOut
        );

        let offline = DiscoveryClient::new(FakeTransport::failing(GoFailure::NetworkUnavailable));
        let error = offline.fetch(&credential()).expect_err("offline");
        assert_eq!(error.code(), ErrorCode::ProviderUnavailable);
        assert!(error.message().contains("could not be reached"));
    }

    #[test]
    fn malformed_json_is_rejected() {
        let client = DiscoveryClient::new(FakeTransport::returning("not json"));
        assert_eq!(
            client.fetch(&credential()),
            Err(GoFailure::MalformedResponse)
        );
    }

    #[test]
    fn cache_is_empty_before_store() {
        let cache = ModelCache::new();

        assert_eq!(cache.state(Duration::ZERO), CatalogState::Empty);
    }

    #[test]
    fn cache_is_fresh_within_ttl_and_stale_after() {
        let client = DiscoveryClient::new(FakeTransport::returning(FIXTURE));
        let models = client.fetch(&credential()).expect("fixture parses");
        let mut cache = ModelCache::new();
        cache.store(Duration::from_secs(100), models);

        assert!(matches!(
            cache.state(Duration::from_secs(100) + CACHE_TTL),
            CatalogState::Fresh(_)
        ));
        assert!(matches!(
            cache.state(Duration::from_secs(100) + CACHE_TTL + Duration::from_secs(1)),
            CatalogState::Stale(_)
        ));
    }

    #[test]
    fn cache_handles_backwards_time_as_fresh() {
        let client = DiscoveryClient::new(FakeTransport::returning(FIXTURE));
        let models = client.fetch(&credential()).expect("fixture parses");
        let mut cache = ModelCache::new();
        cache.store(Duration::from_secs(100), models);

        assert!(matches!(
            cache.state(Duration::from_secs(90)),
            CatalogState::Fresh(_)
        ));
    }

    #[test]
    fn clearing_returns_to_empty() {
        let client = DiscoveryClient::new(FakeTransport::returning(FIXTURE));
        let models = client.fetch(&credential()).expect("fixture parses");
        let mut cache = ModelCache::new();
        cache.store(Duration::ZERO, models);
        cache.clear();

        assert_eq!(cache.state(Duration::ZERO), CatalogState::Empty);
    }

    #[test]
    fn transport_receives_the_bearer_header_and_user_agent_is_constant() {
        let client = DiscoveryClient::new(FakeTransport::returning(FIXTURE));
        let _ = client.fetch(&credential()).expect("fixture parses");

        let requests = client.transport.requests.borrow().clone();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, MODELS_URL);

        let authorization = requests[0]
            .1
            .iter()
            .find(|(name, _)| name == "Authorization")
            .expect("authorization header present");
        assert!(authorization.1.starts_with("Bearer "));
        assert!(BRAINROOT_USER_AGENT.starts_with("brainroot/"));
    }
}
