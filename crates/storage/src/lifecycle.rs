use chrono::{DateTime, Utc};
use noor_domain::{Note, NoteId, NoteState};

use crate::{SqliteNoteRepository, StorageError};

impl SqliteNoteRepository {
    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>, StorageError> {
        let pattern = format!("%{query}%");
        let payloads = sqlx::query_scalar::<_, String>(
            "SELECT payload_json FROM notes WHERE content LIKE ? ORDER BY updated_at DESC",
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await?;

        payloads
            .into_iter()
            .map(|payload| serde_json::from_str(&payload).map_err(StorageError::from))
            .collect()
    }

    pub async fn archive(&self, id: NoteId, now: DateTime<Utc>) -> Result<(), StorageError> {
        self.set_state(id, NoteState::Archived, now).await
    }

    pub async fn trash(&self, id: NoteId, now: DateTime<Utc>) -> Result<(), StorageError> {
        self.set_state(id, NoteState::Trashed { deleted_at: now }, now)
            .await
    }

    pub async fn restore(&self, id: NoteId, now: DateTime<Utc>) -> Result<(), StorageError> {
        self.set_state(id, NoteState::Active, now).await
    }

    async fn set_state(
        &self,
        id: NoteId,
        state: NoteState,
        now: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let mut note = self
            .get_note(id)
            .await?
            .ok_or(StorageError::NoteNotFound(id))?;
        note.state = state;
        note.revision = note.revision.next();
        note.updated_at = now;
        self.save_note(&note).await
    }
}
