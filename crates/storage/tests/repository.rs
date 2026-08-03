use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_storage::SqliteNoteRepository;

#[tokio::test]
async fn save_survives_reopen_and_creates_one_pending_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    let repo = SqliteNoteRepository::open(&path).await.unwrap();
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap());
    note.content = "offline text".into();

    repo.save_note(&note).await.unwrap();
    drop(repo);

    let reopened = SqliteNoteRepository::open(&path).await.unwrap();
    let stored = reopened.get_note(note.id).await.unwrap().unwrap();
    assert_eq!(stored.content, "offline text");
    let pending = reopened.pending_changes(10).await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].note_id, note.id);
}
