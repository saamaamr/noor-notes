use chrono::{DateTime, Utc};
use noor_domain::{Note, NoteId, NoteState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoteSort {
    #[default]
    UpdatedDesc,
    TitleAsc,
    TitleDesc,
    CreatedDesc,
}

use crate::{SqliteNoteRepository, StorageError};

impl SqliteNoteRepository {
    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>, StorageError> {
        self.search_notes_sorted(query, NoteSort::UpdatedDesc).await
    }

    pub async fn search_notes_sorted(
        &self,
        query: &str,
        sort: NoteSort,
    ) -> Result<Vec<Note>, StorageError> {
        let payloads = sqlx::query_scalar::<_, String>("SELECT payload_json FROM notes")
            .fetch_all(&self.pool)
            .await?;
        let needle = query.trim().to_lowercase();
        let mut notes = payloads
            .into_iter()
            .map(|payload| serde_json::from_str(&payload).map_err(StorageError::from))
            .collect::<Result<Vec<Note>, _>>()?;
        if !needle.is_empty() {
            notes.retain(|note| {
                note.title.to_lowercase().contains(&needle)
                    || note.content.to_lowercase().contains(&needle)
                    || note
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&needle))
            });
        }
        match sort {
            NoteSort::UpdatedDesc => notes.sort_by(|a, b| {
                b.updated_at
                    .cmp(&a.updated_at)
                    .then_with(|| a.id.value().cmp(&b.id.value()))
            }),
            NoteSort::TitleAsc => notes.sort_by(|a, b| {
                a.display_title()
                    .to_lowercase()
                    .cmp(&b.display_title().to_lowercase())
                    .then_with(|| a.id.value().cmp(&b.id.value()))
            }),
            NoteSort::TitleDesc => notes.sort_by(|a, b| {
                b.display_title()
                    .to_lowercase()
                    .cmp(&a.display_title().to_lowercase())
                    .then_with(|| a.id.value().cmp(&b.id.value()))
            }),
            NoteSort::CreatedDesc => notes.sort_by(|a, b| {
                b.created_at
                    .cmp(&a.created_at)
                    .then_with(|| a.id.value().cmp(&b.id.value()))
            }),
        }
        Ok(notes)
    }

    pub async fn duplicate_note(
        &self,
        id: NoteId,
        now: DateTime<Utc>,
    ) -> Result<Note, StorageError> {
        let original = self
            .get_note(id)
            .await?
            .ok_or(StorageError::NoteNotFound(id))?;
        let copy = original.duplicate(now);
        self.save_note(&copy).await?;
        Ok(copy)
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
