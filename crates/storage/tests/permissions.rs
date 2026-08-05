#[cfg(unix)]
#[tokio::test]
async fn database_directory_and_files_are_owner_only() {
    use noor_storage::{DatabaseKey, SqliteNoteRepository};
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("private");
    let path = data.join("notes.db");
    let repo = SqliteNoteRepository::open_encrypted(&path, &DatabaseKey::generate())
        .await
        .unwrap();
    assert_eq!(
        std::fs::metadata(&data).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    repo.close().await;
}
