//! Agent Host: owns the `opencode serve` sidecar process.
//!
//! R2-S01 vertical slice. Single-flight sidecar with an ephemeral basic-auth
//! password held only in memory: `start` spawns, polls `/global/health` with
//! auth, `status` reports redacted state, `stop`/`shutdown` terminate the
//! child. The password never crosses IPC, logs, or `Debug` output.
//!
//! `features::conversation` is frozen legacy in this batch: untouched.

mod catalog;

use std::io::{BufRead, Read};
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::provider::contract::{limits, ErrorCode, NormalizedError};

/// Default localhost port for the sidecar. Single session in R2-S01.
pub const AGENT_HOST_DEFAULT_PORT: u16 = 4099;
/// Upper bound on models forwarded to the UI. The catalog is untrusted input.
const MAX_MODELS: usize = 500;
const MAX_NAME_CHARS: usize = 200;
/// Product event name for sidecar session streams. Frontend filters by session.
pub const AGENT_EVENT_NAME: &str = "agent_event";
pub const AGENT_CONTRACT_VERSION: u32 = 1;
/// Bounds one streamed text delta and one full send. The sidecar is untrusted.
const MAX_DELTA_CHARS: usize = 64 * 1024;
const MAX_SEND_CHARS: usize = 2 * 1024 * 1024;
/// Stream calls may run long generations; cancel arrives via flag + abort.
const STREAM_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// How long `start` waits for `/global/health` before giving up.
const READY_TIMEOUT: Duration = Duration::from_secs(10);
const READY_POLL_INTERVAL: Duration = Duration::from_millis(200);
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_HEALTH_BODY_BYTES: u64 = 64 * 1024;
const MAX_VERSION_CHARS: usize = 64;
/// How long `stop` waits for the child to exit after `kill` before forcing.
const KILL_GRACE: Duration = Duration::from_secs(5);

/// Redacted, frontend-safe sidecar state. No secret field exists by design.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHostStatus {
    pub running: bool,
    pub port: u16,
    pub version: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Deserialize)]
struct HealthBody {
    healthy: bool,
    version: String,
}

struct Running {
    child: std::process::Child,
    password: String,
    port: u16,
    version: String,
    last_used: Instant,
}

impl std::fmt::Debug for Running {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Running")
            .field("child", &self.child)
            .field("password", &"<redacted>")
            .field("port", &self.port)
            .field("version", &self.version)
            .field("last_used", &self.last_used)
            .finish()
    }
}

/// One in-flight sidecar turn. Single-flight: a second send is rejected.
#[derive(Debug)]
struct ActiveSend {
    session_id: String,
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
}

