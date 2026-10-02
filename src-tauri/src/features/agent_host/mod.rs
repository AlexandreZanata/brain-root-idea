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
/// Version 2 adds the non-text part events the session timeline needs
/// (reasoning, tool, file, turn divider). See ADR 0017 and
/// `docs/specs/b21-agent-event-contract.md`.
pub const AGENT_CONTRACT_VERSION: u32 = 2;
/// Bounds one streamed text delta and one full send. The sidecar is untrusted.
const MAX_DELTA_CHARS: usize = 64 * 1024;
const MAX_SEND_CHARS: usize = 2 * 1024 * 1024;
/// Per-field bounds for the v2 non-text parts. Truncation, never rejection:
/// a dropped frame would silently lose a tool's terminal state.
const MAX_TOOL_FIELD_CHARS: usize = 8 * 1024;
const MAX_FILE_PATH_CHARS: usize = 2 * 1024;
const MAX_MIME_CHARS: usize = 200;
const MAX_FILE_ENTRIES: usize = 64;
/// Per-turn ceilings, enforced in the worker like `MAX_SEND_CHARS`.
const MAX_REASONING_CHARS_PER_TURN: usize = 256 * 1024;
const MAX_PART_EVENTS_PER_TURN: usize = 512;
/// Reasoning part ids remembered per turn, to attribute deltas to the part
/// type that `message.part.updated` declared (ADR 0017).
const MAX_TRACKED_PARTS: usize = 64;
/// Stream calls may run long generations; cancel arrives via flag + abort.
const STREAM_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// Minimal stable system prompt. Deliberately short and changed only on
/// purpose: stability keeps the sidecar KV-cache hot and tokens low.
/// A test pins this text so casual edits fail loudly.
pub const SYSTEM_PROMPT: &str = "You are a coding agent inside BrainRoot, a lightweight IDE. \
Make the smallest change that satisfies the request. \
Never scan, index, or rewrite files you were not asked about. \
Prefer read-only exploration before editing. \
Explain each change in one short paragraph. \
Never reveal credentials, keys, or tokens.";
/// Agents the sidecar may run. `plan` is read-only exploration, `build` edits.
pub const AGENT_PLAN: &str = "plan";
pub const AGENT_BUILD: &str = "build";
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

/// One tool part as it crosses the boundary. `output`, `error`, and `title`
/// are `None` when the sidecar did not send them — never default-filled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolPartEvent {
    pub phase: String,
    pub tool_id: String,
    pub name: String,
    pub state: String,
    pub input: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub title: Option<String>,
}

/// One changed file as it crosses the boundary. `mime` is empty when the
/// sidecar reported a path without one; `status` is `unknown` when absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FilePartEvent {
    pub path: String,
    pub mime: String,
    pub status: String,
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
    ReasoningDelta {
        session: String,
        text: String,
    },
    ToolEvent {
        session: String,
        #[serde(flatten)]
        tool: ToolPartEvent,
    },
    FileEvent {
        session: String,
        #[serde(flatten)]
        file: FilePartEvent,
    },
    TurnDivider {
        session: String,
        reason: String,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentSendAccepted {
    pub contract_version: u32,
    pub session: String,
}

/// Token/cost ledger for one sidecar session, read from `GET /session/{id}`.
/// Numbers as reported; the sidecar owns their semantics.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TurnCost {
    pub session: String,
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub cost: f64,
}

fn validate_session_id(value: &str) -> Result<String, NormalizedError> {
    let trimmed = value.trim();
    let looks_like_session = trimmed.starts_with("ses")
        && trimmed.len() <= limits::MAX_CONVERSATION_ID_BYTES
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !looks_like_session {
        return Err(host_error(
            ErrorCode::InvalidInput,
            "The session identifier is not valid.",
        ));
    }
    Ok(trimmed.to_string())
}

