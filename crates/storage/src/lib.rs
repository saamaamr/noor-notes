//! Offline SQLite persistence for Noor Notes.

mod backup;
mod encrypted_open;
mod error;
mod import_receipts;
mod journal;
mod lifecycle;
mod migration;
mod permissions;
mod remote;
mod repository;
mod writing_assistance;

pub use encrypted_open::DatabaseKey;
pub use error::StorageError;
pub use journal::PendingChange;
pub use lifecycle::NoteSort;
pub use migration::{DatabaseFormat, detect_database_format, migrate_or_open};
pub use repository::SqliteNoteRepository;
pub use writing_assistance::{PredictionCorpus, PredictionModelRecord};