/// Frontend-facing turn lifecycle. Versioned envelope below.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentStreamEvent {
    Started {
        session: String,
    },
    TextChunk {
        session: String,
        text: String,
    },
    Completed {
        session: String,
    },
    Failed {
        session: String,
        error: NormalizedError,
    },
    Cancelled {
        session: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentEventEnvelope {
    pub contract_version: u32,
    pub event: AgentStreamEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentSendAccepted {
    pub contract_version: u32,
    pub session: String,
}

/// Tauri-managed sidecar owner. Everything behind mutexes so concurrent
/// commands cannot double-spawn or interleave selection writes.
#[derive(Debug, Default)]
pub struct AgentHostState {
    inner: Mutex<Option<Running>>,
    selection: Mutex<Option<AgentModelSelection>>,
    active: Mutex<Option<ActiveSend>>,
    catalog: Mutex<CachedCatalog>,
}

#[derive(Debug, Default)]
struct CachedCatalog {
    fetched_at: Option<Instant>,
    entries: Vec<catalog::OpenRouterEntry>,
}

/// One selectable model. Only identifiers cross into the UI: providers,
/// prices, and any credential-adjacent field never leave the sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentModelSelection {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentModelEntry {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub model_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentModelList {
    pub models: Vec<AgentModelEntry>,
    pub selected: Option<AgentModelSelection>,
}

fn host_error(code: ErrorCode, message: impl Into<String>) -> NormalizedError {
    NormalizedError {
        code,
        message: message.into(),
    }
}

fn validate_port(port: u16) -> Result<u16, NormalizedError> {
    if port == 0 {
        return Err(host_error(
            ErrorCode::InvalidInput,
            "The sidecar port must be a fixed nonzero port in R2-S01.",
        ));
    }
    Ok(port)
}

fn error_for_status(status: u16) -> NormalizedError {
    match status {
        401 => host_error(
            ErrorCode::AuthenticationFailed,
            "The sidecar rejected the session credential. Restart BrainRoot and try again.",
        ),
        429 => host_error(
            ErrorCode::RateLimited,
            "The sidecar reported a rate limit. Wait and try again.",
        ),
        _ => host_error(
            ErrorCode::ProviderUnavailable,
            "The sidecar answered with an unexpected status. Restart it and try again.",
        ),
    }
}

fn parse_health(body: &str) -> Result<String, NormalizedError> {
    let parsed: HealthBody = serde_json::from_str(body).map_err(|_| {
        host_error(
            ErrorCode::MalformedResponse,
            "The sidecar answered with malformed health data.",
        )
    })?;
    if !parsed.healthy {
        return Err(host_error(
            ErrorCode::ProviderUnavailable,
            "The sidecar reported itself unhealthy.",
        ));
    }
    let mut version = parsed.version;
    if version.chars().count() > MAX_VERSION_CHARS {
        version = version.chars().take(MAX_VERSION_CHARS).collect();
    }
    if version.trim().is_empty() {
        return Err(host_error(
            ErrorCode::MalformedResponse,
            "The sidecar answered without a version.",
        ));
    }
    Ok(version)
}

/// Minimal base64 encode (standard alphabet) for the basic-auth header.
/// Encode-only on purpose: the core never decodes credentials.
fn encode_base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((triple >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn basic_auth_value(password: &str) -> String {
    encode_base64(format!("opencode:{password}").as_bytes())
}

fn http_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .build();
    ureq::Agent::new_with_config(config)
}

/// One authenticated GET against the sidecar. Returns the bounded body.
/// Never includes the password in any error path.
fn authed_get(port: u16, password: &str, path: &str) -> Result<String, NormalizedError> {
    let auth = basic_auth_value(password);
    let url = format!("http://127.0.0.1:{port}{path}");
    let response = http_agent()
        .get(&url)
        .header("Authorization", &format!("Basic {auth}"))
        .call();
    match response {
        Ok(response) => {
            let mut reader = response
                .into_body()
                .into_reader()
                .take(MAX_HEALTH_BODY_BYTES);
            let mut body = String::new();
            reader.read_to_string(&mut body).map_err(|_| {
                host_error(
                    ErrorCode::MalformedResponse,
                    "The sidecar answer could not be read.",
                )
            })?;
            Ok(body)
        }
        Err(ureq::Error::StatusCode(status)) => Err(error_for_status(status)),
        Err(ureq::Error::Timeout(_)) => Err(host_error(
            ErrorCode::TimedOut,
            "The sidecar request timed out.",
        )),
        Err(_) => Err(host_error(
            ErrorCode::ProviderUnavailable,
            "The sidecar is unreachable. Start it and try again.",
        )),
    }
}

/// One authenticated health probe. Returns the server version.
fn probe_health(port: u16, password: &str) -> Result<String, NormalizedError> {
    parse_health(&authed_get(port, password, "/global/health")?)
}

/// One authenticated POST with a bounded JSON body. Returns the bounded
/// response body; 204 responses yield an empty string.
fn authed_post(
    port: u16,
    password: &str,
    path: &str,
    body: &str,
) -> Result<String, NormalizedError> {
    let auth = basic_auth_value(password);
    let url = format!("http://127.0.0.1:{port}{path}");
    let response = http_agent()
        .post(&url)
        .header("Authorization", &format!("Basic {auth}"))
        .header("Content-Type", "application/json")
        .send(body);
    match response {
        Ok(response) => {
            let mut reader = response
                .into_body()
                .into_reader()
                .take(MAX_HEALTH_BODY_BYTES);
            let mut text = String::new();
            reader.read_to_string(&mut text).map_err(|_| {
                host_error(
                    ErrorCode::MalformedResponse,
                    "The sidecar answer could not be read.",
                )
            })?;
            Ok(text)
        }
        Err(ureq::Error::StatusCode(404)) => Err(host_error(
            ErrorCode::ProviderUnavailable,
            "The sidecar session is gone. Send again to start a fresh one.",
        )),
        Err(ureq::Error::StatusCode(400)) => Err(host_error(
            ErrorCode::InvalidInput,
            "The sidecar rejected the request.",
        )),
        Err(ureq::Error::StatusCode(status)) => Err(error_for_status(status)),
        Err(ureq::Error::Timeout(_)) => Err(host_error(
            ErrorCode::TimedOut,
            "The sidecar request timed out.",
        )),
        Err(_) => Err(host_error(
            ErrorCode::ProviderUnavailable,
            "The sidecar is unreachable. Start it and try again.",
        )),
    }
}

fn session_id_of(body: &str) -> Result<String, NormalizedError> {
    let parsed: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        host_error(
            ErrorCode::MalformedResponse,
            "The sidecar answered with malformed session data.",
        )
    })?;
    parsed
        .get("id")
        .and_then(|value| value.as_str())
        .filter(|id| id.starts_with("ses") && !id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            host_error(
                ErrorCode::MalformedResponse,
                "The sidecar answered without a session id.",
            )
        })
}

