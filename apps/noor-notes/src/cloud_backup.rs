use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use noor_storage::{NoteSort, SqliteNoteRepository, StorageError};
use noor_sync::{
    BackupArchive, BackupArchiveError, BackupObject, BackupPreview, BackupProvider,
    BackupProviderError, BackupProviderKind, EncryptedBackup, GoogleDriveProvider, MergeOutcome,
    OAuthPkce, OneDriveProvider, ProviderOAuth, ProviderOAuthError, ProviderSession,
    merge_remote_revision,
};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

use crate::cloud_sync::{CloudSyncController, CloudSyncError};
use crate::key_store::{KeyStore, KeyStoreError, SecretKind};

const CURRENT_BACKUP: &str = "current.nnbackup";
const SESSION_ACCOUNT: &str = "active";

#[derive(Clone, Default)]
pub struct BackupConfiguration {
    google: Option<ProviderOAuth>,
    onedrive: Option<ProviderOAuth>,
}

impl BackupConfiguration {
    pub fn load() -> Self {
        let google = std::env::var("NOOR_GOOGLE_DRIVE_CLIENT_ID")
            .ok()
            .and_then(|id| ProviderOAuth::google(id).ok());
        let onedrive = std::env::var("NOOR_ONEDRIVE_CLIENT_ID")
            .ok()
            .and_then(|id| ProviderOAuth::onedrive(id).ok());
        Self { google, onedrive }
    }

    pub fn is_available(&self, kind: BackupProviderKind) -> bool {
        self.oauth(kind).is_some()
    }

    #[doc(hidden)]
    pub fn for_test(google: Option<ProviderOAuth>, onedrive: Option<ProviderOAuth>) -> Self {
        Self { google, onedrive }
    }

