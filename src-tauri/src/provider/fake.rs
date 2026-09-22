//! Deterministic fake provider.
//!
//! Speaks the fake wire format documented here. All content is fixed strings
//! and time appears only as scripted data: no network, thread, randomness, or
//! wall-clock read exists in this module.
//!
//! Wire frames:
//!
//! ```text
//! {"kind":"start"}
//! {"kind":"chunk","text":"…"}
//! {"kind":"end"}
//! {"kind":"error","code":"…","message":"…"}
//! ```

use std::time::Duration;

use super::contract::{ModelDescriptor, ModelId};

pub const FAKE_DELAY: Duration = Duration::from_secs(5);
pub const FAKE_CHUNKS: [&str; 3] = ["Hello", ", ", "BrainRoot"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakeScenario {
    Success,
    AuthFailure,
    RateLimited,
    ServerError,
    MalformedEvent,
    DelayedEvent,
    NeverEnding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeWireStep {
    Frame(String),
    Delay(Duration),
    Hang,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FakeProvider;

impl FakeProvider {
    pub fn models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor::new(model_id("fake-streaming"), "Fake Streaming", true)
                .expect("static fake model data is valid"),
            ModelDescriptor::new(model_id("fake-fast"), "Fake Fast", true)
                .expect("static fake model data is valid"),
        ]
    }

    pub fn script(&self, scenario: FakeScenario) -> Vec<FakeWireStep> {
        match scenario {
            FakeScenario::Success => {
                let mut steps = vec![frame(r#"{"kind":"start"}"#)];
                steps.extend(FAKE_CHUNKS.iter().map(|text| chunk(text)));
                steps.push(frame(r#"{"kind":"end"}"#));
                steps
            }
            FakeScenario::AuthFailure => vec![error_frame(
                "authentication_failed",
                "The provider rejected the credential.",
            )],
            FakeScenario::RateLimited => vec![error_frame(
                "rate_limited",
                "The provider temporarily limited requests.",
            )],
            FakeScenario::ServerError => vec![error_frame(
                "provider_unavailable",
                "The provider is temporarily unavailable.",
            )],
            FakeScenario::MalformedEvent => vec![
                frame(r#"{"kind":"start"}"#),
                frame(r#"{"kind":"chunk","text":"#),
                frame(r#"{"kind":"end"}"#),
            ],
            FakeScenario::DelayedEvent => vec![
                frame(r#"{"kind":"start"}"#),
                FakeWireStep::Delay(FAKE_DELAY),
                chunk(FAKE_CHUNKS[0]),
                frame(r#"{"kind":"end"}"#),
            ],
            FakeScenario::NeverEnding => vec![
                frame(r#"{"kind":"start"}"#),
                chunk(FAKE_CHUNKS[0]),
                chunk(FAKE_CHUNKS[1]),
                FakeWireStep::Hang,
            ],
        }
    }
}

fn model_id(value: &str) -> ModelId {
    ModelId::new(value).expect("static fake model ID is valid")
}

fn frame(payload: &str) -> FakeWireStep {
    FakeWireStep::Frame(payload.to_string())
}

fn chunk(text: &str) -> FakeWireStep {
    FakeWireStep::Frame(format!(r#"{{"kind":"chunk","text":"{text}"}}"#))
}

fn error_frame(code: &str, message: &str) -> FakeWireStep {
    FakeWireStep::Frame(format!(
        r#"{{"kind":"error","code":"{code}","message":"{message}"}}"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn provider() -> FakeProvider {
        FakeProvider
    }

    fn kind_of(step: &FakeWireStep) -> String {
        match step {
            FakeWireStep::Frame(payload) => serde_json::from_str::<Value>(payload)
                .ok()
                .and_then(|value| {
                    value
                        .get("kind")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "malformed".to_string()),
            FakeWireStep::Delay(_) => "delay".to_string(),
            FakeWireStep::Hang => "hang".to_string(),
        }
    }

    fn kinds(steps: &[FakeWireStep]) -> Vec<String> {
        steps.iter().map(kind_of).collect()
    }

    fn field(step: &FakeWireStep, key: &str) -> Option<String> {
        let FakeWireStep::Frame(payload) = step else {
            return None;
        };
        serde_json::from_str::<Value>(payload)
            .ok()
            .and_then(|value| value.get(key).and_then(Value::as_str).map(str::to_string))
    }

    #[test]
    fn models_are_deterministic_and_streaming() {
        let first = provider().models();
        let second = provider().models();

        assert_eq!(first, second);
        assert!(first.iter().all(|model| model.supports_streaming));
        assert_eq!(first[0].id.as_str(), "fake-streaming");
        assert_eq!(first[1].display_name, "Fake Fast");
    }

    #[test]
    fn success_script_starts_chunks_and_ends() {
        let steps = provider().script(FakeScenario::Success);

        assert_eq!(
            kinds(&steps),
            vec!["start", "chunk", "chunk", "chunk", "end"]
        );
        assert_eq!(field(&steps[1], "text").as_deref(), Some(FAKE_CHUNKS[0]));
        assert_eq!(field(&steps[3], "text").as_deref(), Some(FAKE_CHUNKS[2]));
    }

    #[test]
    fn success_script_has_no_delay_or_hang() {
        let steps = provider().script(FakeScenario::Success);

        assert!(!steps
            .iter()
            .any(|step| matches!(step, FakeWireStep::Delay(_) | FakeWireStep::Hang)));
    }

    #[test]
    fn auth_failure_script_is_a_single_error() {
        let steps = provider().script(FakeScenario::AuthFailure);

        assert_eq!(kinds(&steps), vec!["error"]);
        assert_eq!(
            field(&steps[0], "code").as_deref(),
            Some("authentication_failed")
        );
        assert!(field(&steps[0], "message").is_some_and(|message| !message.is_empty()));
    }

    #[test]
    fn rate_limit_script_reports_rate_limited() {
        let steps = provider().script(FakeScenario::RateLimited);

        assert_eq!(kinds(&steps), vec!["error"]);
        assert_eq!(field(&steps[0], "code").as_deref(), Some("rate_limited"));
    }

    #[test]
    fn server_error_script_reports_provider_unavailable() {
        let steps = provider().script(FakeScenario::ServerError);

        assert_eq!(kinds(&steps), vec!["error"]);
        assert_eq!(
            field(&steps[0], "code").as_deref(),
            Some("provider_unavailable")
        );
    }

    #[test]
    fn malformed_script_contains_an_invalid_frame() {
        let steps = provider().script(FakeScenario::MalformedEvent);

        assert_eq!(kinds(&steps), vec!["start", "malformed", "end"]);
        assert!(steps.iter().any(|step| match step {
            FakeWireStep::Frame(payload) => serde_json::from_str::<Value>(payload).is_err(),
            _ => false,
        }));
    }

    #[test]
    fn delayed_script_carries_delay_data() {
        let steps = provider().script(FakeScenario::DelayedEvent);

        assert_eq!(kinds(&steps), vec!["start", "delay", "chunk", "end"]);
        assert_eq!(steps[1], FakeWireStep::Delay(FAKE_DELAY));
    }

    #[test]
    fn never_ending_script_ends_with_hang() {
        let steps = provider().script(FakeScenario::NeverEnding);

        assert_eq!(kinds(&steps), vec!["start", "chunk", "chunk", "hang"]);
        assert_eq!(steps.last(), Some(&FakeWireStep::Hang));
    }

    #[test]
    fn scripts_are_stable_across_calls() {
        for scenario in [
            FakeScenario::Success,
            FakeScenario::AuthFailure,
            FakeScenario::RateLimited,
            FakeScenario::ServerError,
            FakeScenario::MalformedEvent,
            FakeScenario::DelayedEvent,
            FakeScenario::NeverEnding,
        ] {
            assert_eq!(
                provider().script(scenario),
                provider().script(scenario),
                "scenario {scenario:?} must be deterministic"
            );
        }
    }
}