/// One SSE frame classified for our session. Both sidecar generations
/// (`data` and `properties` envelopes) are accepted; anything else is ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FrameOutcome {
    Ignored,
    TextDelta(String),
    Done,
    Failed(String),
}

fn classify_frame(session_id: &str, frame: &serde_json::Value) -> FrameOutcome {
    let event_type = frame.get("type").and_then(|value| value.as_str());
    let container = frame.get("data").or_else(|| frame.get("properties"));
    let (Some(event_type), Some(container)) = (event_type, container) else {
        return FrameOutcome::Ignored;
    };
    let frame_session = container
        .get("sessionID")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if frame_session != session_id {
        return FrameOutcome::Ignored;
    }
    match event_type {
        "message.part.delta" => {
            let is_text = container
                .get("field")
                .and_then(|value| value.as_str())
                .is_none_or(|field| field == "text");
            if !is_text {
                return FrameOutcome::Ignored;
            }
            match container.get("delta").and_then(|value| value.as_str()) {
                Some(delta) if !delta.is_empty() => {
                    let mut text = delta.to_string();
                    if text.chars().count() > MAX_DELTA_CHARS {
                        text = text.chars().take(MAX_DELTA_CHARS).collect();
                    }
                    FrameOutcome::TextDelta(text)
                }
                _ => FrameOutcome::Ignored,
            }
        }
        "session.idle" => FrameOutcome::Done,
        "session.error" => {
            let message = container
                .get("message")
                .or_else(|| container.get("error"))
                .and_then(|value| value.as_str())
                .unwrap_or("The sidecar turn failed.");
            FrameOutcome::Failed(truncate_name(message))
        }
        _ => FrameOutcome::Ignored,
    }
}

fn truncate_send(text: &str) -> String {
    const MAX_PROMPT_CHARS: usize = 16 * 1024;
    if text.chars().count() > MAX_PROMPT_CHARS {
        text.chars().take(MAX_PROMPT_CHARS).collect()
    } else {
        text.to_string()
    }
}

/// Validate one user prompt for the sidecar. Bounded twice: bytes for the
/// wire (matches the product contract) and chars for truncation safety.
fn validate_prompt(input: &str) -> Result<String, NormalizedError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(host_error(
            ErrorCode::InvalidInput,
            "The prompt must not be empty.",
        ));
    }
    if trimmed.len() > limits::MAX_USER_MESSAGE_BYTES {
        return Err(host_error(
            ErrorCode::RequestTooLarge,
            "The prompt is larger than the 16 KiB sidecar limit.",
        ));
    }
    Ok(truncate_send(trimmed))
}

struct DoneGuard(Arc<AtomicBool>);

impl Drop for DoneGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn emit_event(app: &AppHandle, event: AgentStreamEvent) {
    let _ = app.emit(
        AGENT_EVENT_NAME,
        AgentEventEnvelope {
            contract_version: AGENT_CONTRACT_VERSION,
            event,
        },
    );
}

/// One sidecar turn: `prompt_async`, then the `/event` SSE stream filtered to
/// our fresh session. Exactly one terminal event per worker. Secrets stay in
/// `password`, which lives only in this thread's memory.
fn run_send_worker(
    app: AppHandle,
    port: u16,
    password: String,
    session_id: String,
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
) {
    let _guard = DoneGuard(done);
    emit_event(
        &app,
        AgentStreamEvent::Started {
            session: session_id.clone(),
        },
    );

    let config = ureq::Agent::config_builder()
        .timeout_global(Some(STREAM_TIMEOUT))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let auth = basic_auth_value(&password);
    let stream = agent
        .get(&format!("http://127.0.0.1:{port}/event"))
        .header("Authorization", &format!("Basic {auth}"))
        .call();
    let response = match stream {
        Ok(response) => response,
        Err(_) => {
            emit_event(
                &app,
                AgentStreamEvent::Failed {
                    session: session_id,
                    error: host_error(
                        ErrorCode::ProviderUnavailable,
                        "The sidecar event stream is unreachable.",
                    ),
                },
            );
            return;
        }
    };

    let reader = std::io::BufReader::new(response.into_body().into_reader());
    let mut streamed_chars = 0usize;
    for line in reader.lines() {
        if cancel.load(Ordering::SeqCst) {
            emit_event(
                &app,
                AgentStreamEvent::Cancelled {
                    session: session_id.clone(),
                },
            );
            return;
        }
        let Ok(line) = line else { break };
        let Some(payload) = line.strip_prefix("data:").map(str::trim) else {
            continue;
        };
        if payload.is_empty() {
            continue;
        }
        let frame: serde_json::Value = match serde_json::from_str(payload) {
            Ok(frame) => frame,
            Err(_) => continue,
        };
        match classify_frame(&session_id, &frame) {
            FrameOutcome::Ignored => {}
            FrameOutcome::TextDelta(text) => {
                streamed_chars += text.chars().count();
                if streamed_chars > MAX_SEND_CHARS {
                    emit_event(
                        &app,
                        AgentStreamEvent::Failed {
                            session: session_id.clone(),
                            error: host_error(
                                ErrorCode::ResponseTooLarge,
                                "The sidecar answer exceeded the 2 MiB turn limit.",
                            ),
                        },
                    );
                    return;
                }
                emit_event(
                    &app,
                    AgentStreamEvent::TextChunk {
                        session: session_id.clone(),
                        text,
                    },
                );
            }
            FrameOutcome::Done => {
                emit_event(
                    &app,
                    AgentStreamEvent::Completed {
                        session: session_id.clone(),
                    },
                );
                return;
            }
            FrameOutcome::Failed(message) => {
                emit_event(
                    &app,
                    AgentStreamEvent::Failed {
                        session: session_id.clone(),
                        error: host_error(ErrorCode::ProviderUnavailable, message),
                    },
                );
                return;
            }
        }
    }
    if cancel.load(Ordering::SeqCst) {
        emit_event(
            &app,
            AgentStreamEvent::Cancelled {
                session: session_id,
            },
        );
    } else {
        emit_event(
            &app,
            AgentStreamEvent::Failed {
                session: session_id,
                error: host_error(
                    ErrorCode::ProviderUnavailable,
                    "The sidecar stream ended before completion.",
                ),
            },
        );
    }
}

