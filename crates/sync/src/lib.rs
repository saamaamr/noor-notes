//! Encrypted offline-first synchronization.

mod client;
mod types;

pub use client::{EndpointPolicy, SupabaseClient, SyncClientError};
pub use types::{
    AuthSession, AuthUser, OAuthPkce, RemoteRevision, RemoteVault, RevisionValidationError,
    SignUpOutcome, SyncCursor,
};
mod backoff;
mod merge;
mod remote_worker;
mod worker;

pub use backoff::retry_delay;
pub use merge::{MergeOutcome, merge_remote_revision};
pub use remote_worker::RemoteApplyError;
pub use worker::{SyncCycle, SyncStatus, SyncWorker};
