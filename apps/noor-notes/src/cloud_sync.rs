use std::sync::Arc;

use chrono::Utc;
use noor_crypto::{CryptoError, RecoveryKey, Vault};
use noor_storage::SqliteNoteRepository;
use noor_sync::{
    AuthSession, RemoteVault, SupabaseClient, SyncClientError, SyncCursor, SyncCycle, SyncStatus,
    SyncWorker,
};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

use crate::account::{AccountController, AccountError};
use crate::key_store::{KeyStore, KeyStoreError, SecretKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CloudSyncState {
    #[default]
    SignedOut,
    EnrollmentRequired,
    RecoveryConfirmation,
    Locked,
    Ready,
    Running,
    Offline,
    AuthRequired,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum CloudSyncError {
    #[error("encrypted sync is not ready for this action")]
    InvalidState,
    #[error("the sync passphrase must contain at least 12 characters")]
    WeakPassphrase,
    #[error("the recovery key confirmation does not match")]
    RecoveryMismatch,
    #[error("stored encrypted sync material is invalid")]
    InvalidStoredMaterial,
    #[error(transparent)]
    Cloud(#[from] SyncClientError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    Secret(#[from] KeyStoreError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

struct PendingEnrollment {
    vault: Arc<Vault>,
    remote: RemoteVault,
    recovery: Zeroizing<String>,
}

#[derive(Default)]
struct Runtime {
    state: CloudSyncState,
    session: Option<AuthSession>,
    remote: Option<RemoteVault>,
    vault: Option<Arc<Vault>>,
    cursor: SyncCursor,
    pending: Option<PendingEnrollment>,
}

#[derive(Clone)]
pub struct CloudSyncController {
    repository: SqliteNoteRepository,
    client: SupabaseClient,
    account: AccountController,
    secrets: Arc<dyn KeyStore>,
    runtime: Arc<Mutex<Runtime>>,
}

impl CloudSyncController {
    pub fn new(
        repository: SqliteNoteRepository,
        client: SupabaseClient,
        secrets: Arc<dyn KeyStore>,
    ) -> Self {
        Self {
            repository,
            account: AccountController::with_key_store(client.clone(), secrets.clone()),
            client,
            secrets,
            runtime: Arc::new(Mutex::new(Runtime::default())),
        }
    }

    pub async fn state(&self) -> CloudSyncState {
        self.runtime.lock().await.state
    }

    pub async fn attach_session(
        &self,
        session: AuthSession,
    ) -> Result<CloudSyncState, CloudSyncError> {
        self.account.remember_session(&session).await?;
        let user_id = session.user.id.clone();
        let remote = match self.client.get_vault(&session.access_token).await {
            Ok(remote) => remote,
            Err(error) => {
                self.runtime.lock().await.state = state_for_client_error(&error);
                return Err(error.into());
            }
        };
        let cursor = self.load_cursor(&user_id).await?;
        if let Some(remote) = remote {
            self.store_remote_vault(&user_id, &remote).await?;
            let mut runtime = self.runtime.lock().await;
            runtime.state = CloudSyncState::Locked;
            runtime.session = Some(session);
            runtime.remote = Some(remote);
            runtime.vault = None;
            runtime.cursor = cursor;
            runtime.pending = None;
            Ok(runtime.state)
        } else {
            let mut runtime = self.runtime.lock().await;
            runtime.state = CloudSyncState::EnrollmentRequired;
            runtime.session = Some(session);
            runtime.remote = None;
            runtime.vault = None;
            runtime.cursor = cursor;
            runtime.pending = None;
            Ok(runtime.state)
        }
    }

    pub async fn begin_enrollment(
        &self,
        passphrase: &[u8],
    ) -> Result<Zeroizing<String>, CloudSyncError> {
        if std::str::from_utf8(passphrase).map_or(true, |value| value.chars().count() < 12) {
            return Err(CloudSyncError::WeakPassphrase);
        }
        let mut runtime = self.runtime.lock().await;
        if runtime.state != CloudSyncState::EnrollmentRequired || runtime.session.is_none() {
            return Err(CloudSyncError::InvalidState);
        }
        let (vault, wrapped_vault) = Vault::create(passphrase)?;
        let recovery_key = RecoveryKey::generate();
        let recovery = Zeroizing::new(recovery_key.encode());
        let remote = RemoteVault {
            wrapped_vault,
            recovery_wrapped_vault: vault.wrap_for_recovery(&recovery_key)?,
            updated_at: Utc::now(),
        };
        runtime.pending = Some(PendingEnrollment {
            vault: Arc::new(vault),
            remote,
            recovery: Zeroizing::new(recovery.to_string()),
        });
        runtime.state = CloudSyncState::RecoveryConfirmation;
        Ok(recovery)
    }

    pub async fn confirm_enrollment(&self, confirmation: &str) -> Result<(), CloudSyncError> {
        let (session, pending) = {
            let mut runtime = self.runtime.lock().await;
            if runtime.state != CloudSyncState::RecoveryConfirmation {
                return Err(CloudSyncError::InvalidState);
            }
            let session = runtime
                .session
                .clone()
                .ok_or(CloudSyncError::InvalidState)?;
            let pending = runtime.pending.take().ok_or(CloudSyncError::InvalidState)?;
            (session, pending)
        };
        let confirmed = RecoveryKey::decode(confirmation)
            .map(|key| key.encode() == *pending.recovery)
            .unwrap_or(false);
        if !confirmed {
            self.restore_pending(pending).await;
            return Err(CloudSyncError::RecoveryMismatch);
        }
        if let Err(error) = self
            .client
            .put_vault(&session.access_token, &pending.remote)
            .await
        {
            self.restore_pending(pending).await;
            return Err(error.into());
        }
        if let Err(error) = self
            .store_remote_vault(&session.user.id, &pending.remote)
            .await
        {
            self.restore_pending(pending).await;
            return Err(error);
        }
        self.store_cursor(&session.user.id, SyncCursor::default())
            .await?;
        let mut runtime = self.runtime.lock().await;
        runtime.remote = Some(pending.remote);
        runtime.vault = Some(pending.vault);
        runtime.cursor = SyncCursor::default();
        runtime.state = CloudSyncState::Ready;
        Ok(())
    }

    async fn restore_pending(&self, pending: PendingEnrollment) {
        let mut runtime = self.runtime.lock().await;
        runtime.pending = Some(pending);
        runtime.state = CloudSyncState::RecoveryConfirmation;
    }

    pub async fn unlock_with_passphrase(&self, passphrase: &[u8]) -> Result<(), CloudSyncError> {
        let remote = {
            let runtime = self.runtime.lock().await;
            if runtime.state != CloudSyncState::Locked {
                return Err(CloudSyncError::InvalidState);
            }
            runtime.remote.clone().ok_or(CloudSyncError::InvalidState)?
        };
        let vault = Arc::new(Vault::unlock(passphrase, &remote.wrapped_vault)?);
        let mut runtime = self.runtime.lock().await;
        runtime.vault = Some(vault);
        runtime.state = CloudSyncState::Ready;
        Ok(())
    }

    pub async fn unlock_with_recovery(&self, recovery: &str) -> Result<(), CloudSyncError> {
        let remote = {
            let runtime = self.runtime.lock().await;
            if runtime.state != CloudSyncState::Locked {
                return Err(CloudSyncError::InvalidState);
            }
            runtime.remote.clone().ok_or(CloudSyncError::InvalidState)?
        };
        let recovery = RecoveryKey::decode(recovery)?;
        let vault = Arc::new(Vault::unlock_with_recovery(
            &recovery,
            &remote.recovery_wrapped_vault,
        )?);
        let mut runtime = self.runtime.lock().await;
        runtime.vault = Some(vault);
        runtime.state = CloudSyncState::Ready;
        Ok(())
    }

    pub async fn run_once(&self, device_id: &str) -> Result<SyncCycle, CloudSyncError> {
        let (session, vault, cursor) = {
            let mut runtime = self.runtime.lock().await;
            if !matches!(
                runtime.state,
                CloudSyncState::Ready | CloudSyncState::Offline | CloudSyncState::Error
            ) {
                return Err(CloudSyncError::InvalidState);
            }
            let session = runtime
                .session
                .clone()
                .ok_or(CloudSyncError::InvalidState)?;
            let vault = runtime.vault.clone().ok_or(CloudSyncError::InvalidState)?;
            runtime.state = CloudSyncState::Running;
            (session, vault, runtime.cursor)
        };
        let worker = SyncWorker::new(
            self.repository.clone(),
            self.client.clone(),
            vault.clone(),
            session.access_token.clone(),
        );
        let mut cycle = worker.run_cycle(cursor, device_id, Utc::now()).await;
        let mut active_session = session;
        if cycle.status == SyncStatus::AuthRequired {
            active_session = self
                .account
                .refresh_session(&active_session.refresh_token)
                .await?;
            let worker = SyncWorker::new(
                self.repository.clone(),
                self.client.clone(),
                vault,
                active_session.access_token.clone(),
            );
            cycle = worker.run_cycle(cycle.cursor, device_id, Utc::now()).await;
        }
        if cycle.cursor != cursor {
            self.store_cursor(&active_session.user.id, cycle.cursor)
                .await?;
        }
        let mut runtime = self.runtime.lock().await;
        runtime.session = Some(active_session);
        runtime.cursor = cycle.cursor;
        runtime.state = state_for_status(&cycle.status);
        Ok(cycle)
    }

    pub async fn disable(&self) {
        *self.runtime.lock().await = Runtime::default();
    }

    async fn load_cursor(&self, user_id: &str) -> Result<SyncCursor, CloudSyncError> {
        let Some(encoded) = self.secrets.get(SecretKind::SyncCursor, user_id).await? else {
            return Ok(SyncCursor::default());
        };
        serde_json::from_slice(&encoded).map_err(|_| CloudSyncError::InvalidStoredMaterial)
    }

    async fn store_cursor(&self, user_id: &str, cursor: SyncCursor) -> Result<(), CloudSyncError> {
        let encoded = Zeroizing::new(
            serde_json::to_vec(&cursor).map_err(|_| CloudSyncError::InvalidStoredMaterial)?,
        );
        self.secrets
            .put(SecretKind::SyncCursor, user_id, &encoded)
            .await?;
        Ok(())
    }

    async fn store_remote_vault(
        &self,
        user_id: &str,
        remote: &RemoteVault,
    ) -> Result<(), CloudSyncError> {
        let encoded = Zeroizing::new(
            serde_json::to_vec(remote).map_err(|_| CloudSyncError::InvalidStoredMaterial)?,
        );
        self.secrets
            .put(SecretKind::SyncVault, user_id, &encoded)
            .await?;
        Ok(())
    }
}

fn state_for_client_error(error: &SyncClientError) -> CloudSyncState {
    match error {
        SyncClientError::AuthRequired => CloudSyncState::AuthRequired,
        SyncClientError::Transport(_)
        | SyncClientError::Http(reqwest::StatusCode::SERVICE_UNAVAILABLE)
        | SyncClientError::RateLimited(_) => CloudSyncState::Offline,
        _ => CloudSyncState::Error,
    }
}

fn state_for_status(status: &SyncStatus) -> CloudSyncState {
    match status {
        SyncStatus::Idle => CloudSyncState::Ready,
        SyncStatus::Syncing => CloudSyncState::Running,
        SyncStatus::Offline => CloudSyncState::Offline,
        SyncStatus::AuthRequired => CloudSyncState::AuthRequired,
        SyncStatus::Error => CloudSyncState::Error,
    }
}