fn truncate_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() > MAX_NAME_CHARS {
        trimmed.chars().take(MAX_NAME_CHARS).collect()
    } else {
        trimmed.to_string()
    }
}

fn plain_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => {
            let clean = truncate_name(text);
            if clean.is_empty() {
                None
            } else {
                Some(clean)
            }
        }
        _ => None,
    }
}

/// Extract only identifiers from the providers payload. Every other field —
/// keys, URLs, prices, limits — is dropped here and never reaches IPC.
fn extract_models(body: &str) -> Result<Vec<AgentModelEntry>, NormalizedError> {
    let parsed: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        host_error(
            ErrorCode::MalformedResponse,
            "The sidecar answered with malformed provider data.",
        )
    })?;
    let mut models = Vec::new();
    let providers = parsed.get("providers").and_then(|value| value.as_array());
    let Some(providers) = providers else {
        return Err(host_error(
            ErrorCode::MalformedResponse,
            "The sidecar answered without a provider list.",
        ));
    };
    for provider in providers {
        let Some(provider_id) = provider.get("id").and_then(plain_string) else {
            continue;
        };
        let provider_name = provider
            .get("name")
            .and_then(plain_string)
            .unwrap_or_else(|| provider_id.clone());
        let models_map = provider.get("models").and_then(|value| value.as_object());
        let Some(models_map) = models_map else {
            continue;
        };
        for (key, entry) in models_map {
            if models.len() >= MAX_MODELS {
                break;
            }
            let model_id = entry
                .get("id")
                .and_then(plain_string)
                .unwrap_or_else(|| truncate_name(key));
            if model_id.is_empty() {
                continue;
            }
            let model_name = entry
                .get("name")
                .and_then(plain_string)
                .unwrap_or_else(|| model_id.clone());
            models.push(AgentModelEntry {
                provider_id: provider_id.clone(),
                provider_name: provider_name.clone(),
                model_id,
                model_name,
            });
        }
        if models.len() >= MAX_MODELS {
            break;
        }
    }
    Ok(models)
}

fn terminate_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let deadline = Instant::now() + KILL_GRACE;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => return,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

impl AgentHostState {
    fn status_locked(running: Option<&Running>, port: u16) -> AgentHostStatus {
        match running {
            Some(active) => AgentHostStatus {
                running: true,
                port: active.port,
                version: Some(active.version.clone()),
                pid: Some(active.child.id()),
            },
            None => AgentHostStatus {
                running: false,
                port,
                version: None,
                pid: None,
            },
        }
    }

