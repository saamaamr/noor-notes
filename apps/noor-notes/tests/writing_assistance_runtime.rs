use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use noor_domain::Note;
use noor_notes::key_store::InMemoryKeyStore;
use noor_notes::writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore};
use noor_storage::{DatabaseKey, SqliteNoteRepository};

#[tokio::test]
async fn rebuilds_are_shared_debounced_and_follow_note_lifecycle() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open_encrypted(
        &directory.path().join("notes.db"),
        &DatabaseKey::generate(),
    )
    .await
    .unwrap();
    let mut note = Note::new(Utc::now());
    note.content = "private phrase continues. private phrase helps.".into();
    repository.save_note(&note).await.unwrap();
    let runtime = WritingAssistanceRuntime::new(
        repository.clone(),
        WritingAssistanceStore::at(directory.path().join("writing.json")),
        Arc::new(InMemoryKeyStore::default()),
    )
    .await;

    runtime.rebuild_if_stale().await.unwrap();
    assert!(
        runtime
            .suggest("private phrase", "", 3)
            .contains(&"continues".into())
    );
    let initial_count = runtime.rebuild_count();

    runtime.schedule_model_rebuild(Duration::from_millis(25));
    runtime.schedule_model_rebuild(Duration::from_millis(25));
    repository.trash(note.id, Utc::now()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(runtime.rebuild_count(), initial_count + 1);
    assert!(runtime.suggest("private phrase", "", 3).is_empty());
}

#[tokio::test]
async fn valid_cached_models_load_without_rebuilding() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open_encrypted(
        &directory.path().join("notes.db"),
        &DatabaseKey::generate(),
    )
    .await
    .unwrap();
    let mut note = Note::new(Utc::now());
    note.content = "clear support helps".into();
    repository.save_note(&note).await.unwrap();
    let store = WritingAssistanceStore::at(directory.path().join("writing.json"));
    let keys = Arc::new(InMemoryKeyStore::default());
    let first =
        WritingAssistanceRuntime::new(repository.clone(), store.clone(), keys.clone()).await;
    first.rebuild_if_stale().await.unwrap();

    let second = WritingAssistanceRuntime::new(repository, store, keys).await;
    second.rebuild_if_stale().await.unwrap();
    assert_eq!(second.rebuild_count(), 0);
    assert_eq!(second.suggest("clear support", "h", 3), vec!["helps"]);
    assert!(second.cloud_client().is_none());
}
