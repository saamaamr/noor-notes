//! Offline SQLite persistence for Noor Notes.

mod backup;
mod error;
mod import_receipts;
mod journal;
mod lifecycle;
mod remote;
mod repository;

pub use error::StorageError;
pub use journal::PendingChange;
pub use repository::SqliteNoteRepository;
