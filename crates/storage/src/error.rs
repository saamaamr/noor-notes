use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database encryption key is invalid")]
    InvalidDatabaseKey,
    #[error("database path is not a regular file: {0}")]
    UnsafeDatabasePath(PathBuf),
    #[error("cannot back up corrupt database {path} to {backup}: {source}")]
    BackupCorruptDatabase {
        path: PathBuf,
        backup: PathBuf,
        source: std::io::Error,
    },
    #[error("note not found: {0:?}")]
    NoteNotFound(noor_domain::NoteId),
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    #[error("stored note data is invalid: {0}")]
    InvalidData(#[from] serde_json::Error),
    #[error("stored identifier is invalid: {0}")]
    InvalidIdentifier(#[from] uuid::Error),
    #[error("cannot prepare database directory {path}: {source}")]
    PrepareDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot secure database file {path}: {source}")]
    SecureFile {
        path: PathBuf,
        source: std::io::Error,
    },
}