    /// Start the sidecar if needed. Idempotent: a healthy running sidecar is
    /// returned as-is with refreshed `last_used`.
    pub fn start(&self, port: Option<u16>) -> Result<AgentHostStatus, NormalizedError> {
        let port = validate_port(port.unwrap_or(AGENT_HOST_DEFAULT_PORT))?;
        let mut guard = self.inner.lock().map_err(|_| {
            host_error(
                ErrorCode::ProviderUnavailable,
                "The agent host state is unavailable. Restart BrainRoot.",
            )
        })?;

        if let Some(active) = guard.as_mut() {
            if active.port == port {
                match probe_health(port, &active.password) {
                    Ok(version) => {
                        active.version = version.clone();
                        active.last_used = Instant::now();
                        return Ok(Self::status_locked(Some(active), port));
                    }
                    Err(_) => {
                        let mut stale = guard.take();
                        if let Some(running) = stale.as_mut() {
                            terminate_child(&mut running.child);
                        }
                    }
                }
            } else {
                let mut stale = guard.take();
                if let Some(running) = stale.as_mut() {
                    terminate_child(&mut running.child);
                }
            }
        }

        let password = uuid::Uuid::new_v4().to_string();
        let mut child = std::process::Command::new("opencode")
            .args([
                "serve",
                "--port",
                &port.to_string(),
                "--hostname",
                "127.0.0.1",
                "--pure",
            ])
            .env("OPENCODE_SERVER_PASSWORD", &password)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| {
                host_error(
                    ErrorCode::ProviderUnavailable,
                    "The opencode binary could not be started. Install opencode and try again.",
                )
            })?;