fn parse_turn_cost(session: &str, body: &str) -> Result<TurnCost, NormalizedError> {
    let parsed: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        host_error(
            ErrorCode::MalformedResponse,
            "The sidecar answered with malformed session data.",
        )
    })?;
    let number = |path: &[&str]| {
        path.iter()
            .try_fold(&parsed, |value, key| value.get(key))
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
    };
    let cost = parsed
        .get("cost")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    Ok(TurnCost {
        session: session.to_string(),
        input: number(&["tokens", "input"]),
        output: number(&["tokens", "output"]),
        reasoning: number(&["tokens", "reasoning"]),
        cache_read: number(&["tokens", "cache", "read"]),
        cache_write: number(&["tokens", "cache", "write"]),
        cost,
    })
}

/// Tauri-managed sidecar owner. Everything behind mutexes so concurrent
/// commands cannot double-spawn or interleave selection writes.
#[derive(Debug)]
pub struct AgentHostState {
    inner: Mutex<Option<Running>>,
    selection: Mutex<Option<AgentModelSelection>>,
    active: Mutex<Option<ActiveSend>>,
    catalog: Mutex<CachedCatalog>,
    agent: Mutex<String>,
}

impl Default for AgentHostState {
    fn default() -> Self {
        Self {
            inner: Mutex::default(),
            selection: Mutex::default(),
            active: Mutex::default(),
            catalog: Mutex::default(),
            agent: Mutex::new(default_agent()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentConfig {
    pub agent: String,
}

fn default_agent() -> String {
    AGENT_BUILD.to_string()
}

fn validate_agent(value: &str) -> Result<String, NormalizedError> {
    let trimmed = value.trim();
    if trimmed == AGENT_PLAN || trimmed == AGENT_BUILD {
        Ok(trimmed.to_string())
    } else {
        Err(host_error(
            ErrorCode::InvalidInput,
            "The agent must be plan or build.",
        ))
    }
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
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
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
    ReasoningDelta(String),
    Tool(ToolPartEvent),
    Files(Vec<FilePartEvent>),
    TurnDivider(String),
    Done,
    Failed(String),
}

/// Reasoning part ids seen this turn. The pin's reducer only applies a delta
/// to a part it already holds, and the part type is the sole discriminator
/// between an answer delta and a reasoning delta — both arrive as
/// `field: "text"` on `message.part.delta`.
#[derive(Debug, Default)]
struct PartIndex {
    reasoning: std::collections::HashSet<String>,
}

impl PartIndex {
    fn mark_reasoning(&mut self, part_id: &str) {
        if self.reasoning.len() < MAX_TRACKED_PARTS {
            self.reasoning.insert(part_id.to_string());
        }
    }

    fn is_reasoning(&self, part_id: &str) -> bool {
        self.reasoning.contains(part_id)
    }
}

fn truncate_chars(value: &str, max: usize) -> String {
    if value.chars().count() > max {
        value.chars().take(max).collect()
    } else {
        value.to_string()
    }
}

/// Bounded, plain-text view of a tool part's `input` record. Only the
/// serialized form is forwarded; no nested map crosses the boundary.
fn tool_input_text(part: &serde_json::Value) -> String {
    let raw = part
        .get("state")
        .and_then(|state| state.get("input"))
        .map(|input| input.to_string())
        .unwrap_or_default();
    truncate_chars(&raw, MAX_TOOL_FIELD_CHARS)
}

fn optional_text(value: Option<&serde_json::Value>, max: usize) -> Option<String> {
    value
        .and_then(|value| value.as_str())
        .map(|text| truncate_chars(text, max))
}

/// Map one `tool` part onto the boundary shape. The phase is derived from the
/// declared state status; a status we do not recognise is ignored rather than
/// guessed.
fn classify_tool_part(part: &serde_json::Value) -> Option<ToolPartEvent> {
    let state = part.get("state")?;
    let status = state.get("status").and_then(|value| value.as_str())?;
    let phase = match status {
        "pending" => "called",
        "running" => "progress",
        "completed" => "success",
        "error" => "failed",
        _ => return None,
    };
    Some(ToolPartEvent {
        phase: phase.to_string(),
        tool_id: truncate_chars(
            part.get("id")
                .and_then(|value| value.as_str())
                .unwrap_or(""),
            MAX_NAME_CHARS,
        ),
        name: truncate_chars(
            part.get("tool")
                .or_else(|| part.get("name"))
                .and_then(|value| value.as_str())
                .unwrap_or(""),
            MAX_NAME_CHARS,
        ),
        state: status.to_string(),
        input: tool_input_text(part),
        output: optional_text(state.get("output"), MAX_TOOL_FIELD_CHARS),
        error: optional_text(state.get("error"), MAX_TOOL_FIELD_CHARS),
        title: optional_text(state.get("title"), MAX_NAME_CHARS),
    })
}

/// One `session.diff` entry: `{file, additions, deletions, status}`.
fn classify_diff_entries(container: &serde_json::Value) -> Vec<FilePartEvent> {
    let Some(entries) = container.get("diff").and_then(|value| value.as_array()) else {
        return Vec::new();
    };
    entries
        .iter()
        .take(MAX_FILE_ENTRIES)
        .filter_map(|entry| {
            let path = entry.get("file").and_then(|value| value.as_str())?;
            if path.is_empty() {
                return None;
            }
            Some(FilePartEvent {
                path: truncate_chars(path, MAX_FILE_PATH_CHARS),
                mime: String::new(),
                status: entry
                    .get("status")
                    .and_then(|value| value.as_str())
                    .filter(|status| matches!(*status, "added" | "modified" | "deleted"))
                    .unwrap_or("unknown")
                    .to_string(),
            })
        })
        .collect()
}

fn classify_frame(
    session_id: &str,
    frame: &serde_json::Value,
    index: &mut PartIndex,
) -> FrameOutcome {
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
            let part_id = container
                .get("partID")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let reasoning = !part_id.is_empty() && index.is_reasoning(part_id);
            match container.get("delta").and_then(|value| value.as_str()) {
                Some(delta) if !delta.is_empty() => {
                    let text = truncate_chars(delta, MAX_DELTA_CHARS);
                    if reasoning {
                        FrameOutcome::ReasoningDelta(text)
                    } else {
                        FrameOutcome::TextDelta(text)
                    }
                }
                _ => FrameOutcome::Ignored,
            }
        }
        "message.part.updated" => {
            let Some(part) = container.get("part") else {
                return FrameOutcome::Ignored;
            };
            match part.get("type").and_then(|value| value.as_str()) {
                Some("tool") => match classify_tool_part(part) {
                    Some(tool) => FrameOutcome::Tool(tool),
                    None => FrameOutcome::Ignored,
                },
                Some("reasoning") => {
                    if let Some(part_id) = part.get("id").and_then(|value| value.as_str()) {
                        index.mark_reasoning(part_id);
                    }
                    FrameOutcome::Ignored
                }
                Some("patch") => {
                    let files = part
                        .get("files")
                        .and_then(|value| value.as_array())
                        .map(|files| {
                            files
                                .iter()
                                .take(MAX_FILE_ENTRIES)
                                .filter_map(|value| value.as_str())
                                .filter(|path| !path.is_empty())
                                .map(|path| FilePartEvent {
                                    path: truncate_chars(path, MAX_FILE_PATH_CHARS),
                                    mime: String::new(),
                                    status: "unknown".to_string(),
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if files.is_empty() {
                        FrameOutcome::Ignored
                    } else {
                        FrameOutcome::Files(files)
                    }
                }
                Some("file") => {
                    let path = part
                        .get("filename")
                        .or_else(|| part.get("url"))
                        .and_then(|value| value.as_str())
                        .filter(|path| !path.is_empty());
                    match path {
                        Some(path) => FrameOutcome::Files(vec![FilePartEvent {
                            path: truncate_chars(path, MAX_FILE_PATH_CHARS),
                            mime: truncate_chars(
                                part.get("mime")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or(""),
                                MAX_MIME_CHARS,
                            ),
                            status: "unknown".to_string(),
                        }]),
                        None => FrameOutcome::Ignored,
                    }
                }
                Some("compaction") => FrameOutcome::TurnDivider("compaction".to_string()),
                _ => FrameOutcome::Ignored,
            }
        }
        "session.diff" => {
            let files = classify_diff_entries(container);
            if files.is_empty() {
                FrameOutcome::Ignored
            } else {
                FrameOutcome::Files(files)
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
    let mut reasoning_chars = 0usize;
    let mut part_events = 0usize;
    let mut index = PartIndex::default();
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
        match classify_frame(&session_id, &frame, &mut index) {
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
            FrameOutcome::ReasoningDelta(text) => {
                reasoning_chars += text.chars().count();
                if reasoning_chars > MAX_REASONING_CHARS_PER_TURN {
                    emit_event(
                        &app,
                        AgentStreamEvent::Failed {
                            session: session_id.clone(),
                            error: host_error(
                                ErrorCode::ResponseTooLarge,
                                "The sidecar reasoning exceeded the 256 KiB turn limit.",
                            ),
                        },
                    );
                    return;
                }
                emit_event(
                    &app,
                    AgentStreamEvent::ReasoningDelta {
                        session: session_id.clone(),
                        text,
                    },
                );
            }
            FrameOutcome::Tool(tool) => {
                part_events += 1;
                if part_events > MAX_PART_EVENTS_PER_TURN {
                    emit_event(
                        &app,
                        AgentStreamEvent::Failed {
                            session: session_id.clone(),
                            error: host_error(
                                ErrorCode::ResponseTooLarge,
                                "The sidecar sent more than 512 tool or file events in one turn.",
                            ),
                        },
                    );
                    return;
                }
                emit_event(
                    &app,
                    AgentStreamEvent::ToolEvent {
                        session: session_id.clone(),
                        tool,
                    },
                );
            }
            FrameOutcome::Files(files) => {
                part_events += files.len();
                if part_events > MAX_PART_EVENTS_PER_TURN {
                    emit_event(
                        &app,
                        AgentStreamEvent::Failed {
                            session: session_id.clone(),
                            error: host_error(
                                ErrorCode::ResponseTooLarge,
                                "The sidecar sent more than 512 tool or file events in one turn.",
                            ),
                        },
                    );
                    return;
                }
                for file in files {
                    emit_event(
                        &app,
                        AgentStreamEvent::FileEvent {
                            session: session_id.clone(),
                            file,
                        },
                    );
                }
            }
            FrameOutcome::TurnDivider(reason) => {
                emit_event(
                    &app,
                    AgentStreamEvent::TurnDivider {
                        session: session_id.clone(),
                        reason,
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
            "system": SYSTEM_PROMPT,
        });
        if let Ok(guard) = self.agent.lock() {
            prompt_body["agent"] = serde_json::json!(guard.clone());
        }
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

    /// Remember the chosen agent for future sends. Read-only `plan` is the
    /// cheap exploration path; `build` edits.
    pub fn set_agent(&self, agent: String) -> Result<AgentConfig, NormalizedError> {
        let agent = validate_agent(&agent)?;
        match self.agent.lock() {
            Ok(mut guard) => *guard = agent.clone(),
            Err(_) => {
                return Err(host_error(
                    ErrorCode::ProviderUnavailable,
                    "The agent host state is unavailable. Restart BrainRoot.",
                ))
            }
        }
        Ok(AgentConfig { agent })
    }
    /// Token/cost ledger for a finished turn. Any session id is accepted
    /// after shape validation; the sidecar 404s unknown ones honestly.
    pub fn turn_cost(&self, session_id: String) -> Result<TurnCost, NormalizedError> {
        let session_id = validate_session_id(&session_id)?;
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
        let body = authed_get(port, &password, &format!("/session/{session_id}"))?;
        parse_turn_cost(&session_id, &body)
    }

    /// Kill the sidecar when it has been idle longer than `max_idle`.
    /// Returns true when a stop happened. Reads timestamps only: safe for the
    /// governor tick path (no network probe).
    ///
    /// An unfinished turn always wins over the idle budget: `last_used` is set
    /// when a send starts, so a generation longer than `max_idle` would
    /// otherwise be killed mid-answer. A poisoned state lock means we cannot
    /// prove the sidecar is unused, so it is left alone rather than killed.
    pub fn stop_if_idle(&self, max_idle: Duration) -> bool {
        match self.active.lock() {
            Ok(guard) => {
                if guard
                    .as_ref()
                    .is_some_and(|active| !active.done.load(Ordering::SeqCst))
                {
                    return false;
                }
            }
            Err(_) => return false,
        }
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

    /// True while a child handle is held. Lock read only: no health probe,
    /// safe for hot paths like the governor status.
    pub fn has_sidecar(&self) -> bool {
        self.inner
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
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

#[tauri::command]
pub fn agent_host_set_agent(
    agent: String,
    state: tauri::State<'_, AgentHostState>,
) -> Result<AgentConfig, NormalizedError> {
    state.set_agent(agent)
}

#[tauri::command]
pub fn agent_host_turn_cost(
    session: String,
    state: tauri::State<'_, AgentHostState>,
) -> Result<TurnCost, NormalizedError> {
    state.turn_cost(session)
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
        let body = r#"{"providers":[{"id":"acme","name":"Acme","key":"sk-live-secret","models":{"fast":{"id":"acme-fast","name":"Acme Fast","apiKey":"sk-other"},"alias":{"name":"Alias Only"}}}]}"#;
        let models = extract_models(body).unwrap();
        assert_eq!(models.len(), 2);
        // NOTE: provider models arrive as a JSON map, so order is by key,
        // not insertion. Look up by id; the picker sorts for display.
        let by_id = |id: &str| models.iter().find(|entry| entry.model_id == id).unwrap();
        assert_eq!(by_id("acme-fast").provider_id, "acme");
        assert_eq!(by_id("alias").model_name, "Alias Only");
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
        let mut index = PartIndex::default();
        classify_frame("ses_abc", &value, &mut index)
    }

    /// Same as `frame`, but keeps the part index across frames so reasoning
    /// attribution can be exercised the way the worker does it.
    fn frame_with(value: serde_json::Value, index: &mut PartIndex) -> FrameOutcome {
        classify_frame("ses_abc", &value, index)
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
        // A `field: "reasoning"` frame is still ignored: the sidecar streams
        // both kinds as `field: "text"`, and the part type is the only
        // discriminator (ADR 0017).
        let reasoning_field = serde_json::json!({
            "id": "evt_4", "type": "message.part.delta",
            "data": {"sessionID": "ses_abc", "messageID": "msg_1",
                     "partID": "prt_2", "field": "reasoning", "delta": "hmm"}
        });
        assert_eq!(frame(reasoning_field), FrameOutcome::Ignored);
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

    // B21-S01 (deferred case IDs B20-U3a-T01..T05): the v2 non-text parts.
    // Written here, executed at the versioned release gate (ADR 0014).

    #[test]
    fn part_updated_marks_reasoning_then_attributes_its_delta() {
        let mut index = PartIndex::default();
        let updated = serde_json::json!({
            "id": "evt_7", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_r", "sessionID": "ses_abc", "messageID": "msg_1",
                "type": "reasoning", "text": "", "time": {"start": 1}
            }}
        });
        assert_eq!(frame_with(updated, &mut index), FrameOutcome::Ignored);
        let delta = serde_json::json!({
            "id": "evt_8", "type": "message.part.delta",
            "properties": {"sessionID": "ses_abc", "messageID": "msg_1",
                "partID": "prt_r", "field": "text", "delta": "thinking"}
        });
        assert_eq!(
            frame_with(delta, &mut index),
            FrameOutcome::ReasoningDelta("thinking".to_string())
        );
        // A text part keeps producing text chunks.
        let text_delta = serde_json::json!({
            "id": "evt_9", "type": "message.part.delta",
            "properties": {"sessionID": "ses_abc", "messageID": "msg_1",
                "partID": "prt_t", "field": "text", "delta": "answer"}
        });
        assert_eq!(
            frame_with(text_delta, &mut index),
            FrameOutcome::TextDelta("answer".to_string())
        );
    }

    #[test]
    fn tool_part_maps_every_state_without_inventing_fields() {
        let running = serde_json::json!({
            "id": "evt_10", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_tool", "type": "tool", "tool": "bash", "callID": "call_1",
                "state": {"status": "running", "input": {"command": "ls"}, "title": "ls"}
            }}
        });
        match frame(running) {
            FrameOutcome::Tool(tool) => {
                assert_eq!(tool.phase, "progress");
                assert_eq!(tool.name, "bash");
                assert_eq!(tool.state, "running");
                assert!(tool.input.contains("ls"));
                assert_eq!(tool.output, None);
                assert_eq!(tool.error, None);
                assert_eq!(tool.title.as_deref(), Some("ls"));
            }
            other => panic!("expected a tool event, got {other:?}"),
        }

        let completed = serde_json::json!({
            "id": "evt_11", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_tool", "type": "tool", "tool": "read", "callID": "call_1",
                "state": {"status": "completed", "input": {"path": "a.rs"},
                    "output": "ok", "title": "read", "metadata": {"secret": "sk-x"}}
            }}
        });
        match frame(completed) {
            FrameOutcome::Tool(tool) => {
                assert_eq!(tool.phase, "success");
                assert_eq!(tool.output.as_deref(), Some("ok"));
                // Arbitrary nested maps never cross the boundary.
                let json = serde_json::to_string(&tool).unwrap();
                assert!(!json.contains("sk-x"));
                assert!(!json.contains("metadata"));
            }
            other => panic!("expected a tool event, got {other:?}"),
        }

        let failed = serde_json::json!({
            "id": "evt_12", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_tool", "type": "tool", "tool": "bash", "callID": "call_1",
                "state": {"status": "error", "input": {}, "error": "exit 1"}
            }}
        });
        match frame(failed) {
            FrameOutcome::Tool(tool) => {
                assert_eq!(tool.phase, "failed");
                assert_eq!(tool.error.as_deref(), Some("exit 1"));
            }
            other => panic!("expected a tool event, got {other:?}"),
        }

        // An unknown status is ignored rather than guessed (T04).
        let unknown = serde_json::json!({
            "id": "evt_13", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_tool", "type": "tool", "tool": "bash",
                "state": {"status": "teleported", "input": {}}
            }}
        });
        assert_eq!(frame(unknown), FrameOutcome::Ignored);
        let malformed = serde_json::json!({
            "id": "evt_14", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": "not-an-object"}
        });
        assert_eq!(frame(malformed), FrameOutcome::Ignored);
    }

    #[test]
    fn session_diff_and_patch_parts_become_bounded_file_events() {
        let diff = serde_json::json!({
            "id": "evt_15", "type": "session.diff",
            "properties": {"sessionID": "ses_abc", "diff": [
                {"file": "src/a.rs", "additions": 3, "deletions": 1, "status": "modified"},
                {"file": "src/b.rs", "additions": 0, "deletions": 0},
                {"file": "", "additions": 0, "deletions": 0, "status": "added"}
            ]}
        });
        match frame(diff) {
            FrameOutcome::Files(files) => {
                assert_eq!(files.len(), 2, "the empty path is dropped, not defaulted");
                assert_eq!(files[0].path, "src/a.rs");
                assert_eq!(files[0].status, "modified");
                assert_eq!(files[0].mime, "");
                assert_eq!(
                    files[1].status, "unknown",
                    "an absent status is not guessed"
                );
            }
            other => panic!("expected file events, got {other:?}"),
        }

        let patch = serde_json::json!({
            "id": "evt_16", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_p", "type": "patch", "hash": "abc", "files": ["x.rs"]
            }}
        });
        match frame(patch) {
            FrameOutcome::Files(files) => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0].path, "x.rs");
            }
            other => panic!("expected file events, got {other:?}"),
        }

