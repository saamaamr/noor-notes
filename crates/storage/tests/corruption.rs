use std::fs;

use noor_storage::SqliteNoteRepository;

#[tokio::test]
async fn corrupt_database_is_preserved_and_backed_up_before_error_is_returned() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    let original = b"this is not a sqlite database";
    fs::write(&path, original).unwrap();

    assert!(SqliteNoteRepository::open(&path).await.is_err());

    assert_eq!(fs::read(&path).unwrap(), original);
    let backups: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|candidate| {
            candidate
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("notes.db.corrupt-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(&backups[0]).unwrap(), original);
}
