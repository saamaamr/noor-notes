use std::fs;

use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_storage::{DatabaseKey, SqliteNoteRepository};

#[tokio::test]
async fn encrypted_repository_hides_plaintext_and_requires_the_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    let key = DatabaseKey::generate();
    let repo = SqliteNoteRepository::open_encrypted(&path, &key)
        .await
        .unwrap();
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 5, 12, 0, 0).unwrap());
    note.title = "NOOR-PLAINTEXT-SENTINEL".into();
    note.content = "NOOR-PLAINTEXT-SENTINEL".into();
    repo.save_note(&note).await.unwrap();
    repo.close().await;
    let bytes = fs::read(&path).unwrap();
    assert!(!bytes.windows(16).any(|window| window == b"SQLite format 3"));
    assert!(
        !bytes
            .windows(23)
            .any(|window| window == b"NOOR-PLAINTEXT-SENTINEL")
    );
    let reopened = SqliteNoteRepository::open_encrypted(&path, &key)
        .await
        .unwrap();
    assert_eq!(
        reopened.get_note(note.id).await.unwrap().unwrap().content,
        note.content
    );
    reopened.close().await;
    assert!(
        SqliteNoteRepository::open_encrypted(&path, &DatabaseKey::generate())
            .await
            .is_err()
    );
}
