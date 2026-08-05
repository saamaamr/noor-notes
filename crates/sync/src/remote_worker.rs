use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use noor_crypto::EncryptedNote;
use noor_domain::{Note, NoteId, Revision};

use crate::{MergeOutcome, RemoteRevision, SyncWorker, merge_remote_revision};

#[derive(Debug, thiserror::Error)]
pub enum RemoteApplyError {
    #[error("remote envelope encoding is invalid")]
    InvalidEncoding,
    #[error("remote envelope failed authentication")]
    AuthenticationFailed,
    #[error("remote note payload is invalid")]
    InvalidPayload,
    #[error("local storage rejected remote note: {0}")]
    Storage(#[from] noor_storage::StorageError),
}

impl SyncWorker {
    pub async fn apply_remote_revision(
        &self,
        remote: RemoteRevision,
        remote_device: &str,
        now: DateTime<Utc>,
    ) -> Result<(), RemoteApplyError> {
        remote
            .validate(now)
            .map_err(|_| RemoteApplyError::InvalidEncoding)?;
        let nonce: [u8; 24] = STANDARD
            .decode(remote.nonce)
            .map_err(|_| RemoteApplyError::InvalidEncoding)?
            .try_into()
            .map_err(|_| RemoteApplyError::InvalidEncoding)?;
        let ciphertext = STANDARD
            .decode(remote.ciphertext)
            .map_err(|_| RemoteApplyError::InvalidEncoding)?;
        let note_id = NoteId::from_uuid(remote.note_id);
        let revision = Revision::from_value(remote.revision);
        let envelope = EncryptedNote {
            version: 1,
            note_id,
            revision,
            nonce,
            ciphertext,
        };
        let plaintext = self
            .vault
            .decrypt_note(&envelope)
            .map_err(|_| RemoteApplyError::AuthenticationFailed)?;
        let note: Note =
            serde_json::from_slice(&plaintext).map_err(|_| RemoteApplyError::InvalidPayload)?;
        if note.id != note_id || note.revision != revision {
            return Err(RemoteApplyError::AuthenticationFailed);
        }

        let local = self.repository.get_note(note.id).await?;
        match merge_remote_revision(local.as_ref(), note, remote_device, now) {
            MergeOutcome::Apply(note) => self.repository.save_remote_note(&note).await?,
            MergeOutcome::ConflictCopy(note) => self.repository.save_note(&note).await?,
            MergeOutcome::Ignore => {}
        }
        Ok(())
    }
}
