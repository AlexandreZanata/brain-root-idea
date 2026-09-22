//! Linux Secret Service credential storage with a session-only fallback.
//!
//! The real backend is the `keyring` crate, whose `v1` layer initializes the
//! freedesktop Secret Service (zbus) on Linux. `SecretBackend` is the
//! injectable seam so tests never touch the user's keyring; the only code that
//! does is `KeyringBackend`, exercised by the ignored opt-in smoke test.
//!
//! The store never logs and never holds the secret beyond the call that stores
//! or resolves it.

use super::credential::{
    CredentialError, CredentialReference, CredentialStatus, CredentialStore, CredentialValue,
    InMemoryCredentialStore,
};

pub const KEYRING_SERVICE: &str = "dev.brainroot.experiment";
pub const KEYRING_ACCOUNT: &str = "default";
pub const SMOKE_SERVICE: &str = "brainroot-smoke-test";
pub const SMOKE_ACCOUNT: &str = "smoke";

pub trait SecretBackend {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CredentialError>;
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), CredentialError>;
    fn delete(&self, service: &str, account: &str) -> Result<(), CredentialError>;
}

/// The only implementation that touches the real credential store.
pub struct KeyringBackend;

impl SecretBackend for KeyringBackend {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CredentialError> {
        let entry = keyring::Entry::new(service, account).map_err(map_error)?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(map_error(error)),
        }
    }

    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(service, account).map_err(map_error)?;
        entry.set_password(secret).map_err(map_error)
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(service, account).map_err(map_error)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(map_error(error)),
        }
    }
}

fn map_error(_error: keyring::Error) -> CredentialError {
    CredentialError::BackendUnavailable
}

pub struct SecretServiceCredentialStore<B: SecretBackend> {
    backend: B,
}

impl<B: SecretBackend> SecretServiceCredentialStore<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// True when the backend answers; a locked or missing Secret Service is
    /// reported as unavailable so the caller can fall back honestly.
    pub fn is_available(&self) -> bool {
        self.backend.get(KEYRING_SERVICE, KEYRING_ACCOUNT).is_ok()
    }

    fn account_matches(reference: &CredentialReference) -> Result<(), CredentialError> {
        if reference.as_str() == KEYRING_ACCOUNT {
            Ok(())
        } else {
            Err(CredentialError::UnknownReference)
        }
    }
}

impl<B: SecretBackend> CredentialStore for SecretServiceCredentialStore<B> {
    fn status(&self) -> CredentialStatus {
        match self.backend.get(KEYRING_SERVICE, KEYRING_ACCOUNT) {
            Ok(Some(_)) => CredentialStatus::Configured,
            Ok(None) => CredentialStatus::NotConfigured,
            Err(_) => CredentialStatus::Unavailable,
        }
    }

    fn reference(&self) -> Option<CredentialReference> {
        match self.status() {
            CredentialStatus::Configured => CredentialReference::new(KEYRING_ACCOUNT).ok(),
            CredentialStatus::NotConfigured | CredentialStatus::Unavailable => None,
        }
    }

    fn resolve(&self, reference: &CredentialReference) -> Result<CredentialValue, CredentialError> {
        Self::account_matches(reference)?;
        match self.backend.get(KEYRING_SERVICE, KEYRING_ACCOUNT)? {
            Some(secret) => CredentialValue::new(secret),
            None => Err(CredentialError::NotConfigured),
        }
    }

    fn set(
        &self,
        reference: CredentialReference,
        value: CredentialValue,
    ) -> Result<(), CredentialError> {
        Self::account_matches(&reference)?;
        self.backend
            .set(KEYRING_SERVICE, KEYRING_ACCOUNT, value.expose_to_core())
    }
}

/// Picks the Secret Service store when the backend answers, otherwise the
/// session-only store. The status the UI shows comes from the selected store.
pub fn select_store<B: SecretBackend + Send + Sync + 'static>(
    backend: B,
) -> Box<dyn CredentialStore + Send + Sync> {
    let store = SecretServiceCredentialStore::new(backend);
    if store.is_available() {
        Box::new(store)
    } else {
        Box::new(InMemoryCredentialStore::new())
    }
}