        let deadline = Instant::now() + READY_TIMEOUT;
        let mut last_error = host_error(
            ErrorCode::TimedOut,
            "The sidecar did not become ready in time.",
        );
        while Instant::now() < deadline {
            match probe_health(port, &password) {
                Ok(version) => {
                    let status = AgentHostStatus {
                        running: true,
                        port,
                        version: Some(version.clone()),
                        pid: Some(child.id()),
                    };
                    *guard = Some(Running {
                        child,
                        password,
                        port,
                        version,
                        last_used: Instant::now(),
                    });
                    return Ok(status);
                }
                Err(error) => {
                    // 401/429 are terminal for this password: do not spin.
                    if matches!(
                        error.code,
                        ErrorCode::AuthenticationFailed | ErrorCode::RateLimited
                    ) {
                        terminate_child(&mut child);
                        return Err(error);
                    }
                    last_error = error;
                    std::thread::sleep(READY_POLL_INTERVAL);
                }
            }
            if let Ok(Some(_)) = child.try_wait() {
                terminate_child(&mut child);
                return Err(host_error(
                    ErrorCode::ProviderUnavailable,
                    "The sidecar exited before becoming ready.",
                ));
            }
        }
        terminate_child(&mut child);
        Err(last_error)
    }

    /// Redacted status probe. Never fails: unreachable means `running: false`.
    pub fn status(&self) -> AgentHostStatus {
        let mut guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return AgentHostStatus {
                    running: false,
                    port: AGENT_HOST_DEFAULT_PORT,
                    version: None,
                    pid: None,
                }
            }
        };
        let port = guard
            .as_ref()
            .map(|r| r.port)
            .unwrap_or(AGENT_HOST_DEFAULT_PORT);
        if let Some(active) = guard.as_mut() {
            match probe_health(active.port, &active.password) {
                Ok(version) => {
                    active.version = version;
                    active.last_used = Instant::now();
                }
                Err(_) => {
                    let mut stale = guard.take();
                    if let Some(running) = stale.as_mut() {
                        terminate_child(&mut running.child);
                    }
                    return Self::status_locked(None, port);
                }
            }
        }
        Self::status_locked(guard.as_ref(), port)
    }

    /// Stop the sidecar if running. Always succeeds; already-stopped is OK.
    pub fn stop(&self) -> AgentHostStatus {
        let mut guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return AgentHostStatus {
                    running: false,
                    port: AGENT_HOST_DEFAULT_PORT,
                    version: None,
                    pid: None,
                }
            }
        };
        let mut taken = guard.take();
        if let Some(running) = taken.as_mut() {
            terminate_child(&mut running.child);
        }
        AgentHostStatus {
            running: false,
            port: AGENT_HOST_DEFAULT_PORT,
            version: None,
            pid: None,
        }
    }

    /// Live model catalog, identifiers only. Requires a running sidecar;
    /// secrets stay in the sidecar by construction of `extract_models`.
    pub fn models(&self) -> Result<AgentModelList, NormalizedError> {
        let guard = self.inner.lock().map_err(|_| {
            host_error(
                ErrorCode::ProviderUnavailable,
                "The agent host state is unavailable. Restart BrainRoot.",
            )
        })?;
        let Some(active) = guard.as_ref() else {
            return Err(host_error(
                ErrorCode::ProviderUnavailable,
                "The sidecar is not running. Start it and try again.",
            ));
        };
        let body = authed_get(active.port, &active.password, "/config/providers")?;
        let models = extract_models(&body)?;
        let selected = self
            .selection
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or(None);
        Ok(AgentModelList { models, selected })
    }

    /// Remember the chosen model for future session sends. Validates shape
    /// only; the sidecar resolves availability at send time.
    pub fn select_model(
        &self,
        provider_id: String,
        model_id: String,
    ) -> Result<AgentModelSelection, NormalizedError> {
        let provider_id = truncate_name(&provider_id);
        let model_id = truncate_name(&model_id);
        if provider_id.is_empty() || model_id.is_empty() {
            return Err(host_error(
                ErrorCode::InvalidInput,
                "The provider and model identifiers must not be empty.",
            ));
        }
        if provider_id.len() > MAX_NAME_CHARS || model_id.len() > MAX_NAME_CHARS {
            return Err(host_error(
                ErrorCode::InvalidInput,
                "The provider or model identifier is too long.",
            ));
        }
        let selection = AgentModelSelection {
            provider_id,
            model_id,
        };
        match self.selection.lock() {
            Ok(mut guard) => *guard = Some(selection.clone()),
            Err(_) => {
                return Err(host_error(
                    ErrorCode::ProviderUnavailable,
                    "The agent host state is unavailable. Restart BrainRoot.",
                ))
            }
        }
        Ok(selection)
    }

    /// Public catalog merged with the sidecar list: context lengths and
    /// per-million prices where the ids match, `None` (UNKNOWN) otherwise.
    /// Six-hour TTL; a failed refresh serves stale cache honestly, and an
    /// offline host without cache reports unavailable instead of guessing.
    pub fn catalog(&self, refresh: bool) -> Result<catalog::CatalogResult, NormalizedError> {
        let selected = self
            .selection
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or(None);
        let sidecar = match self.models() {
            Ok(list) => list.models,
            Err(error) => {
                if error.code == ErrorCode::ProviderUnavailable {
                    return Ok(catalog::CatalogResult {
                        models: Vec::new(),
                        selected,
                        stale: false,
                    });
                }
                return Err(error);
            }
        };
        let fresh = self.catalog.lock().map(|guard| {
            guard
                .fetched_at
                .is_some_and(|at| at.elapsed() < catalog::CATALOG_TTL)
        });
        let fresh = fresh.unwrap_or(false);
        if !refresh && fresh {
            let guard = self.catalog.lock().map_err(|_| {
                host_error(
                    ErrorCode::ProviderUnavailable,
                    "The agent host state is unavailable. Restart BrainRoot.",
                )
            })?;
            return Ok(catalog::CatalogResult {
                models: catalog::merge(sidecar, &guard.entries),
                selected,
                stale: false,
            });
        }
        match catalog::fetch_catalog() {
            Ok(entries) => {
                if let Ok(mut guard) = self.catalog.lock() {
                    guard.fetched_at = Some(Instant::now());
                    guard.entries = entries.clone();
                }
                Ok(catalog::CatalogResult {
                    models: catalog::merge(sidecar, &entries),
                    selected,
                    stale: false,
                })
            }
            Err(error) => {
                let stale = self.catalog.lock().map(|guard| {
                    let entries = guard.entries.clone();
                    let has = !entries.is_empty();
                    if has {
                        Some(catalog::merge(sidecar, &entries))
                    } else {
                        None
                    }
                });
                match stale {
                    Ok(Some(models)) => Ok(catalog::CatalogResult {
                        models,
                        selected,
                        stale: true,
                    }),
                    _ => Err(error),
                }
            }
        }
    }

    /// Start one sidecar turn in a fresh session: `prompt_async`, then a
    /// worker thread streams `/event` frames filtered to that session.
    /// Single-flight: a second send while one runs is rejected.
    pub fn send(
        &self,
        app: AppHandle,
        prompt: String,
    ) -> Result<AgentSendAccepted, NormalizedError> {
        let prompt = validate_prompt(&prompt)?;
        {
            let guard = self.active.lock().map_err(|_| {
                host_error(
                    ErrorCode::ProviderUnavailable,
                    "The agent host state is unavailable. Restart BrainRoot.",
                )
            })?;
            if let Some(active) = guard.as_ref() {
                if !active.done.load(Ordering::SeqCst) {
                    return Err(host_error(
                        ErrorCode::InvalidState,
                        "A sidecar turn is already running. Cancel it before sending again.",
                    ));
                }
            }
        }

        let (port, password) = {
            let guard = self.inner.lock().map_err(|_| {
                host_error(
                    ErrorCode::ProviderUnavailable,
                    "The agent host state is unavailable. Restart BrainRoot.",
                )
            })?;
            match guard.as_ref() {
                Some(active) => (active.port, active.password.clone()),
                None => {
                    return Err(host_error(
                        ErrorCode::ProviderUnavailable,
                        "The sidecar is not running. Start it and try again.",
                    ))
                }
            }
        };

        let title: String = prompt.chars().take(64).collect();
        let session_body = serde_json::json!({ "title": title }).to_string();
        let session_response = authed_post(port, &password, "/session", &session_body)?;
        let session_id = session_id_of(&session_response)?;

        let mut prompt_body = serde_json::json!({
            "parts": [{ "type": "text", "text": prompt }],
        });
        if let Ok(guard) = self.selection.lock() {
            if let Some(selection) = guard.as_ref() {
                prompt_body["model"] = serde_json::json!({
                    "providerID": selection.provider_id,
                    "modelID": selection.model_id,
                });
            }
        }
        authed_post(
            port,
            &password,
            &format!("/session/{session_id}/prompt_async"),
            &prompt_body.to_string(),
        )?;

        let cancel = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicBool::new(false));
        let worker_app = app;
        let worker_password = password;
        let worker_session = session_id.clone();
        let worker_cancel = Arc::clone(&cancel);
        let worker_done = Arc::clone(&done);
        std::thread::spawn(move || {
            run_send_worker(
                worker_app,
                port,
                worker_password,
                worker_session,
                worker_cancel,
                worker_done,
            );
        });
        if let Ok(mut guard) = self.active.lock() {
            *guard = Some(ActiveSend {
                session_id: session_id.clone(),
                cancel,
                done,
            });
        }
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(active) = guard.as_mut() {
                active.last_used = Instant::now();
            }
        }
        Ok(AgentSendAccepted {
            contract_version: AGENT_CONTRACT_VERSION,
            session: session_id,
        })
    }

    /// Cancel the in-flight turn: flag the worker, best-effort abort the
    /// server turn, keep the sidecar itself running. Idempotent.
    pub fn cancel_send(&self) -> AgentHostStatus {
        let target = self.active.lock().map(|guard| {
            guard.as_ref().map(|active| {
                active.cancel.store(true, Ordering::SeqCst);
                active.session_id.clone()
            })
        });
        if let Ok(Some(session_id)) = target {
            if let Ok(guard) = self.inner.lock() {
                if let Some(active) = guard.as_ref() {
                    let _ = authed_post(
                        active.port,
                        &active.password,
                        &format!("/session/{session_id}/abort"),
                        "{}",
                    );
                }
            }
        }
        self.status()
    }

    /// Kill the sidecar when it has been idle longer than `max_idle`.    /// Returns true when a stop happened. Governor hook for B-R5; unused until then.
    #[allow(dead_code)]
    pub fn stop_if_idle(&self, max_idle: Duration) -> bool {
        let idle = match self.inner.lock() {
            Ok(guard) => guard.as_ref().map(|r| r.last_used.elapsed()),
            Err(_) => return false,
        };
        match idle {
            Some(elapsed) if elapsed >= max_idle => {
                self.stop();
                true
            }
            _ => false,
        }
    }

    /// Blocking shutdown for window close. Never panics, never logs secrets.
    pub fn shutdown(&self) {
        let _ = self.stop();
    }
}

