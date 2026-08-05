use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use noor_domain::{Note, NoteId};
use noor_notes::autosave::{AutosaveQueue, NoteDraft, NoteSaver};
use noor_notes::save_status::SaveState;
use noor_storage::StorageError;

struct ControlledSaver {
    fail: AtomicBool,
}

#[async_trait]
impl NoteSaver for ControlledSaver {
    async fn save_note(&self, note: &Note) -> Result<(), StorageError> {
        if self.fail.load(Ordering::SeqCst) {
            Err(StorageError::NoteNotFound(note.id))
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn failed_save_is_visible_and_retryable_without_losing_draft() {
    tokio::time::pause();
    let saver = Arc::new(ControlledSaver {
        fail: AtomicBool::new(true),
    });
    let queue = AutosaveQueue::with_saver(saver.clone(), Duration::from_millis(20));
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 5, 9, 0, 0).unwrap());
    note.content = "must survive".into();
    let mut state = queue.subscribe(note.id);
    queue.schedule(NoteDraft::from(note.clone()));
    assert_eq!(*state.borrow_and_update(), SaveState::Saving);
    tokio::time::advance(Duration::from_millis(20)).await;
    tokio::task::yield_now().await;
    state.changed().await.unwrap();
    assert!(
        matches!(&*state.borrow(), SaveState::Failed(message) if message.contains("note not found"))
    );
    assert!(queue.has_pending(note.id));
    saver.fail.store(false, Ordering::SeqCst);
    queue.retry(note.id).await.unwrap();
    assert_eq!(*state.borrow_and_update(), SaveState::Saved);
    assert!(!queue.has_pending(note.id));
}

#[test]
fn subscribing_before_edits_starts_idle() {
    let saver = Arc::new(ControlledSaver {
        fail: AtomicBool::new(false),
    });
    let queue = AutosaveQueue::with_saver(saver, Duration::from_secs(1));
    let state = queue.subscribe(NoteId::new());
    assert_eq!(*state.borrow(), SaveState::Idle);
}
