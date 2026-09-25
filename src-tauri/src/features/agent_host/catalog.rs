//! Public OpenRouter catalog merged with the sidecar model list.
//!
//! The catalog is untrusted, externally mutable input: only identifiers,
//! context lengths, and per-million USD prices are kept. Anything else is
//! dropped. Prices are labeled by fetch time and never hardcoded.

use std::time::Duration;

use serde::Serialize;

use super::{host_error, truncate_name, AgentModelEntry};
use crate::provider::contract::{ErrorCode, NormalizedError};
use crate::provider::discovery::BRAINROOT_USER_AGENT;

pub const CATALOG_URL: &str = "https://openrouter.ai/api/v1/models";
pub const CATALOG_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const CATALOG_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_CATALOG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_MERGED: usize = 500;

#[derive(Debug, Clone, Default)]
pub struct OpenRouterEntry {
    pub id: String,
    pub context_length: Option<u64>,
    pub prompt_per_m: Option<f64>,
    pub completion_per_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CatalogModel {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub model_name: String,
    pub context_length: Option<u64>,
    pub prompt_usd_per_m: Option<f64>,
    pub completion_usd_per_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CatalogResult {
    pub models: Vec<CatalogModel>,
    pub selected: Option<super::AgentModelSelection>,
    pub stale: bool,
}

fn parse_million(raw: &serde_json::Value) -> Option<f64> {
    let text = match raw {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        _ => return None,
    };
    text.parse::<f64>().ok().map(|per_token| per_token * 1e6)
}

fn parse_entry(value: &serde_json::Value) -> Option<OpenRouterEntry> {
    let id = value.get("id")?.as_str()?;
    let id = truncate_name(id);
    if id.is_empty() {
        return None;
    }
    let context_length = value
        .get("context_length")
        .and_then(|length| length.as_u64());
    let pricing = value.get("pricing");
    let prompt_per_m = pricing
        .and_then(|pricing| pricing.get("prompt"))
        .and_then(parse_million);
    let completion_per_m = pricing
        .and_then(|pricing| pricing.get("completion"))
        .and_then(parse_million);
    Some(OpenRouterEntry {
        id,
        context_length,
        prompt_per_m,
        completion_per_m,
    })
}

pub fn parse_catalog(body: &str) -> Result<Vec<OpenRouterEntry>, NormalizedError> {
    let parsed: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        host_error(
            ErrorCode::MalformedResponse,
            "The public catalog answered with malformed data.",
        )
    })?;
    let items = parsed.get("data").and_then(|data| data.as_array());
    let Some(items) = items else {
        return Err(host_error(
            ErrorCode::MalformedResponse,
            "The public catalog answered without a model list.",
        ));
    };
    Ok(items.iter().filter_map(parse_entry).collect())
}

/// Match one sidecar model id against the catalog: exact id, then the slug
/// after the `/`, then containment. First hit wins; no match is `None`
/// (UNKNOWN price/context, never guessed).
pub fn match_entry(model_id: &str, catalog: &[OpenRouterEntry]) -> Option<OpenRouterEntry> {
    let want = model_id.trim().to_lowercase();
    if want.is_empty() {
        return None;
    }
    if let Some(hit) = catalog.iter().find(|entry| entry.id.to_lowercase() == want) {
        return Some(hit.clone());
    }
    if let Some(hit) = catalog.iter().find(|entry| {
        entry
            .id
            .rsplit('/')
            .next()
            .is_some_and(|slug| slug.to_lowercase() == want)
    }) {
        return Some(hit.clone());
    }
    catalog
        .iter()
        .find(|entry| {
            let id = entry.id.to_lowercase();
            id.contains(&want) || want.contains(&id)
        })
        .cloned()
}

