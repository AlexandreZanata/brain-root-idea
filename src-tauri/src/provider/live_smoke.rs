//! Opt-in live OpenCode Go smoke test (test-only).
//!
//! Never part of the default gates: the test is `#[ignore]`d and additionally
//! requires `BRAINROOT_LIVE_SMOKE=1` and `BRAINROOT_OPENCODE_GO_KEY` in the
//! environment. It sends one harmless fixed prompt through the validated
//! `chat/completions` path and records only counts, field names, and timing;
//! response bodies and the credential never reach stdout, the repository, the
//! PR, the Wiki, or the issue evidence.

use std::io::Read;
use std::time::{Duration, Instant};

use super::contract::{limits, ConversationId, StreamEvent, UserMessage};
use super::credential::{AuthorizationHeader, CredentialValue};
use super::discovery::{DiscoveryClient, HttpTransport, UreqTransport, MODELS_URL};
use super::execution::ExecutionLimits;
use super::go::{headers, ChatStreamParser, GoConfig, GoTransport, SessionId, UreqGoTransport};

const OPT_IN_VAR: &str = "BRAINROOT_LIVE_SMOKE";
const KEY_VAR: &str = "BRAINROOT_OPENCODE_GO_KEY";
const FIXED_PROMPT: &str = "Reply with the single word: ready";
const READ_BUFFER_BYTES: usize = 8 * 1024;

#[test]
#[ignore = "live provider request; requires BRAINROOT_LIVE_SMOKE=1 and BRAINROOT_OPENCODE_GO_KEY"]
fn live_go_smoke() {
    if std::env::var(OPT_IN_VAR).as_deref() != Ok("1") {
        eprintln!("live smoke skipped: set {OPT_IN_VAR}=1 to opt in");
        return;
    }
    let Some(key) = std::env::var(KEY_VAR)
        .ok()
        .filter(|key| !key.trim().is_empty())
    else {
        panic!("{OPT_IN_VAR}=1 requires {KEY_VAR} in the environment");
    };
    let credential = CredentialValue::new(key).expect("the opt-in key is not empty");
    let timeout = ExecutionLimits::default().total;

    check_discovery(&credential, timeout);
    check_chat(&credential, timeout);
}

fn check_discovery(credential: &CredentialValue, timeout: Duration) {
    let client = DiscoveryClient::new(UreqTransport::with_timeout(timeout));
    let started = Instant::now();
    let models = client
        .fetch(credential)
        .unwrap_or_else(|failure| panic!("live discovery failed: {}", failure.message()));
    let elapsed = started.elapsed();

    if models.is_empty() {
        let keys = observed_entry_keys(credential, timeout);
        panic!(
            "live discovery returned no models; observed entry field names: {keys:?} \
             (if the authenticated payload matches the 2026-09-22 public shape, B03-S07 owns the remediation)"
        );
    }
    eprintln!(
        "live smoke discovery ok: models={} elapsed_ms={}",
        models.len(),
        elapsed.as_millis()
    );
}

/// Diagnostic for the empty-catalog failure: prints only the union of JSON
/// field names in the payload, never values.
fn observed_entry_keys(credential: &CredentialValue, timeout: Duration) -> Vec<String> {
    let authorization = AuthorizationHeader::bearer(credential);
    let body = UreqTransport::with_timeout(timeout)
        .get(
            MODELS_URL,
            &[("Authorization", authorization.expose_to_core())],
        )
        .unwrap_or_else(|failure| {
            panic!("live discovery diagnostic failed: {}", failure.message())
        });
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else {
        return Vec::new();
    };
    let entries = match value {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::Object(object) => object
            .get("data")
            .and_then(|data| data.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let mut keys: Vec<String> = entries
        .iter()
        .filter_map(|entry| entry.as_object())
        .flat_map(|object| object.keys().cloned())
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

fn check_chat(credential: &CredentialValue, timeout: Duration) {
    let config = GoConfig::default_go();
    let session = SessionId::generate();
    let conversation = ConversationId::new("live-smoke").expect("static conversation id is valid");
    let message = UserMessage::new(FIXED_PROMPT).expect("static prompt is valid");
    let body = config
        .build_request_body(&message)
        .expect("the static request body builds");

    let started = Instant::now();
    let mut reader = UreqGoTransport::with_timeout(timeout)
        .post(config.endpoint(), &headers(credential, &session), &body)
        .unwrap_or_else(|failure| panic!("live request failed: {}", failure.message()));

    let mut parser = ChatStreamParser::new(conversation);
    let mut buffer = [0u8; READ_BUFFER_BYTES];
    let mut bytes = 0usize;
    let mut chunks = 0usize;
    let mut completed = false;

    loop {
        let read = reader
            .read(&mut buffer)
            .unwrap_or_else(|error| panic!("live stream read failed: {error}"));
        if read == 0 {
            break;
        }
        bytes += read;
        assert!(
            bytes <= limits::MAX_RESPONSE_BYTES,
            "live response exceeded the {} byte limit",
            limits::MAX_RESPONSE_BYTES
        );
        for event in parser
            .push(&buffer[..read])
            .unwrap_or_else(|failure| panic!("live stream unreadable: {}", failure.message()))
        {
            match event {
                StreamEvent::TextChunk { .. } => chunks += 1,
                StreamEvent::Completed { .. } => completed = true,
                StreamEvent::Failed { error } => panic!("live request failed: {}", error.message),
                StreamEvent::Started | StreamEvent::Cancelled { .. } => {}
            }
        }
        if completed {
            break;
        }
    }

    let terminated_by_stream = parser.is_finished();
    if !terminated_by_stream {
        for event in parser.finish() {
            match event {
                StreamEvent::Completed { .. } => completed = true,
                StreamEvent::Failed { error } => panic!("live request failed: {}", error.message),
                _ => {}
            }
        }
    }
    let elapsed = started.elapsed();
    drop(reader);

    assert!(chunks >= 1, "the live stream produced no text chunk");
    assert!(
        completed,
        "the live stream never reached a completed terminal"
    );
    eprintln!(
        "live smoke chat ok: chunks={chunks} bytes={bytes} terminated_by_stream={terminated_by_stream} elapsed_ms={}",
        elapsed.as_millis()
    );
}
