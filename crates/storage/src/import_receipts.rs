use chrono::Utc;
use noor_domain::NoteId;

use crate::{SqliteNoteRepository, StorageError};

impl SqliteNoteRepository {
    pub async fn has_import_receipt(&self, source_key: &str) -> Result<bool, StorageError> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM import_receipts WHERE source_key = ?)",
        )
        .bind(source_key)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists != 0)
    }

    pub async fn record_import_receipt(
        &self,
        source_key: &str,
        note_id: NoteId,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT OR IGNORE INTO import_receipts (source_key, imported_note_id, imported_at)
             VALUES (?, ?, ?)",
        )
        .bind(source_key)
        .bind(note_id.value().to_string())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
