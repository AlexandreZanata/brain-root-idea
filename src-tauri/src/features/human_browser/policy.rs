//! Typed Human Browser role and navigation policy per ADR 0013.
//!
//! The policy is the only authority for what the Human Browser view may load:
//! `http` and `https` are allowed, external protocol handlers are surfaced as
//! an explicit user action, and every other scheme, unparsable input, and URL
//! with embedded credentials is denied with a reason code. Nothing here talks
//! to a webview; the view module consumes the decisions.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserRole {
    #[default]
    Human,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationDecision {
    Allow,
    Deny { code: String },
    OpenExternal,
}

fn deny(code: &str) -> NavigationDecision {
    NavigationDecision::Deny {
        code: code.to_string(),
    }
}

/// Decides what the Human Browser may do with a navigation candidate.
pub fn decide_navigation(candidate: &str) -> NavigationDecision {
    let Ok(url) = tauri::Url::parse(candidate) else {
        return deny("human_url_invalid");
    };
    match url.scheme() {
        "http" | "https" => {
            if url.username().is_empty() && url.password().is_none() {
                NavigationDecision::Allow
            } else {
                deny("human_userinfo_denied")
            }
        }
        "mailto" | "tel" | "sms" | "magnet" => NavigationDecision::OpenExternal,
        _ => deny("human_scheme_denied"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn corpus() -> Value {
        serde_json::from_str(include_str!("../../../tests/human-navigation-corpus.json"))
            .expect("navigation corpus json")
    }

    #[test]
    fn corpus_matches_the_policy() {
        let corpus = corpus();
        for entry in corpus["allowed"].as_array().expect("allowed entries") {
            let candidate = entry.as_str().expect("allowed candidate");
            assert_eq!(
                decide_navigation(candidate),
                NavigationDecision::Allow,
                "denied {candidate}"
            );
        }
        for entry in corpus["open_external"]
            .as_array()
            .expect("external entries")
        {
            let candidate = entry.as_str().expect("external candidate");
            assert_eq!(
                decide_navigation(candidate),
                NavigationDecision::OpenExternal,
                "not external {candidate}"
            );
        }
        for (group, code) in [
            ("denied_scheme", "human_scheme_denied"),
            ("denied_other", "human_userinfo_denied"),
        ] {
            for entry in corpus[group].as_array().expect("denied entries") {
                let candidate = entry.as_str().expect("denied candidate");
                match decide_navigation(candidate) {
                    NavigationDecision::Deny { code: actual } => {
                        assert_eq!(actual, code, "wrong code for {candidate}");
                    }
                    other => panic!("allowed {candidate}: {other:?}"),
                }
            }
        }
        for entry in corpus["invalid"].as_array().expect("invalid entries") {
            let candidate = entry.as_str().expect("invalid candidate");
            match decide_navigation(candidate) {
                NavigationDecision::Deny { code } => {
                    assert_eq!(code, "human_url_invalid", "wrong code for {candidate}");
                }
                other => panic!("allowed {candidate}: {other:?}"),
            }
        }
    }
}
