use std::sync::Arc;

use noor_sync::{AuthSession, OAuthPkce, SignUpOutcome, SupabaseClient, SyncClientError};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::key_store::{KeyStore, KeyStoreError, Oo7KeyStore, SecretKind};

const ACTIVE_CLOUD_SESSION: &str = "active";

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error(transparent)]
    Cloud(#[from] SyncClientError),
    #[error(transparent)]
    Secret(#[from] KeyStoreError),
    #[error("the stored cloud session is invalid")]
    InvalidStoredSession,
    #[error("the cloud account changed unexpectedly")]
    AccountMismatch,
}

#[derive(Serialize, Deserialize)]
struct StoredCloudSession {
    user_id: String,
    email: String,
    refresh_token: String,
}

#[derive(Clone)]
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

    pub async fn sign_up(
        &self,
        email: &str,
        password: &str,
    ) -> Result<SignUpOutcome, AccountError> {
        let outcome = self.client.sign_up(email, password).await?;
        if let Some(session) = outcome.session.as_ref() {
            self.persist_session(session).await?;
        }
        Ok(outcome)
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthSession, AccountError> {
        let session = self.client.sign_in(email, password).await?;
        self.persist_session(&session).await?;
        Ok(session)
    }

    pub fn google_oauth_pkce(&self, redirect_to: &str) -> Result<OAuthPkce, AccountError> {
        Ok(self.client.google_oauth_pkce(redirect_to)?)
    }

    pub async fn complete_google_sign_in(
        &self,
        auth_code: &str,
        code_verifier: &str,
    ) -> Result<AuthSession, AccountError> {
        let session = self
            .client
            .exchange_oauth_code(auth_code, code_verifier)
            .await?;
        self.persist_session(&session).await?;
        Ok(session)
    }

    pub async fn restore_session(&self) -> Result<Option<AuthSession>, AccountError> {
        let Some(stored) = self
            .secrets
            .get(SecretKind::CloudSession, ACTIVE_CLOUD_SESSION)
            .await?
        else {
            return Ok(None);
        };
        let stored: StoredCloudSession =
            serde_json::from_slice(&stored).map_err(|_| AccountError::InvalidStoredSession)?;
        let session = self.client.refresh_session(&stored.refresh_token).await?;
        self.secrets
            .delete(SecretKind::CloudSession, ACTIVE_CLOUD_SESSION)
            .await?;
        if session.user.id != stored.user_id {
            return Err(AccountError::AccountMismatch);
        }
        self.persist_session(&session).await?;
        Ok(Some(session))
    }
    pub async fn remember_session(&self, session: &AuthSession) -> Result<(), AccountError> {
        self.persist_session(session).await
    }

    pub async fn refresh_session(&self, refresh_token: &str) -> Result<AuthSession, AccountError> {
        let session = self.client.refresh_session(refresh_token).await?;
        self.persist_session(&session).await?;
        Ok(session)
    }

    async fn persist_session(&self, session: &AuthSession) -> Result<(), AccountError> {
        let stored = StoredCloudSession {
            user_id: session.user.id.clone(),
            email: session.user.email.clone(),
            refresh_token: session.refresh_token.clone(),
        };
        let encoded = Zeroizing::new(
            serde_json::to_vec(&stored).map_err(|_| AccountError::InvalidStoredSession)?,
        );
        self.secrets
            .put(
                SecretKind::CloudSession,
                ACTIVE_CLOUD_SESSION,
                encoded.as_slice(),
            )
            .await?;
        Ok(())
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

    pub async fn sign_out(&self, access_token: Option<&str>) -> Result<(), AccountError> {
        let remote_result = match access_token {
            Some(access_token) => self.client.sign_out(access_token).await,
            None => Ok(()),
        };
        self.secrets
            .delete(SecretKind::CloudSession, ACTIVE_CLOUD_SESSION)
            .await?;
        remote_result.map_err(AccountError::Cloud)
    }
}
