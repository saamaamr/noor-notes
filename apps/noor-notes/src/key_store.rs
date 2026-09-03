use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use zeroize::Zeroizing;

const APP_ID: &str = "io.github.saamaamr.NoorNotes";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SecretKind {
    DatabaseKey,
    RefreshToken,
    CloudSession,
    WrappedVault,
    WritingAssistanceApiKey,
}

impl SecretKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::DatabaseKey => "database-key",
            Self::RefreshToken => "refresh-token",
            Self::CloudSession => "cloud-session",
            Self::WrappedVault => "wrapped-vault",
            Self::WritingAssistanceApiKey => "writing-assistance-api-key",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("the desktop Secret Service is unavailable")]
    Unavailable,
    #[error("the Secret Service returned ambiguous credentials")]
    Ambiguous,
}

#[async_trait]
pub trait KeyStore: Send + Sync {
    async fn get(
        &self,
        kind: SecretKind,
        account: &str,
    ) -> Result<Option<Zeroizing<Vec<u8>>>, KeyStoreError>;
    async fn put(&self, kind: SecretKind, account: &str, value: &[u8])
    -> Result<(), KeyStoreError>;
    async fn delete(&self, kind: SecretKind, account: &str) -> Result<(), KeyStoreError>;
}

#[derive(Debug)]
pub struct Oo7KeyStore {
    keyring: oo7::Keyring,
}

impl Oo7KeyStore {
    pub async fn new() -> Result<Self, KeyStoreError> {
        let keyring = oo7::Keyring::new()
            .await
            .map_err(|_| KeyStoreError::Unavailable)?;
        Ok(Self { keyring })
    }
}

fn attributes(kind: SecretKind, account: &str) -> Vec<(&'static str, &str)> {
    vec![
        ("application", APP_ID),
        ("kind", kind.as_str()),
        ("account", account),
    ]
}

#[async_trait]
impl KeyStore for Oo7KeyStore {
    async fn get(
        &self,
        kind: SecretKind,
        account: &str,
    ) -> Result<Option<Zeroizing<Vec<u8>>>, KeyStoreError> {
        let items = self
            .keyring
            .search_items(&attributes(kind, account))
            .await
            .map_err(|_| KeyStoreError::Unavailable)?;
        if items.len() > 1 {
            return Err(KeyStoreError::Ambiguous);
        }
        let Some(item) = items.first() else {
            return Ok(None);
        };
        let secret = item
            .secret()
            .await
            .map_err(|_| KeyStoreError::Unavailable)?;
        Ok(Some(Zeroizing::new(secret.as_bytes().to_vec())))
    }

    async fn put(
        &self,
        kind: SecretKind,
        account: &str,
        value: &[u8],
    ) -> Result<(), KeyStoreError> {
        self.keyring
            .create_item(
                "Noor Notes",
                &attributes(kind, account),
                oo7::Secret::blob(value),
                true,
            )
            .await
            .map_err(|_| KeyStoreError::Unavailable)
    }

    async fn delete(&self, kind: SecretKind, account: &str) -> Result<(), KeyStoreError> {
        self.keyring
            .delete(&attributes(kind, account))
            .await
            .map_err(|_| KeyStoreError::Unavailable)
    }
}

#[derive(Clone, Default)]
pub struct InMemoryKeyStore {
    values: SharedTestSecrets,
}

type SharedTestSecrets = Arc<Mutex<HashMap<(SecretKind, String), Vec<Zeroizing<Vec<u8>>>>>>;

impl InMemoryKeyStore {
    pub fn inject_duplicate_for_test(&self, kind: SecretKind, account: &str, value: &[u8]) {
        let mut values = self.values.lock().expect("test key store mutex poisoned");
        let entry = values.entry((kind, account.to_owned())).or_default();
        entry.push(Zeroizing::new(value.to_vec()));
        entry.push(Zeroizing::new(value.to_vec()));
    }
}

#[async_trait]
impl KeyStore for InMemoryKeyStore {
    async fn get(
        &self,
        kind: SecretKind,
        account: &str,
    ) -> Result<Option<Zeroizing<Vec<u8>>>, KeyStoreError> {
        let values = self.values.lock().expect("test key store mutex poisoned");
        match values.get(&(kind, account.to_owned())).map(Vec::as_slice) {
            None | Some([]) => Ok(None),
            Some([value]) => Ok(Some(Zeroizing::new(value.to_vec()))),
            Some(_) => Err(KeyStoreError::Ambiguous),
        }
    }

    async fn put(
        &self,
        kind: SecretKind,
        account: &str,
        value: &[u8],
    ) -> Result<(), KeyStoreError> {
        self.values
            .lock()
            .expect("test key store mutex poisoned")
            .insert(
                (kind, account.to_owned()),
                vec![Zeroizing::new(value.to_vec())],
            );
        Ok(())
    }

    async fn delete(&self, kind: SecretKind, account: &str) -> Result<(), KeyStoreError> {
        self.values
            .lock()
            .expect("test key store mutex poisoned")
            .remove(&(kind, account.to_owned()));
        Ok(())
    }
}
