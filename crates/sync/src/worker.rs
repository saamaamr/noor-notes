use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use noor_crypto::Vault;
use noor_domain::Note;
use noor_storage::SqliteNoteRepository;

use crate::{RemoteRevision, SupabaseClient, SyncClientError, SyncCursor};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Offline,
    AuthRequired,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncCycle {
    pub status: SyncStatus,
    pub cursor: SyncCursor,
    pub uploaded: usize,
    pub downloaded: usize,
}

pub struct SyncWorker {
    pub(crate) repository: SqliteNoteRepository,
    client: SupabaseClient,
    pub(crate) vault: Arc<Vault>,
    access_token: String,
}

impl SyncWorker {
    pub fn new(
        repository: SqliteNoteRepository,
        client: SupabaseClient,
        vault: Arc<Vault>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            repository,
            client,
            vault,
            access_token: access_token.into(),
        }
    }

    pub async fn run_once(&self) -> SyncStatus {
        let pending = match self.repository.pending_changes(100).await {
            Ok(pending) => pending,
            Err(_) => return SyncStatus::Error,
        };
        for change in pending {
            let note: Note = match serde_json::from_str(&change.payload_json) {
                Ok(note) => note,
                Err(_) => return SyncStatus::Error,
            };
            let plaintext = match serde_json::to_vec(&note) {
                Ok(plaintext) => plaintext,
                Err(_) => return SyncStatus::Error,
            };
            let envelope = match self.vault.encrypt_note(note.id, note.revision, &plaintext) {
                Ok(envelope) => envelope,
                Err(_) => return SyncStatus::Error,
            };
            let remote = RemoteRevision {
                note_id: note.id.value(),
                revision: note.revision.value(),
                ciphertext: STANDARD.encode(envelope.ciphertext),
                nonce: STANDARD.encode(envelope.nonce),
                updated_at: note.updated_at,
                deleted_at: match note.state {
                    noor_domain::NoteState::Trashed { deleted_at } => Some(deleted_at),
                    _ => None,
                },
            };
            match self
                .client
                .upload_revision(&self.access_token, &remote)
                .await
            {
                Ok(()) => {
                    if self.repository.ack_change(change.id).await.is_err() {
                        return SyncStatus::Error;
                    }
                }
                Err(SyncClientError::AuthRequired) => return SyncStatus::AuthRequired,
                Err(SyncClientError::Transport(_))
                | Err(SyncClientError::Http(reqwest::StatusCode::SERVICE_UNAVAILABLE))
                | Err(SyncClientError::RateLimited(_)) => return SyncStatus::Offline,
                Err(_) => return SyncStatus::Error,
            }
        }
        SyncStatus::Idle
    }
    pub async fn run_cycle(
        &self,
        mut cursor: SyncCursor,
        remote_device: &str,
        now: DateTime<Utc>,
    ) -> SyncCycle {
        let pending = match self.repository.pending_changes(100).await {
            Ok(pending) => pending.len(),
            Err(_) => {
                return SyncCycle {
                    status: SyncStatus::Error,
                    cursor,
                    uploaded: 0,
                    downloaded: 0,
                };
            }
        };
        let upload_status = self.run_once().await;
        if upload_status != SyncStatus::Idle {
            return SyncCycle {
                status: upload_status,
                cursor,
                uploaded: 0,
                downloaded: 0,
            };
        }
        let revisions = match self
            .client
            .list_changes_after(&self.access_token, cursor)
            .await
        {
            Ok(revisions) => revisions,
            Err(error) => {
                let status = match error {
                    SyncClientError::AuthRequired => SyncStatus::AuthRequired,
                    SyncClientError::Transport(_)
                    | SyncClientError::Http(reqwest::StatusCode::SERVICE_UNAVAILABLE)
                    | SyncClientError::RateLimited(_) => SyncStatus::Offline,
                    _ => SyncStatus::Error,
                };
                return SyncCycle {
                    status,
                    cursor,
                    uploaded: pending,
                    downloaded: 0,
                };
            }
        };
        let mut downloaded = 0;
        for revision in revisions {
            let next_cursor = SyncCursor::from_revision(&revision);
            if self
                .apply_remote_revision(revision, remote_device, now)
                .await
                .is_err()
            {
                return SyncCycle {
                    status: SyncStatus::Error,
                    cursor,
                    uploaded: pending,
                    downloaded,
                };
            }
            cursor = next_cursor;
            downloaded += 1;
        }
        SyncCycle {
            status: SyncStatus::Idle,
            cursor,
            uploaded: pending,
            downloaded,
        }
    }
}
