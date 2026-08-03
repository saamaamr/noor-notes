use std::time::Duration;

use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_notes::autosave::{AutosaveQueue, NoteDraft};
use noor_storage::SqliteNoteRepository;

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
