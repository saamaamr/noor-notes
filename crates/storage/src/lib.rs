//! Offline SQLite persistence for Noor Notes.

mod backup;
mod encrypted_open;
mod error;
mod import_receipts;
mod journal;
mod lifecycle;
mod permissions;
mod remote;
mod repository;

pub use encrypted_open::DatabaseKey;
pub use error::StorageError;
pub use journal::PendingChange;
pub use lifecycle::NoteSort;
pub use repository::SqliteNoteRepository;
