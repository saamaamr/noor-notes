use std::io::{self, Write};

use chrono::{DateTime, Utc};
use noor_crypto::{CryptoError, Vault};
use noor_domain::Note;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const BACKUP_VERSION: u8 = 1;
pub const MAX_BACKUP_BYTES: usize = 128 * 1024 * 1024;
const AEAD_OVERHEAD: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedBackup {
    pub version: u8,
    pub created_at: DateTime<Utc>,
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupPreview {
    pub created_at: DateTime<Utc>,
    pub note_count: usize,
    pub device_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct BackupPayload {
    device_id: String,
    notes: Vec<Note>,
}

#[derive(Debug, thiserror::Error)]
pub enum BackupArchiveError {
    #[error("backup is larger than 128 MiB")]
    TooLarge,
    #[error("unsupported backup version {0}")]
    UnsupportedVersion(u8),
    #[error("backup is malformed")]
    Malformed,
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

pub struct BackupArchive;

impl BackupArchive {
    pub fn create(
        vault: &Vault,
        created_at: DateTime<Utc>,
        device_id: impl Into<String>,
        notes: Vec<Note>,
    ) -> Result<EncryptedBackup, BackupArchiveError> {
        let payload = BackupPayload {
            device_id: device_id.into(),
            notes,
        };
        let mut plaintext = LimitedWriter::new(MAX_BACKUP_BYTES - AEAD_OVERHEAD);
        serde_json::to_writer(&mut plaintext, &payload).map_err(|error| {
            if plaintext.exceeded {
                BackupArchiveError::TooLarge
            } else {
                let _ = error;
                BackupArchiveError::Malformed
            }
        })?;
        let plaintext = Zeroizing::new(plaintext.bytes);
        let metadata = metadata(BACKUP_VERSION, created_at);
        let (nonce, ciphertext) = vault.encrypt_backup(&plaintext, &metadata)?;
        if ciphertext.len() > MAX_BACKUP_BYTES {
            return Err(BackupArchiveError::TooLarge);
        }
        Ok(EncryptedBackup {
            version: BACKUP_VERSION,
            created_at,
            nonce,
            ciphertext,
        })
    }

    pub fn preview(
        vault: &Vault,
        backup: &EncryptedBackup,
    ) -> Result<BackupPreview, BackupArchiveError> {
        let payload = Self::decrypt_payload(vault, backup)?;
        Ok(BackupPreview {
            created_at: backup.created_at,
            note_count: payload.notes.len(),
            device_id: payload.device_id,
        })
    }

    pub fn decrypt(
        vault: &Vault,
        backup: &EncryptedBackup,
    ) -> Result<Vec<Note>, BackupArchiveError> {
        Ok(Self::decrypt_payload(vault, backup)?.notes)
    }

    fn decrypt_payload(
        vault: &Vault,
        backup: &EncryptedBackup,
    ) -> Result<BackupPayload, BackupArchiveError> {
        if backup.version != BACKUP_VERSION {
            return Err(BackupArchiveError::UnsupportedVersion(backup.version));
        }
        if backup.ciphertext.len() > MAX_BACKUP_BYTES {
            return Err(BackupArchiveError::TooLarge);
        }
        let metadata = metadata(backup.version, backup.created_at);
        let plaintext =
            Zeroizing::new(vault.decrypt_backup(&backup.nonce, &backup.ciphertext, &metadata)?);
        serde_json::from_slice(&plaintext).map_err(|_| BackupArchiveError::Malformed)
    }
}

fn metadata(version: u8, created_at: DateTime<Utc>) -> Vec<u8> {
    let mut metadata = Vec::with_capacity(9);
    metadata.push(version);
    metadata.extend_from_slice(&created_at.timestamp_millis().to_be_bytes());
    metadata
}

struct LimitedWriter {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}

impl LimitedWriter {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            exceeded: false,
        }
    }
}

impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > self.limit {
            self.exceeded = true;
            return Err(io::Error::new(io::ErrorKind::FileTooLarge, "backup limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
