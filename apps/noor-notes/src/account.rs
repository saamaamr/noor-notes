use std::io::Write;
use std::process::{Command, Stdio};

use noor_sync::{AuthSession, SupabaseClient, SyncClientError};

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error(transparent)]
    Cloud(#[from] SyncClientError),
    #[error("the desktop Secret Service is unavailable")]
    SecretServiceUnavailable,
    #[error("the desktop Secret Service rejected the credential")]
    SecretServiceRejected,
}

#[derive(Clone, Debug, Default)]
pub struct SecretServiceStore;

impl SecretServiceStore {
    pub fn store(&self, kind: &str, account: &str, secret: &[u8]) -> Result<(), AccountError> {
        let mut child = Command::new("secret-tool")
            .args([
                "store",
                "--label=Noor Notes",
                "application",
                "io.github.saamaamr.NoorNotes",
                "kind",
                kind,
                "account",
                account,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| AccountError::SecretServiceUnavailable)?;
        child
            .stdin
            .as_mut()
            .ok_or(AccountError::SecretServiceUnavailable)?
            .write_all(secret)
            .map_err(|_| AccountError::SecretServiceUnavailable)?;
        if child
            .wait()
            .map_err(|_| AccountError::SecretServiceUnavailable)?
            .success()
        {
            Ok(())
        } else {
            Err(AccountError::SecretServiceRejected)
        }
    }
}

pub struct AccountController {
    client: SupabaseClient,
    secrets: SecretServiceStore,
}

impl AccountController {
    pub fn new(client: SupabaseClient) -> Self {
        Self {
            client,
            secrets: SecretServiceStore,
        }
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthSession, AccountError> {
        let session = self.client.sign_in(email, password).await?;
        self.secrets
            .store("refresh-token", email, session.refresh_token.as_bytes())?;
        Ok(session)
    }

    pub fn store_wrapped_vault(
        &self,
        account: &str,
        wrapped_vault_json: &[u8],
    ) -> Result<(), AccountError> {
        self.secrets
            .store("wrapped-vault", account, wrapped_vault_json)
    }
}
