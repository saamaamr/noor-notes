use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_notes::autosave::{AutosaveQueue, NoteDraft, NoteSaver};
use noor_storage::{SqliteNoteRepository, StorageError};
use tokio::sync::Notify;

struct BlockingSaver {
    started: Notify,
    release: Notify,
    calls: AtomicUsize,
}

#[async_trait]
impl NoteSaver for BlockingSaver {
    async fn save_note(&self, _note: &Note) -> Result<(), StorageError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        self.release.notified().await;
        Ok(())
    }
}

#[tokio::test]
async fn rapid_edits_are_debounced_to_the_latest_content() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    tokio::time::pause();
    let queue = AutosaveQueue::new(repo.clone(), Duration::from_millis(400));
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap());
    note.content = "first".into();
    queue.schedule(NoteDraft::from(note.clone()));
    tokio::task::yield_now().await;
    note.content = "final".into();
    queue.schedule(NoteDraft::from(note.clone()));
    tokio::task::yield_now().await;

    tokio::time::advance(Duration::from_millis(400)).await;
    tokio::time::resume();
    tokio::time::sleep(Duration::from_millis(20)).await;
    tokio::task::yield_now().await;
    assert_eq!(
        repo.get_note(note.id).await.unwrap().unwrap().content,
        "final"
    );
    assert!(!queue.has_pending(note.id));
}

#[tokio::test]
async fn flush_saves_immediately_without_waiting_for_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    tokio::time::pause();
    let queue = AutosaveQueue::new(repo.clone(), Duration::from_secs(30));
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap());
    note.content = "close-safe".into();
    queue.schedule(NoteDraft::from(note.clone()));
    tokio::task::yield_now().await;

    tokio::time::resume();
    tokio::time::sleep(Duration::from_millis(20)).await;
    queue.flush(note.id).await.unwrap();

    assert_eq!(
        repo.get_note(note.id).await.unwrap().unwrap().content,
        "close-safe"
    );
}

#[tokio::test]
async fn concurrent_flushes_for_one_note_wait_for_the_in_flight_save() {
    let saver = Arc::new(BlockingSaver {
        started: Notify::new(),
        release: Notify::new(),
        calls: AtomicUsize::new(0),
    });
    let queue = AutosaveQueue::with_saver(saver.clone(), Duration::from_secs(30));
    let note = Note::new(Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap());
    queue.schedule(NoteDraft::from(note.clone()));

    let first = {
        let queue = queue.clone();
        tokio::spawn(async move { queue.flush(note.id).await })
    };
    saver.started.notified().await;
    let second = {
        let queue = queue.clone();
        tokio::spawn(async move { queue.flush(note.id).await })
    };
    tokio::task::yield_now().await;
    assert!(
        !second.is_finished(),
        "second flush bypassed the active save"
    );

    saver.release.notify_waiters();
    first.await.unwrap().unwrap();
    second.await.unwrap().unwrap();
    assert_eq!(saver.calls.load(Ordering::SeqCst), 1);
}
