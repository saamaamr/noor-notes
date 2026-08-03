use noor_domain::Note;

use crate::{SqliteNoteRepository, StorageError};

impl SqliteNoteRepository {
    pub async fn save_remote_note(&self, note: &Note) -> Result<(), StorageError> {
        self.save_note(note).await?;
        sqlx::query(
            "DELETE FROM change_journal WHERE note_id = ? AND revision = ? AND operation = 'upsert'",
        )
        .bind(note.id.value().to_string())
        .bind(note.revision.value() as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