        let file_part = serde_json::json!({
            "id": "evt_17", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_f", "type": "file", "mime": "text/plain", "filename": "notes.txt"
            }}
        });
        match frame(file_part) {
            FrameOutcome::Files(files) => {
                assert_eq!(files[0].path, "notes.txt");
                assert_eq!(files[0].mime, "text/plain");
                assert_eq!(files[0].status, "unknown");
            }
            other => panic!("expected file events, got {other:?}"),
        }

        // An empty diff list is not a file event (T04).
        let empty = serde_json::json!({
            "id": "evt_18", "type": "session.diff",
            "properties": {"sessionID": "ses_abc", "diff": []}
        });
        assert_eq!(frame(empty), FrameOutcome::Ignored);
        // A file part with neither filename nor url is ignored, not invented.
        let unnamed = serde_json::json!({
            "id": "evt_18b", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {"id": "prt_f", "type": "file", "mime": "text/plain"}}
        });
        assert_eq!(frame(unnamed), FrameOutcome::Ignored);
    }

    #[test]
    fn compaction_part_is_a_turn_divider() {
        let compaction = serde_json::json!({
            "id": "evt_19", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_c", "type": "compaction", "auto": true
            }}
        });
        assert_eq!(
            frame(compaction),
            FrameOutcome::TurnDivider("compaction".to_string())
        );
    }

    #[test]
    fn foreign_sessions_are_ignored_for_every_new_variant() {
        for frame_value in [
            serde_json::json!({"id": "e1", "type": "message.part.updated", "properties": {
                "sessionID": "ses_other", "part": {"id": "p", "type": "tool", "tool": "bash",
                "state": {"status": "running", "input": {}}}}}),
            serde_json::json!({"id": "e2", "type": "session.diff", "properties": {
                "sessionID": "ses_other", "diff": [{"file": "a.rs"}]}}),
            serde_json::json!({"id": "e3", "type": "message.part.updated", "properties": {
                "sessionID": "ses_other", "part": {"id": "p", "type": "compaction"}}}),
        ] {
            assert_eq!(frame(frame_value), FrameOutcome::Ignored);
        }
    }

    #[test]
    fn oversized_tool_and_file_fields_truncate_instead_of_rejecting() {
        let huge = "x".repeat(MAX_TOOL_FIELD_CHARS + 500);
        let tool = serde_json::json!({
            "id": "evt_20", "type": "message.part.updated",
            "properties": {"sessionID": "ses_abc", "part": {
                "id": "prt_tool", "type": "tool", "tool": "bash",
                "state": {"status": "completed", "input": {}, "output": huge, "title": "t"}
            }}
        });
        match frame(tool) {
            FrameOutcome::Tool(tool) => {
                assert_eq!(tool.output.unwrap().chars().count(), MAX_TOOL_FIELD_CHARS);
            }
            other => panic!("expected a tool event, got {other:?}"),
        }

        let long_path = "p".repeat(MAX_FILE_PATH_CHARS + 10);
        let diff = serde_json::json!({
            "id": "evt_21", "type": "session.diff",
            "properties": {"sessionID": "ses_abc", "diff": [{"file": long_path}]}
        });
        match frame(diff) {
            FrameOutcome::Files(files) => {
                assert_eq!(files[0].path.chars().count(), MAX_FILE_PATH_CHARS);
            }
            other => panic!("expected file events, got {other:?}"),
        }
    }

    #[test]
    fn contract_version_two_serializes_without_secret_shaped_fields() {
        let tool = ToolPartEvent {
            phase: "success".to_string(),
            tool_id: "prt_1".to_string(),
            name: "bash".to_string(),
            state: "completed".to_string(),
            input: "{\"command\":\"ls\"}".to_string(),
            output: Some("ok".to_string()),
            error: None,
            title: None,
        };
        let envelope = AgentEventEnvelope {
            contract_version: AGENT_CONTRACT_VERSION,
            event: AgentStreamEvent::ToolEvent {
                session: "ses_abc".to_string(),
                tool,
            },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"contract_version\":2"));
        assert!(json.contains("\"type\":\"tool_event\""));
        assert!(json.contains("\"phase\":\"success\""));
        assert!(!json.contains("password"));
        assert!(!json.contains("Authorization"));
    }

    #[test]
    fn agent_selection_accepts_plan_and_build() {
        assert_eq!(validate_agent("plan").unwrap(), AGENT_PLAN);
        assert_eq!(validate_agent("build").unwrap(), AGENT_BUILD);
        assert!(validate_agent("turbo").is_err());
        assert!(validate_agent("").is_err());
        assert_eq!(
            AgentHostState::default().agent.lock().unwrap().as_str(),
            "build"
        );
    }

    #[test]
    fn system_prompt_stays_minimal_and_stable() {
        // Pinned on purpose: edits bust the sidecar KV-cache and cost tokens.
        // Change only with a spec note justifying the extra tokens.
        assert!(SYSTEM_PROMPT.contains("smallest change"));
        assert!(SYSTEM_PROMPT.contains("Never reveal credentials"));
        assert!(
            SYSTEM_PROMPT.len() < 600,
            "system prompt grew to {} bytes",
            SYSTEM_PROMPT.len()
        );
    }

    #[test]
    fn prompt_validation_bounds_input() {
        assert!(validate_prompt("  ").is_err());
        assert!(validate_prompt("hello").is_ok());
        assert!(validate_prompt(&"x".repeat(limits::MAX_USER_MESSAGE_BYTES + 1)).is_err());
        assert_eq!(session_id_of(r#"{"id":"ses_123"}"#).unwrap(), "ses_123");
        assert!(session_id_of(r#"{"id":"nope"}"#).is_err());
    }

    #[test]
    fn idle_stop_never_kills_an_active_turn() {
        let state = AgentHostState::default();
        // No active send: the timestamp path decides (no sidecar, so no stop).
        assert!(!state.stop_if_idle(Duration::ZERO));

        let cancel = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicBool::new(false));
        *state.active.lock().unwrap() = Some(ActiveSend {
            session_id: "ses_active".to_string(),
            cancel: Arc::clone(&cancel),
            done: Arc::clone(&done),
        });
        // An unfinished turn is never stopped, even with a zero budget.
        assert!(!state.stop_if_idle(Duration::ZERO));

        // Once the worker signals completion the budget applies again.
        done.store(true, Ordering::SeqCst);
        assert!(!state.stop_if_idle(Duration::ZERO));
    }

    #[test]
    fn turn_cost_reads_ledger_and_rejects_bad_ids() {
        let body = r#"{"id":"ses_1","tokens":{"input":1200,"output":340,"reasoning":50,"cache":{"read":1000,"write":20}},"cost":0.0042}"#;
        let cost = parse_turn_cost("ses_1", body).unwrap();
        assert_eq!(cost.input, 1200);
        assert_eq!(cost.output, 340);
        assert_eq!(cost.reasoning, 50);
        assert_eq!(cost.cache_read, 1000);
        assert_eq!(cost.cost, 0.0042);
        let sparse = parse_turn_cost("ses_1", r#"{"id":"ses_1"}"#).unwrap();
        assert_eq!(sparse.input, 0);
        assert!(validate_session_id("ses_abc-123_X").is_ok());
        assert!(validate_session_id("../../../etc").is_err());
        assert!(validate_session_id("nope").is_err());
    }

    // T-R2-01..04 integration (sidecar start/health/stop, secret scan,
    // soak): written here at release gate, executed only at the versioned
    // batch gate per ADR 0014. Not run per microstep.
}