/// Merge keeping sidecar identifiers; catalog only fills context and prices.
pub fn merge(sidecar: Vec<AgentModelEntry>, catalog: &[OpenRouterEntry]) -> Vec<CatalogModel> {
    sidecar
        .into_iter()
        .take(MAX_MERGED)
        .map(|entry| {
            let hit = match_entry(&entry.model_id, catalog);
            CatalogModel {
                provider_id: entry.provider_id,
                provider_name: entry.provider_name,
                model_id: entry.model_id,
                model_name: entry.model_name,
                context_length: hit.as_ref().and_then(|hit| hit.context_length),
                prompt_usd_per_m: hit.as_ref().and_then(|hit| hit.prompt_per_m),
                completion_usd_per_m: hit.as_ref().and_then(|hit| hit.completion_per_m),
            }
        })
        .collect()
}

pub fn fetch_catalog() -> Result<Vec<OpenRouterEntry>, NormalizedError> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(CATALOG_TIMEOUT))
        .build();
    let response = ureq::Agent::new_with_config(config)
        .get(CATALOG_URL)
        .header("User-Agent", BRAINROOT_USER_AGENT)
        .call();
    match response {
        Ok(response) => {
            use std::io::Read;
            let mut reader = response.into_body().into_reader().take(MAX_CATALOG_BYTES);
            let mut body = String::new();
            reader.read_to_string(&mut body).map_err(|_| {
                host_error(
                    ErrorCode::MalformedResponse,
                    "The public catalog answer could not be read.",
                )
            })?;
            parse_catalog(&body)
        }
        Err(ureq::Error::StatusCode(status)) => Err(super::error_for_status(status)),
        Err(ureq::Error::Timeout(_)) => Err(host_error(
            ErrorCode::TimedOut,
            "The public catalog request timed out.",
        )),
        Err(_) => Err(host_error(
            ErrorCode::ProviderUnavailable,
            "The public catalog is unreachable. Check the network and try again.",
        )),
    }
}

#[cfg(test)]
mod deferred {
    use super::*;

    #[test]
    fn prices_parse_strings_and_numbers() {
        let body = r#"{"data":[
            {"id":"acme/fast","name":"Fast","context_length":128000,
             "pricing":{"prompt":"0.000003","completion":"0.000015"}},
            {"id":"acme/free","name":"Free","pricing":{"prompt":0,"completion":0}}
        ]}"#;
        let entries = parse_catalog(body).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].prompt_per_m, Some(3.0));
        assert_eq!(entries[0].completion_per_m, Some(15.0));
        assert_eq!(entries[0].context_length, Some(128000));
        assert_eq!(entries[1].prompt_per_m, Some(0.0));
    }

    #[test]
    fn matching_prefers_exact_then_slug() {
        let catalog = vec![
            OpenRouterEntry {
                id: "acme/fast".to_string(),
                ..Default::default()
            },
            OpenRouterEntry {
                id: "other/fast".to_string(),
                ..Default::default()
            },
        ];
        assert_eq!(match_entry("acme/fast", &catalog).unwrap().id, "acme/fast");
        assert_eq!(match_entry("fast", &catalog).unwrap().id, "acme/fast");
        assert!(match_entry("unknown-xyz", &catalog).is_none());
    }

    #[test]
    fn merge_keeps_sidecar_ids_and_unknown_stays_empty() {
        let sidecar = vec![AgentModelEntry {
            provider_id: "p".to_string(),
            provider_name: "P".to_string(),
            model_id: "fast".to_string(),
            model_name: "Fast".to_string(),
        }];
        let catalog = vec![OpenRouterEntry {
            id: "acme/fast".to_string(),
            context_length: Some(1000),
            prompt_per_m: Some(1.0),
            completion_per_m: Some(2.0),
        }];
        let merged = merge(sidecar, &catalog);
        assert_eq!(merged[0].model_id, "fast");
        assert_eq!(merged[0].model_name, "Fast");
        assert_eq!(merged[0].context_length, Some(1000));
        let merged = merge(
            vec![AgentModelEntry {
                provider_id: "p".to_string(),
                provider_name: "P".to_string(),
                model_id: "nope".to_string(),
                model_name: "Nope".to_string(),
            }],
            &catalog,
        );
        assert_eq!(merged[0].context_length, None);
        assert_eq!(merged[0].prompt_usd_per_m, None);
    }
}
