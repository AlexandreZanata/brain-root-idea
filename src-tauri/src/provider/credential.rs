//! Core-only credential boundary.
//!
//! Only [`CredentialStatus`] is serializable. Credential values, authorization
//! headers, and references never cross to the frontend and redact themselves in
//! debug output. No real key is stored anywhere: [`InMemoryCredentialStore`] is
//! a deterministic fake for tests and the pre-credential experiment.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatus {
    Configured,
    NotConfigured,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    NotConfigured,
    UnknownReference,
    BackendUnavailable,
    Empty,
}

impl CredentialError {
    pub fn message(&self) -> String {
        match self {
            CredentialError::NotConfigured => {
                "No credential is configured for this provider.".to_string()
            }
            CredentialError::UnknownReference => {
                "The credential reference does not exist.".to_string()
            }
            CredentialError::BackendUnavailable => {
                "The credential store is not available on this system.".to_string()
            }
            CredentialError::Empty => "The credential value must not be empty.".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CredentialValue(String);

impl CredentialValue {
    pub fn new(value: impl Into<String>) -> Result<Self, CredentialError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CredentialError::Empty);
        }
        Ok(Self(value))
    }

    /// Core-only accessor for the transport layer. Never serialize the result.
    pub fn expose_to_core(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for CredentialValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("CredentialValue([redacted])")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AuthorizationHeader(String);

impl AuthorizationHeader {
    pub fn bearer(value: &CredentialValue) -> Self {
        Self(format!("Bearer {}", value.expose_to_core()))
    }

    /// Core-only accessor for the HTTP layer. Never serialize the result.
    pub fn expose_to_core(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for AuthorizationHeader {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AuthorizationHeader([redacted])")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CredentialReference(String);

impl CredentialReference {
    pub fn new(id: impl Into<String>) -> Result<Self, CredentialError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(CredentialError::Empty);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for CredentialReference {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("CredentialReference([redacted])")
    }
}

pub trait CredentialStore {
    fn status(&self) -> CredentialStatus;
    fn reference(&self) -> Option<CredentialReference>;
    fn resolve(&self, reference: &CredentialReference) -> Result<CredentialValue, CredentialError>;
    fn set(
        &self,
        reference: CredentialReference,
        value: CredentialValue,
    ) -> Result<(), CredentialError>;
}

#[derive(Default)]
pub struct InMemoryCredentialStore {
    entries: Mutex<Vec<(CredentialReference, CredentialValue)>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .expect("credential store lock is not poisoned")
            .clear();
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn status(&self) -> CredentialStatus {
        let entries = self
            .entries
            .lock()
            .expect("credential store lock is not poisoned");
        if entries.is_empty() {
            CredentialStatus::NotConfigured
        } else {
            CredentialStatus::Configured
        }
    }

    fn reference(&self) -> Option<CredentialReference> {
        self.entries
            .lock()
            .expect("credential store lock is not poisoned")
            .first()
            .map(|(reference, _)| reference.clone())
    }

    fn resolve(&self, reference: &CredentialReference) -> Result<CredentialValue, CredentialError> {
        let entries = self
            .entries
            .lock()
            .expect("credential store lock is not poisoned");
        entries
            .iter()
            .find(|(existing, _)| existing == reference)
            .map(|(_, value)| value.clone())
            .ok_or(CredentialError::UnknownReference)
    }

    fn set(
        &self,
        reference: CredentialReference,
        value: CredentialValue,
    ) -> Result<(), CredentialError> {
        let mut entries = self
            .entries
            .lock()
            .expect("credential store lock is not poisoned");
        entries.retain(|(existing, _)| existing != &reference);
        entries.push((reference, value));
        Ok(())
    }
}

#[derive(Clone)]
pub struct ProviderState {
    store: Arc<Mutex<Box<dyn CredentialStore + Send + Sync>>>,
}

impl ProviderState {
    pub fn new() -> Self {
        Self::with_store(InMemoryCredentialStore::new())
    }

    pub fn with_store(store: impl CredentialStore + Send + Sync + 'static) -> Self {
        Self::with_boxed_store(Box::new(store))
    }

    pub fn with_boxed_store(store: Box<dyn CredentialStore + Send + Sync>) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }

    pub fn credential_status(&self) -> CredentialStatus {
        self.store
            .lock()
            .expect("provider state lock is not poisoned")
            .status()
    }

    pub fn credential_reference(&self) -> Option<CredentialReference> {
        self.store
            .lock()
            .expect("provider state lock is not poisoned")
            .reference()
    }

    pub fn set_credential(
        &self,
        reference: CredentialReference,
        value: CredentialValue,
    ) -> Result<(), CredentialError> {
        self.store
            .lock()
            .expect("provider state lock is not poisoned")
            .set(reference, value)
    }

    pub fn resolve(
        &self,
        reference: &CredentialReference,
    ) -> Result<CredentialValue, CredentialError> {
        self.store
            .lock()
            .expect("provider state lock is not poisoned")
            .resolve(reference)
    }
}

impl Default for ProviderState {
    fn default() -> Self {
        Self::new()
    }
}

/// Frontend-visible status only; the value never crosses this boundary.
#[tauri::command]
pub fn provider_status(state: tauri::State<'_, ProviderState>) -> CredentialStatus {
    state.credential_status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FAKE_SECRET: &str = "fake-secret-value";

    fn fake_value() -> CredentialValue {
        CredentialValue::new(FAKE_SECRET).expect("fake value is valid")
    }

    #[test]
    fn empty_store_is_not_configured() {
        let store = InMemoryCredentialStore::new();

        assert_eq!(store.status(), CredentialStatus::NotConfigured);
        assert_eq!(store.reference(), None);
        let reference = CredentialReference::new("default").expect("valid reference");
        assert_eq!(
            store.resolve(&reference),
            Err(CredentialError::UnknownReference)
        );
    }

    #[test]
    fn stored_credential_is_configured_and_resolves() {
        let store = InMemoryCredentialStore::new();
        let reference = CredentialReference::new("default").expect("valid reference");
        store
            .set(reference.clone(), fake_value())
            .expect("session store accepts");

        assert_eq!(store.status(), CredentialStatus::Configured);
        assert_eq!(store.reference(), Some(reference.clone()));
        let value = store.resolve(&reference).expect("resolves");
        assert_eq!(value.expose_to_core(), FAKE_SECRET);
    }

    #[test]
    fn clearing_returns_to_not_configured() {
        let store = InMemoryCredentialStore::new();
        let reference = CredentialReference::new("default").expect("valid reference");
        store
            .set(reference, fake_value())
            .expect("session store accepts");
        store.clear();

        assert_eq!(store.status(), CredentialStatus::NotConfigured);
    }

    #[test]
    fn unknown_reference_is_rejected() {
        let store = InMemoryCredentialStore::new();
        store
            .set(
                CredentialReference::new("default").expect("valid reference"),
                fake_value(),
            )
            .expect("session store accepts");
        let other = CredentialReference::new("other").expect("valid reference");

        assert_eq!(
            store.resolve(&other),
            Err(CredentialError::UnknownReference)
        );
        assert!(!CredentialError::UnknownReference.message().is_empty());
    }

    #[test]
    fn empty_value_is_rejected() {
        assert_eq!(CredentialValue::new("   "), Err(CredentialError::Empty));
        assert_eq!(CredentialReference::new(""), Err(CredentialError::Empty));
    }

    #[test]
    fn credential_debug_is_redacted() {
        let rendered = format!("{:?}", fake_value());

        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains(FAKE_SECRET));
    }

    #[test]
    fn authorization_header_debug_is_redacted() {
        let header = AuthorizationHeader::bearer(&fake_value());
        let rendered = format!("{header:?}");

        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains(FAKE_SECRET));
        assert_eq!(header.expose_to_core(), format!("Bearer {FAKE_SECRET}"));
    }

    #[test]
    fn reference_debug_is_redacted() {
        let rendered = format!("{:?}", CredentialReference::new("default").expect("valid"));

        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains("default"));
    }

    #[test]
    fn status_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(CredentialStatus::Configured).expect("serializes"),
            Value::String("configured".to_string())
        );
        assert_eq!(
            serde_json::to_value(CredentialStatus::NotConfigured).expect("serializes"),
            Value::String("not_configured".to_string())
        );
        assert_eq!(
            serde_json::to_value(CredentialStatus::Unavailable).expect("serializes"),
            Value::String("unavailable".to_string())
        );
    }

    #[test]
    fn provider_state_tracks_status() {
        let state = ProviderState::new();
        assert_eq!(state.credential_status(), CredentialStatus::NotConfigured);

        let reference = CredentialReference::new("default").expect("valid reference");
        state
            .set_credential(reference.clone(), fake_value())
            .expect("session store accepts");

        assert_eq!(state.credential_status(), CredentialStatus::Configured);
        assert_eq!(state.credential_reference(), Some(reference.clone()));
        assert_eq!(
            state
                .resolve(&reference)
                .expect("resolves")
                .expose_to_core(),
            FAKE_SECRET
        );
    }

    #[test]
    fn provider_state_clones_share_the_core_only_store() {
        let state = ProviderState::new();
        let worker_state = state.clone();
        let reference = CredentialReference::new("default").expect("valid reference");

        state
            .set_credential(reference.clone(), fake_value())
            .expect("session store accepts");

        assert_eq!(
            worker_state.credential_status(),
            CredentialStatus::Configured
        );
        assert_eq!(
            worker_state
                .resolve(&reference)
                .expect("shared store resolves")
                .expose_to_core(),
            FAKE_SECRET
        );
    }
}
