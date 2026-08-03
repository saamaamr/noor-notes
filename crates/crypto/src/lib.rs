//! End-to-end encryption primitives for Noor Notes.

mod envelope;
mod error;
mod recovery;
mod vault;

pub use envelope::{EncryptedNote, RecoveryWrappedVault, WrappedVault};
pub use error::CryptoError;
pub use recovery::RecoveryKey;
pub use vault::Vault;
