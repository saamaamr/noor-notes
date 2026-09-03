//! Encrypted offline-first synchronization.

mod client;
mod types;

mod backup_archive;
mod backup_provider;
mod google_drive;
mod onedrive;
pub use backup_archive::{
    BACKUP_VERSION, BackupArchive, BackupArchiveError, BackupPreview, EncryptedBackup,
    MAX_BACKUP_BYTES,
};
pub use backup_provider::{BackupObject, BackupProvider, BackupProviderError};
pub use google_drive::GoogleDriveProvider;
pub use onedrive::OneDriveProvider;

pub use client::{EndpointPolicy, SupabaseClient, SyncClientError};
pub use types::{
    AuthSession, AuthUser, OAuthPkce, RemoteRevision, RemoteVault, RevisionValidationError,
    SignUpOutcome, SyncCursor,
};
mod backoff;
mod merge;
mod provider_oauth;
mod remote_worker;
mod worker;

pub use backoff::retry_delay;
pub use merge::{MergeOutcome, merge_remote_revision};
pub use provider_oauth::{BackupProviderKind, ProviderOAuth, ProviderOAuthError, ProviderSession};
pub use remote_worker::RemoteApplyError;
pub use worker::{SyncCycle, SyncStatus, SyncWorker};