/// The platform store used when the app chooses Secret Service storage.
pub fn platform_store() -> Box<dyn CredentialStore + Send + Sync> {
    select_store(KeyringBackend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    const FAKE_SECRET: &str = "fake-secret-value";

    #[derive(Default)]
    struct FakeBackend {
        unavailable: bool,
        entries: Mutex<Vec<(String, String, String)>>,
    }

    impl FakeBackend {
        fn unavailable() -> Self {
            Self {
                unavailable: true,
                ..Self::default()
            }
        }
    }

    impl SecretBackend for FakeBackend {
        fn get(&self, service: &str, account: &str) -> Result<Option<String>, CredentialError> {
            if self.unavailable {
                return Err(CredentialError::BackendUnavailable);
            }
            Ok(self
                .entries
                .lock()
                .expect("fake lock is not poisoned")
                .iter()
                .find(|(stored_service, stored_account, _)| {
                    stored_service == service && stored_account == account
                })
                .map(|(_, _, secret)| secret.clone()))
        }

        fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), CredentialError> {
            if self.unavailable {
                return Err(CredentialError::BackendUnavailable);
            }
            self.entries
                .lock()
                .expect("fake lock is not poisoned")
                .push((service.to_string(), account.to_string(), secret.to_string()));
            Ok(())
        }

        fn delete(&self, service: &str, account: &str) -> Result<(), CredentialError> {
            if self.unavailable {
                return Err(CredentialError::BackendUnavailable);
            }
            self.entries
                .lock()
                .expect("fake lock is not poisoned")
                .retain(|(stored_service, stored_account, _)| {
                    !(stored_service == service && stored_account == account)
                });
            Ok(())
        }
    }

    fn reference() -> CredentialReference {
        CredentialReference::new(KEYRING_ACCOUNT).expect("valid reference")
    }

    fn value() -> CredentialValue {
        CredentialValue::new(FAKE_SECRET).expect("valid value")
    }

    #[test]
    fn empty_backend_is_not_configured() {
        let store = SecretServiceCredentialStore::new(FakeBackend::default());

        assert_eq!(store.status(), CredentialStatus::NotConfigured);
        assert_eq!(store.reference(), None);
    }

    #[test]
    fn configured_backend_reports_and_resolves() {
        let store = SecretServiceCredentialStore::new(FakeBackend::default());
        store.set(reference(), value()).expect("store accepts");

        assert_eq!(store.status(), CredentialStatus::Configured);
        assert_eq!(store.reference(), Some(reference()));
        assert_eq!(
            store
                .resolve(&reference())
                .expect("resolves")
                .expose_to_core(),
            FAKE_SECRET
        );
    }

    #[test]
    fn failed_backend_reports_unavailable() {
        let store = SecretServiceCredentialStore::new(FakeBackend::unavailable());

        assert_eq!(store.status(), CredentialStatus::Unavailable);
        assert_eq!(store.reference(), None);
    }

    #[test]
    fn set_persists_the_documented_coordinates() {
        let store = SecretServiceCredentialStore::new(FakeBackend::default());
        store.set(reference(), value()).expect("store accepts");

        assert_eq!(store.status(), CredentialStatus::Configured);
    }

    #[test]
    fn unknown_reference_is_rejected() {
        let store = SecretServiceCredentialStore::new(FakeBackend::default());
        let other = CredentialReference::new("other").expect("valid reference");

        assert_eq!(
            store.resolve(&other),
            Err(CredentialError::UnknownReference)
        );
        assert_eq!(
            store.set(other, value()),
            Err(CredentialError::UnknownReference)
        );
    }

    #[test]
    fn not_configured_resolve_is_rejected() {
        let store = SecretServiceCredentialStore::new(FakeBackend::default());

        assert_eq!(
            store.resolve(&reference()),
            Err(CredentialError::NotConfigured)
        );
    }

    #[test]
    fn unavailable_backend_errors_on_set_and_resolve() {
        let store = SecretServiceCredentialStore::new(FakeBackend::unavailable());

        assert_eq!(
            store.resolve(&reference()),
            Err(CredentialError::BackendUnavailable)
        );
        assert_eq!(
            store.set(reference(), value()),
            Err(CredentialError::BackendUnavailable)
        );
    }

    #[test]
    fn select_store_falls_back_to_session_only() {
        let unavailable = select_store(FakeBackend::unavailable());
        assert_eq!(unavailable.status(), CredentialStatus::NotConfigured);
        assert_eq!(unavailable.reference(), None);

        let secret_service = select_store(FakeBackend::default());
        assert_eq!(secret_service.status(), CredentialStatus::NotConfigured);
        secret_service
            .set(reference(), value())
            .expect("selected store accepts");
        assert_eq!(secret_service.status(), CredentialStatus::Configured);
    }

    #[test]
    #[ignore = "writes and deletes a smoke entry in the real Secret Service; run explicitly"]
    fn real_secret_service_round_trip() {
        let backend = KeyringBackend;
        let smoke_value =
            CredentialValue::new("brainroot-smoke-secret").expect("valid smoke value");

        backend
            .set(SMOKE_SERVICE, SMOKE_ACCOUNT, smoke_value.expose_to_core())
            .expect("smoke set");
        let read = backend
            .get(SMOKE_SERVICE, SMOKE_ACCOUNT)
            .expect("smoke get")
            .expect("smoke entry exists");
        assert_eq!(read, "brainroot-smoke-secret");
        backend
            .delete(SMOKE_SERVICE, SMOKE_ACCOUNT)
            .expect("smoke delete");
        assert_eq!(
            backend
                .get(SMOKE_SERVICE, SMOKE_ACCOUNT)
                .expect("smoke get after delete"),
            None
        );
    }
}
