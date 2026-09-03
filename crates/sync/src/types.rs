use chrono::{DateTime, Utc};
use noor_crypto::{RecoveryWrappedVault, WrappedVault};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroizing;

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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: AuthUser,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SignUpOutcome {
    pub user: AuthUser,
    pub session: Option<AuthSession>,
    pub confirmation_required: bool,
}

pub struct OAuthPkce {
    pub authorization_url: Url,
    pub verifier: Zeroizing<String>,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteVault {
    pub wrapped_vault: WrappedVault,
    pub recovery_wrapped_vault: RecoveryWrappedVault,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCursor {
    pub updated_at: DateTime<Utc>,
    pub note_id: Uuid,
    pub revision: u64,
}

impl SyncCursor {
    pub fn from_revision(revision: &RemoteRevision) -> Self {
        Self {
            updated_at: revision.updated_at,
            note_id: revision.note_id,
            revision: revision.revision,
        }
    }

    pub fn is_before(&self, revision: &RemoteRevision) -> bool {
        (self.updated_at, self.note_id, self.revision)
            < (revision.updated_at, revision.note_id, revision.revision)
    }
}

impl Default for SyncCursor {
    fn default() -> Self {
        Self {
            updated_at: DateTime::from_timestamp(0, 0).expect("Unix epoch is valid"),
            note_id: Uuid::nil(),
            revision: 0,
        }
    }
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
