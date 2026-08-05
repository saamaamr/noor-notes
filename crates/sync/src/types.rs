use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_CIPHERTEXT_BASE64_BYTES: usize = 5_592_408;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RevisionValidationError {
    #[error("remote revision payload exceeds the security limit")]
    TooLarge,
    #[error("remote revision nonce is malformed")]
    InvalidNonce,
    #[error("remote revision timestamp is outside the accepted range")]
    InvalidTimestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteRevision {
    pub note_id: Uuid,
    pub revision: u64,
    pub ciphertext: String,
    pub nonce: String,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl RemoteRevision {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), RevisionValidationError> {
        if self.ciphertext.len() > MAX_CIPHERTEXT_BASE64_BYTES {
            return Err(RevisionValidationError::TooLarge);
        }
        if self.nonce.len() != 32 {
            return Err(RevisionValidationError::InvalidNonce);
        }
        let latest = now + chrono::Duration::minutes(5);
        if self.updated_at > latest || self.deleted_at.is_some_and(|value| value > latest) {
            return Err(RevisionValidationError::InvalidTimestamp);
        }
        Ok(())
    }
}