#[tauri::command]
pub fn agent_host_start(
    port: Option<u16>,
    state: tauri::State<'_, AgentHostState>,
) -> Result<AgentHostStatus, NormalizedError> {
    state.start(port)
}

#[tauri::command]
pub fn agent_host_status(state: tauri::State<'_, AgentHostState>) -> AgentHostStatus {
    state.status()
}

#[tauri::command]
pub fn agent_host_stop(state: tauri::State<'_, AgentHostState>) -> AgentHostStatus {
    state.stop()
}

#[tauri::command]
pub fn agent_host_models(
    state: tauri::State<'_, AgentHostState>,
) -> Result<AgentModelList, NormalizedError> {
    state.models()
}

#[tauri::command]
pub fn agent_host_select_model(
    provider_id: String,
    model_id: String,
    state: tauri::State<'_, AgentHostState>,
) -> Result<AgentModelSelection, NormalizedError> {
    state.select_model(provider_id, model_id)
}

#[tauri::command]
pub fn agent_host_send(
    prompt: String,
    app: AppHandle,
    state: tauri::State<'_, AgentHostState>,
) -> Result<AgentSendAccepted, NormalizedError> {
    state.send(app, prompt)
}

#[tauri::command]
pub fn agent_host_cancel_send(state: tauri::State<'_, AgentHostState>) -> AgentHostStatus {
    state.cancel_send()
}

#[tauri::command]
pub fn agent_host_catalog(
    refresh: Option<bool>,
    state: tauri::State<'_, AgentHostState>,
) -> Result<catalog::CatalogResult, NormalizedError> {
    state.catalog(refresh.unwrap_or(false))
}

#[cfg(test)]
mod deferred {
    use super::*;

