use std::path::Path;
use std::sync::Arc;

use noor_storage::{
    DatabaseFormat, DatabaseKey, SqliteNoteRepository, StorageError, detect_database_format,
    migrate_or_open,
};

use crate::key_store::{KeyStore, KeyStoreError, SecretKind};

const LOCAL_ACCOUNT: &str = "local-default";

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error(transparent)]
    KeyStore(#[from] KeyStoreError),
    #[error(
        "the local encryption key is missing; restore the GNOME Keyring item before opening this database"
    )]
    DatabaseKeyMissing,
    #[error(transparent)]
    Storage(#[from] StorageError),
}

pub async fn open_repository(
    path: &Path,
    keys: Arc<dyn KeyStore>,
) -> Result<SqliteNoteRepository, BootstrapError> {
    let format = detect_database_format(path)?;
    let stored = keys.get(SecretKind::DatabaseKey, LOCAL_ACCOUNT).await?;
    let key = match stored {
        Some(bytes) => DatabaseKey::try_from_slice(&bytes)?,
        None if format == DatabaseFormat::Encrypted => {
            return Err(BootstrapError::DatabaseKeyMissing);
        }
        None => {
            let key = DatabaseKey::generate();
            keys.put(SecretKind::DatabaseKey, LOCAL_ACCOUNT, key.as_bytes())
                .await?;
            key
        }
    };
    Ok(migrate_or_open(path, &key).await?)
}
