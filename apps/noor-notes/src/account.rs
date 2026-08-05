use std::sync::Arc;

use noor_sync::{AuthSession, SupabaseClient, SyncClientError};

use crate::key_store::{KeyStore, KeyStoreError, Oo7KeyStore, SecretKind};

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error(transparent)]
    Cloud(#[from] SyncClientError),
    #[error(transparent)]
    Secret(#[from] KeyStoreError),
}

pub struct AccountController {
    client: SupabaseClient,
    secrets: Arc<dyn KeyStore>,
}

impl AccountController {
    pub async fn new(client: SupabaseClient) -> Result<Self, AccountError> {
        Ok(Self {
            client,
            secrets: Arc::new(Oo7KeyStore::new().await?),
        })
    }

    pub fn with_key_store(client: SupabaseClient, secrets: Arc<dyn KeyStore>) -> Self {
        Self { client, secrets }
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthSession, AccountError> {
        let session = self.client.sign_in(email, password).await?;
        self.secrets
            .put(
                SecretKind::RefreshToken,
                email,
                session.refresh_token.as_bytes(),
            )
            .await?;
        Ok(session)
    }

    pub async fn store_wrapped_vault(
        &self,
        account: &str,
        wrapped_vault_json: &[u8],
    ) -> Result<(), AccountError> {
        self.secrets
            .put(SecretKind::WrappedVault, account, wrapped_vault_json)
            .await?;
        Ok(())
    }

    pub async fn sign_out(&self, account: &str) -> Result<(), AccountError> {
        self.secrets
            .delete(SecretKind::RefreshToken, account)
            .await?;
        self.secrets
            .delete(SecretKind::WrappedVault, account)
            .await?;
        Ok(())
    }
}