    #[test]
    fn base64_known_vectors() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(b"Man"), "TWFu");
    }

    #[test]
    fn port_zero_rejected() {
        assert!(validate_port(0).is_err());
        assert_eq!(validate_port(4099).unwrap(), 4099);
    }

    #[test]
    fn auth_mapping_401() {
        assert_eq!(error_for_status(401).code, ErrorCode::AuthenticationFailed);
    }

    #[test]
    fn health_parsing_trims_version() {
        let long = format!("{{\"healthy\":true,\"version\":\"{}\"}}", "v".repeat(200));
        let version = parse_health(&long).unwrap();
        assert_eq!(version.chars().count(), MAX_VERSION_CHARS);
        assert!(parse_health("{\"healthy\":false,\"version\":\"1\"}").is_err());
        assert!(parse_health("not-json").is_err());
    }

    #[test]
    fn status_never_carries_secret() {
        let status = AgentHostStatus {
            running: true,
            port: 4099,
            version: Some("1.18.31".to_string()),
            pid: Some(1),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.contains("password"));
        assert!(!json.contains("opencode:"));
        let debug = format!("{status:?}");
        assert!(!debug.contains("sk-"));
    }

    #[test]
    fn passwords_unique_per_session() {
        let first = uuid::Uuid::new_v4().to_string();
        let second = uuid::Uuid::new_v4().to_string();
        assert_ne!(first, second);
    }

    #[test]
    fn models_keep_ids_and_drop_secrets() {
        let body = r#"{"providers":[{"
            id":"acme","name":"Acme","key":"sk-live-secret","models":{"
            fast":{"id":"acme-fast","name":"Acme Fast","apiKey":"sk-other"},"
            alias":{"name":"Alias Only"}}]}"#;
        let models = extract_models(body).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].provider_id, "acme");
        assert_eq!(models[0].model_id, "acme-fast");
        assert_eq!(models[1].model_id, "alias");
        assert_eq!(models[1].model_name, "Alias Only");
        let json = serde_json::to_string(&models).unwrap();
        assert!(!json.contains("sk-live-secret"));
        assert!(!json.contains("sk-other"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn models_reject_missing_catalog() {
        assert!(extract_models("{}").is_err());
        assert!(extract_models("not-json").is_err());
    }

    fn frame(value: serde_json::Value) -> FrameOutcome {
        classify_frame("ses_abc", &value)
    }

    #[test]
    fn frames_accept_both_sidecar_generations() {
        let v2 = serde_json::json!({
            "id": "evt_1", "type": "message.part.delta",
            "data": {"sessionID": "ses_abc", "messageID": "msg_1",
                     "partID": "prt_1", "field": "text", "delta": "hi"}
        });
        assert_eq!(frame(v2), FrameOutcome::TextDelta("hi".to_string()));
        let v1 = serde_json::json!({
            "id": "evt_2", "type": "session.idle",
            "properties": {"sessionID": "ses_abc"}
        });
        assert_eq!(frame(v1), FrameOutcome::Done);
    }

    #[test]
    fn frames_ignore_foreign_sessions_and_non_text() {
        let foreign = serde_json::json!({
            "id": "evt_3", "type": "message.part.delta",
            "data": {"sessionID": "ses_other", "messageID": "msg_1",
                     "partID": "prt_1", "field": "text", "delta": "hi"}
        });
        assert_eq!(frame(foreign), FrameOutcome::Ignored);
        let reasoning = serde_json::json!({
            "id": "evt_4", "type": "message.part.delta",
            "data": {"sessionID": "ses_abc", "messageID": "msg_1",
                     "partID": "prt_2", "field": "reasoning", "delta": "hmm"}
        });
        assert_eq!(frame(reasoning), FrameOutcome::Ignored);
        let unknown = serde_json::json!({"id": "evt_5", "type": "server.connected"});
        assert_eq!(frame(unknown), FrameOutcome::Ignored);
    }

    #[test]
    fn frames_report_session_errors() {
        let error = serde_json::json!({
            "id": "evt_6", "type": "session.error",
            "data": {"sessionID": "ses_abc", "message": "boom"}
        });
        assert_eq!(frame(error), FrameOutcome::Failed("boom".to_string()));
    }

    #[test]
    fn prompt_validation_bounds_input() {
        assert!(validate_prompt("  ").is_err());
        assert!(validate_prompt("hello").is_ok());
        assert!(validate_prompt(&"x".repeat(limits::MAX_USER_MESSAGE_BYTES + 1)).is_err());
        assert_eq!(session_id_of(r#"{"id":"ses_123"}"#).unwrap(), "ses_123");
        assert!(session_id_of(r#"{"id":"nope"}"#).is_err());
    }

    // T-R2-01..04 integration (sidecar start/health/stop, secret scan,
    // soak): written here at release gate, executed only at the versioned
    // batch gate per ADR 0014. Not run per microstep.
}
