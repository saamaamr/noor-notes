use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use noor_domain::{Note, NoteId};
use noor_storage::{SqliteNoteRepository, StorageError};
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub struct NoteDraft {
    pub note: Note,
}

impl From<Note> for NoteDraft {
    fn from(note: Note) -> Self {
        Self { note }
    }
}

struct PendingSave {
    draft: NoteDraft,
    task: JoinHandle<()>,
}

#[derive(Clone)]
pub struct AutosaveQueue {
    repository: SqliteNoteRepository,
    delay: Duration,
    pending: Arc<Mutex<HashMap<NoteId, PendingSave>>>,
}

impl AutosaveQueue {
    pub fn new(repository: SqliteNoteRepository, delay: Duration) -> Self {
        Self {
            repository,
            delay,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn schedule(&self, draft: NoteDraft) {
        let id = draft.note.id;
        let repository = self.repository.clone();
        let save_draft = draft.clone();
        let delay = self.delay;
        let task = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = repository.save_note(&save_draft.note).await;
        });

        let mut pending = self.pending.lock().expect("autosave queue mutex poisoned");
        if let Some(previous) = pending.insert(id, PendingSave { draft, task }) {
            previous.task.abort();
        }
    }

    pub async fn flush(&self, id: NoteId) -> Result<(), StorageError> {
        let pending = self
            .pending
            .lock()
            .expect("autosave queue mutex poisoned")
            .remove(&id);
        if let Some(pending) = pending {
            pending.task.abort();
            self.repository.save_note(&pending.draft.note).await?;
        }
        Ok(())
    }
}
