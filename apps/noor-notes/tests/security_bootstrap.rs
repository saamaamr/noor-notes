use noor_notes::key_store::{InMemoryKeyStore, KeyStore, SecretKind};
use noor_notes::security_bootstrap::{BootstrapError, open_repository};
use noor_storage::{DatabaseKey, SqliteNoteRepository};
use std::sync::Arc;

#[tokio::test]
async fn first_run_creates_one_key_and_restart_reuses_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    let keys = Arc::new(InMemoryKeyStore::default());
    open_repository(&path, keys.clone())
        .await
        .unwrap()
        .close()
        .await;
    let first = keys
        .get(SecretKind::DatabaseKey, "local-default")
        .await
        .unwrap()
        .unwrap();
    open_repository(&path, keys.clone())
        .await
        .unwrap()
        .close()
        .await;
    let second = keys
        .get(SecretKind::DatabaseKey, "local-default")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.as_slice(), second.as_slice());
}

#[tokio::test]
async fn encrypted_database_without_key_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    SqliteNoteRepository::open_encrypted(&path, &DatabaseKey::generate())
        .await
        .unwrap()
        .close()
        .await;
    let error = open_repository(&path, Arc::new(InMemoryKeyStore::default()))
        .await
        .err()
        .unwrap();
    assert!(matches!(error, BootstrapError::DatabaseKeyMissing));
}
