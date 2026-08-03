#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("cannot read Xpad data: {0}")]
    Io(#[from] std::io::Error),
    #[error("Xpad text is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("missing Xpad field: {0}")]
    MissingField(String),
    #[error("invalid integer in Xpad field: {0}")]
    InvalidInteger(String),
    #[error("unsafe Xpad content path: {0}")]
    UnsafeContentPath(String),
    #[error("cannot store imported note: {0}")]
    Storage(#[from] noor_storage::StorageError),
}
