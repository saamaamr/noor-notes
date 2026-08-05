use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use noor_domain::{Note, NoteId};
use noor_storage::{SqliteNoteRepository, StorageError};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::save_status::SaveState;

#[derive(Clone, Debug)]
pub struct NoteDraft {
    pub note: Note,
}

impl From<Note> for NoteDraft {
    fn from(note: Note) -> Self {
        Self { note }
    }
}

#[async_trait]
pub trait NoteSaver: Send + Sync {
    async fn save_note(&self, note: &Note) -> Result<(), StorageError>;
}

#[async_trait]
impl NoteSaver for SqliteNoteRepository {
    async fn save_note(&self, note: &Note) -> Result<(), StorageError> {
        SqliteNoteRepository::save_note(self, note).await
    }
}

struct PendingSave {
    draft: NoteDraft,
    generation: u64,
    task: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct AutosaveQueue {
    saver: Arc<dyn NoteSaver>,
    delay: Duration,
    pending: Arc<Mutex<HashMap<NoteId, PendingSave>>>,
    states: Arc<Mutex<HashMap<NoteId, watch::Sender<SaveState>>>>,
    generation: Arc<AtomicU64>,
}

impl AutosaveQueue {
    pub fn new(repository: SqliteNoteRepository, delay: Duration) -> Self {
        Self::with_saver(Arc::new(repository), delay)
    }

    pub fn with_saver(saver: Arc<dyn NoteSaver>, delay: Duration) -> Self {
        Self {
            saver,
            delay,
            pending: Arc::new(Mutex::new(HashMap::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn subscribe(&self, id: NoteId) -> watch::Receiver<SaveState> {
        let mut states = self.states.lock().expect("save state mutex poisoned");
        states
            .entry(id)
            .or_insert_with(|| watch::channel(SaveState::Idle).0)
            .subscribe()
    }

    pub fn has_pending(&self, id: NoteId) -> bool {
        self.pending
            .lock()
            .expect("autosave queue mutex poisoned")
            .contains_key(&id)
    }

    fn publish(&self, id: NoteId, state: SaveState) {
        let mut states = self.states.lock().expect("save state mutex poisoned");
        let sender = states
            .entry(id)
            .or_insert_with(|| watch::channel(SaveState::Idle).0);
        sender.send_replace(state);
    }

    pub fn schedule(&self, draft: NoteDraft) {
        let id = draft.note.id;
        self.publish(id, SaveState::Saving);
        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        let saver = self.saver.clone();
        let save_draft = draft.clone();
        let delay = self.delay;
        let states = self.states.clone();
        let pending_saves = self.pending.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let state = match saver.save_note(&save_draft.note).await {
                Ok(()) => SaveState::Saved,
                Err(error) => SaveState::Failed(error.to_string()),
            };
            let succeeded = matches!(state, SaveState::Saved);
            if let Some(sender) = states.lock().expect("save state mutex poisoned").get(&id) {
                sender.send_replace(state);
            }
            let mut pending = pending_saves.lock().expect("autosave queue mutex poisoned");
            if pending
                .get(&id)
                .is_some_and(|entry| entry.generation == generation)
            {
                if succeeded {
                    pending.remove(&id);
                } else if let Some(entry) = pending.get_mut(&id) {
                    entry.task = None;
                }
            }
        });
        let mut pending = self.pending.lock().expect("autosave queue mutex poisoned");
        if let Some(mut previous) = pending.insert(
            id,
            PendingSave {
                draft,
                generation,
                task: Some(task),
            },
        ) {
            if let Some(task) = previous.task.take() {
                task.abort();
            }
        }
    }

    pub async fn flush(&self, id: NoteId) -> Result<(), StorageError> {
        let pending = self
            .pending
            .lock()
            .expect("autosave queue mutex poisoned")
            .remove(&id);
        let Some(mut pending) = pending else {
            return Ok(());
        };
        if let Some(task) = pending.task.take() {
            task.abort();
        }
        self.publish(id, SaveState::Saving);
        match self.saver.save_note(&pending.draft.note).await {
            Ok(()) => {
                self.publish(id, SaveState::Saved);
                Ok(())
            }
            Err(error) => {
                self.publish(id, SaveState::Failed(error.to_string()));
                self.pending
                    .lock()
                    .expect("autosave queue mutex poisoned")
                    .insert(id, pending);
                Err(error)
            }
        }
    }

    pub async fn retry(&self, id: NoteId) -> Result<(), StorageError> {
        self.flush(id).await
    }
}