    fn oauth(&self, kind: BackupProviderKind) -> Option<ProviderOAuth> {
        match kind {
            BackupProviderKind::GoogleDrive => self.google.clone(),
            BackupProviderKind::OneDrive => self.onedrive.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderBackupResult {
    pub provider: BackupProviderKind,
    pub uploaded: bool,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestorePreview {
    pub token: String,
    pub provider: BackupProviderKind,
    pub object: BackupObject,
    pub archive: BackupPreview,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreReport {
    pub applied: usize,
    pub conflicts: usize,
    pub ignored: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum CloudBackupError {
    #[error("backup provider is not configured")]
    NotConfigured,
    #[error("backup provider is not connected")]
    NotConnected,
    #[error("restore confirmation is missing or expired")]
    RestoreNotConfirmed,
    #[error(transparent)]
    Sync(#[from] CloudSyncError),
    #[error(transparent)]
    OAuth(#[from] ProviderOAuthError),
    #[error(transparent)]
    Provider(#[from] BackupProviderError),
    #[error(transparent)]
    Archive(#[from] BackupArchiveError),
    #[error(transparent)]
    Secret(#[from] KeyStoreError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("stored backup session is invalid")]
    InvalidSession,
}

#[derive(Serialize, Deserialize)]
struct StoredProviderSession {
    refresh_token: String,
}

struct PendingRestore {
    kind: BackupProviderKind,
    backup: EncryptedBackup,
}

#[derive(Default)]
struct BackupRuntime {
    sessions: HashMap<BackupProviderKind, ProviderSession>,
    previews: HashMap<String, PendingRestore>,
}

#[derive(Clone)]
pub struct CloudBackupController {
    repository: SqliteNoteRepository,
    sync: CloudSyncController,
    keys: Arc<dyn KeyStore>,
    configuration: BackupConfiguration,
    google: GoogleDriveProvider,
    onedrive: OneDriveProvider,
    runtime: Arc<Mutex<BackupRuntime>>,
}

impl CloudBackupController {
    pub fn new(
        repository: SqliteNoteRepository,
        sync: CloudSyncController,
        keys: Arc<dyn KeyStore>,
        configuration: BackupConfiguration,
    ) -> Result<Self, CloudBackupError> {
        Ok(Self {
            repository,
            sync,
            keys,
            configuration,
            google: GoogleDriveProvider::new()?,
            onedrive: OneDriveProvider::new()?,
            runtime: Arc::new(Mutex::new(BackupRuntime::default())),
        })
    }

    #[doc(hidden)]
    pub fn for_test(
        repository: SqliteNoteRepository,
        sync: CloudSyncController,
        keys: Arc<dyn KeyStore>,
        configuration: BackupConfiguration,
        google: GoogleDriveProvider,
        onedrive: OneDriveProvider,
    ) -> Self {
        Self {
            repository,
            sync,
            keys,
            configuration,
            google,
            onedrive,
            runtime: Arc::new(Mutex::new(BackupRuntime::default())),
        }
    }

    pub fn available(&self, kind: BackupProviderKind) -> bool {
        self.configuration.is_available(kind)
    }

    pub fn authorization(&self, kind: BackupProviderKind) -> Result<OAuthPkce, CloudBackupError> {
        Ok(self
            .configuration
            .oauth(kind)
            .ok_or(CloudBackupError::NotConfigured)?
            .authorization())
    }

    pub async fn connect(
        &self,
        kind: BackupProviderKind,
        code: &str,
        verifier: &str,
    ) -> Result<(), CloudBackupError> {
        let oauth = self
            .configuration
            .oauth(kind)
            .ok_or(CloudBackupError::NotConfigured)?;
        let session = oauth.exchange(code, verifier).await?;
        self.store_session(kind, &session).await?;
        self.runtime.lock().await.sessions.insert(kind, session);
        Ok(())
    }

    pub async fn restore_connections(&self) -> Vec<ProviderBackupResult> {
        let mut results = Vec::new();
        for kind in [
            BackupProviderKind::GoogleDrive,
            BackupProviderKind::OneDrive,
        ] {
            if !self.available(kind) {
                continue;
            }
            let result = self.restore_connection(kind).await;
            results.push(ProviderBackupResult {
                provider: kind,
                uploaded: false,
                message: match result {
                    Ok(true) => "Connected".into(),
                    Ok(false) => "Not connected".into(),
                    Err(error) => format!("Could not restore connection: {error}"),
                },
            });
        }
        results
    }

    async fn restore_connection(&self, kind: BackupProviderKind) -> Result<bool, CloudBackupError> {
        let Some(stored) = self.keys.get(secret_kind(kind), SESSION_ACCOUNT).await? else {
            return Ok(false);
        };
        let stored: StoredProviderSession =
            serde_json::from_slice(&stored).map_err(|_| CloudBackupError::InvalidSession)?;
        let refresh = Zeroizing::new(stored.refresh_token);
        let session = self
            .configuration
            .oauth(kind)
            .ok_or(CloudBackupError::NotConfigured)?
            .refresh(&refresh)
            .await?;
        self.store_session(kind, &session).await?;
        self.runtime.lock().await.sessions.insert(kind, session);
        Ok(true)
    }

    pub async fn connected(&self, kind: BackupProviderKind) -> bool {
        self.runtime.lock().await.sessions.contains_key(&kind)
    }

    pub async fn backup_now(&self, device_id: &str) -> Vec<ProviderBackupResult> {
        let vault = match self.sync.unlocked_vault().await {
            Ok(vault) => vault,
            Err(error) => {
                return connected_kinds(&*self.runtime.lock().await)
                    .into_iter()
                    .map(|provider| ProviderBackupResult {
                        provider,
                        uploaded: false,
                        message: format!("Encrypted vault is locked: {error}"),
                    })
                    .collect();
            }
        };
        let notes = match self
            .repository
            .search_notes_sorted("", NoteSort::UpdatedDesc)
            .await
        {
            Ok(notes) => notes,
            Err(error) => {
                return connected_kinds(&*self.runtime.lock().await)
                    .into_iter()
                    .map(|provider| ProviderBackupResult {
                        provider,
                        uploaded: false,
                        message: format!("Could not read local notes: {error}"),
                    })
                    .collect();
            }
        };
        let now = Utc::now();
        let backup = match BackupArchive::create(&vault, now, device_id, notes) {
            Ok(backup) => backup,
            Err(error) => {
                return connected_kinds(&*self.runtime.lock().await)
                    .into_iter()
                    .map(|provider| ProviderBackupResult {
                        provider,
                        uploaded: false,
                        message: format!("Could not encrypt backup: {error}"),
                    })
                    .collect();
            }
        };
        let encrypted = match serde_json::to_vec(&backup) {
            Ok(bytes) => bytes,
            Err(_) => return Vec::new(),
        };
        let sessions = self.runtime.lock().await.sessions.clone();
        let mut results = Vec::new();
        for (kind, session) in sessions {
            let outcome = self
                .upload_pair(kind, &session.access_token, now, &encrypted)
                .await;
            results.push(ProviderBackupResult {
                provider: kind,
                uploaded: outcome.is_ok(),
                message: outcome
                    .map(|_| "Encrypted backup uploaded".into())
                    .unwrap_or_else(|error| format!("Backup failed: {error}")),
            });
        }
        results
    }

    async fn upload_pair(
        &self,
        kind: BackupProviderKind,
        access_token: &str,
        now: DateTime<Utc>,
        encrypted: &[u8],
    ) -> Result<(), CloudBackupError> {
        let timestamped = format!("noor-notes-{}.nnbackup", now.format("%Y%m%d-%H%M%S"));
        self.provider_upload(kind, access_token, &timestamped, encrypted)
            .await?;
        self.provider_upload(kind, access_token, CURRENT_BACKUP, encrypted)
            .await?;
        Ok(())
    }

    pub async fn list_backups(
        &self,
        kind: BackupProviderKind,
    ) -> Result<Vec<BackupObject>, CloudBackupError> {
        let session = self.session(kind).await?;
        self.provider_list(kind, &session.access_token)
            .await
            .map_err(Into::into)
    }

    pub async fn preview_restore(
        &self,
        kind: BackupProviderKind,
        object: BackupObject,
    ) -> Result<RestorePreview, CloudBackupError> {
        let session = self.session(kind).await?;
        let bytes = self
            .provider_download(kind, &session.access_token, &object)
            .await?;
        let backup: EncryptedBackup =
            serde_json::from_slice(&bytes).map_err(|_| BackupArchiveError::Malformed)?;
        let vault = self.sync.unlocked_vault().await?;
        let archive = BackupArchive::preview(&vault, &backup)?;
        let token = random_token();
        self.runtime
            .lock()
            .await
            .previews
            .insert(token.clone(), PendingRestore { kind, backup });
        Ok(RestorePreview {
            token,
            provider: kind,
            object,
            archive,
        })
    }

    pub async fn restore(&self, token: &str) -> Result<RestoreReport, CloudBackupError> {
        let pending = self
            .runtime
            .lock()
            .await
            .previews
            .remove(token)
            .ok_or(CloudBackupError::RestoreNotConfirmed)?;
        let vault = self.sync.unlocked_vault().await?;
        let notes = BackupArchive::decrypt(&vault, &pending.backup)?;
        let mut report = RestoreReport {
            applied: 0,
            conflicts: 0,
            ignored: 0,
        };
        for note in notes {
            let local = self.repository.get_note(note.id).await?;
            match merge_remote_revision(
                local.as_ref(),
                note,
                provider_name(pending.kind),
                Utc::now(),
            ) {
                MergeOutcome::Apply(note) => {
                    self.repository.save_remote_note(&note).await?;
                    report.applied += 1;
                }
                MergeOutcome::ConflictCopy(note) => {
                    self.repository.save_note(&note).await?;
                    report.conflicts += 1;
                }
                MergeOutcome::Ignore => report.ignored += 1,
            }
        }
        Ok(report)
    }

    pub async fn disconnect(&self, kind: BackupProviderKind) -> Result<(), CloudBackupError> {
        self.runtime.lock().await.sessions.remove(&kind);
        self.keys.delete(secret_kind(kind), SESSION_ACCOUNT).await?;
        Ok(())
    }

    async fn session(&self, kind: BackupProviderKind) -> Result<ProviderSession, CloudBackupError> {
        self.runtime
            .lock()
            .await
            .sessions
            .get(&kind)
            .cloned()
            .ok_or(CloudBackupError::NotConnected)
    }

    async fn store_session(
        &self,
        kind: BackupProviderKind,
        session: &ProviderSession,
    ) -> Result<(), CloudBackupError> {
        let stored = StoredProviderSession {
            refresh_token: session.refresh_token.to_string(),
        };
        let encoded = Zeroizing::new(
            serde_json::to_vec(&stored).map_err(|_| CloudBackupError::InvalidSession)?,
        );
        self.keys
            .put(secret_kind(kind), SESSION_ACCOUNT, &encoded)
            .await?;
        Ok(())
    }

    async fn provider_upload(
        &self,
        kind: BackupProviderKind,
        access_token: &str,
        name: &str,
        bytes: &[u8],
    ) -> Result<BackupObject, BackupProviderError> {
        match kind {
            BackupProviderKind::GoogleDrive => self.google.upload(access_token, name, bytes).await,
            BackupProviderKind::OneDrive => self.onedrive.upload(access_token, name, bytes).await,
        }
    }

    async fn provider_list(
        &self,
        kind: BackupProviderKind,
        access_token: &str,
    ) -> Result<Vec<BackupObject>, BackupProviderError> {
        match kind {
            BackupProviderKind::GoogleDrive => self.google.list(access_token).await,
            BackupProviderKind::OneDrive => self.onedrive.list(access_token).await,
        }
    }

    async fn provider_download(
        &self,
        kind: BackupProviderKind,
        access_token: &str,
        object: &BackupObject,
    ) -> Result<Vec<u8>, BackupProviderError> {
        match kind {
            BackupProviderKind::GoogleDrive => self.google.download(access_token, object).await,
            BackupProviderKind::OneDrive => self.onedrive.download(access_token, object).await,
        }
    }
}

fn connected_kinds(runtime: &BackupRuntime) -> Vec<BackupProviderKind> {
    runtime.sessions.keys().copied().collect()
}

fn secret_kind(kind: BackupProviderKind) -> SecretKind {
    match kind {
        BackupProviderKind::GoogleDrive => SecretKind::GoogleDriveSession,
        BackupProviderKind::OneDrive => SecretKind::OneDriveSession,
    }
}

fn provider_name(kind: BackupProviderKind) -> &'static str {
    match kind {
        BackupProviderKind::GoogleDrive => "Google Drive backup",
        BackupProviderKind::OneDrive => "OneDrive backup",
    }
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
