use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_storage::{DatabaseKey, SqliteNoteRepository, migrate_or_open};

#[tokio::test]
async fn plaintext_database_migrates_once_without_losing_notes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    let legacy = SqliteNoteRepository::open(&path).await.unwrap();
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 5, 12, 0, 0).unwrap());
    note.title = "Migrated title".into();
    note.content = "migration secret".into();
    note.tags = vec!["secure".into()];
    legacy.save_note(&note).await.unwrap();
    legacy.close().await;
    assert!(
        std::fs::read(&path)
            .unwrap()
            .starts_with(b"SQLite format 3")
    );
    let key = DatabaseKey::generate();
    let encrypted = migrate_or_open(&path, &key).await.unwrap();
    assert_eq!(encrypted.get_note(note.id).await.unwrap().unwrap(), note);
    encrypted.close().await;
    assert!(
        !std::fs::read(&path)
            .unwrap()
            .starts_with(b"SQLite format 3")
    );
    let reopened = migrate_or_open(&path, &key).await.unwrap();
    assert_eq!(
        reopened.get_note(note.id).await.unwrap().unwrap().content,
        "migration secret"
    );
}
