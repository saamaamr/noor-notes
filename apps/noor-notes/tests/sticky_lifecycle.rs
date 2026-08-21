use std::sync::Arc;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use noor_domain::{Note, NoteState};
use noor_notes::autosave::AutosaveQueue;
use noor_notes::ui::library_window::{
    apply_saved_card_action, persist_sticky_preferences, persist_sticky_preferences_serialized,
};
use noor_notes::ui::note_card::CardAction;
use noor_storage::SqliteNoteRepository;

#[tokio::test(flavor = "current_thread")]
async fn sticky_close_preserves_lifecycle_state_and_never_resurrects_deleted_note() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let mut note = Note::new(now);
    note.editor_preferences.view_only = true;
    repository.save_note(&note).await.unwrap();

    apply_saved_card_action(
        &repository,
        note.id,
        CardAction::Archive,
        now + ChronoDuration::seconds(1),
    )
    .await
    .unwrap();
    let archived = persist_sticky_preferences(
        &repository,
        note.id,
        Some(false),
        None,
        now + ChronoDuration::seconds(2),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(archived.state, NoteState::Archived);
    assert!(!archived.editor_preferences.view_only);

    apply_saved_card_action(
        &repository,
        note.id,
        CardAction::Trash,
        now + ChronoDuration::seconds(3),
    )
    .await
    .unwrap();
    let trashed = persist_sticky_preferences(
        &repository,
        note.id,
        Some(false),
        None,
        now + ChronoDuration::seconds(4),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(matches!(trashed.state, NoteState::Trashed { .. }));

    apply_saved_card_action(
        &repository,
        note.id,
        CardAction::DeletePermanently,
        now + ChronoDuration::seconds(5),
    )
    .await
    .unwrap();
    assert!(
        persist_sticky_preferences(
            &repository,
            note.id,
            Some(false),
            None,
            now + ChronoDuration::seconds(6),
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(repository.get_note(note.id).await.unwrap().is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn sticky_always_on_top_updates_only_the_current_authoritative_note() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let mut note = Note::new(now);
    note.title = "Authoritative title".into();
    note.editor_preferences.view_only = true;
    repository.save_note(&note).await.unwrap();

    let saved = persist_sticky_preferences(
        &repository,
        note.id,
        None,
        Some(true),
        now + ChronoDuration::seconds(1),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(saved.always_on_top);
    assert!(saved.editor_preferences.view_only);
    assert_eq!(saved.title, "Authoritative title");

    let reopened = repository.get_note(note.id).await.unwrap().unwrap();
    assert_eq!(reopened, saved);
}

#[tokio::test(flavor = "current_thread")]
async fn rapid_sticky_preferences_persist_latest_intent() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc::now();
    let note = Note::new(now);
    repository.save_note(&note).await.unwrap();
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_secs(30));
    let lane = Arc::new(tokio::sync::Mutex::new(()));
    let gate = lane.clone().lock_owned().await;

    let first = tokio::spawn(persist_sticky_preferences_serialized(
        repository.clone(),
        autosave.clone(),
        lane.clone(),
        note.id,
        None,
        Some(true),
        now + ChronoDuration::seconds(1),
    ));
    tokio::task::yield_now().await;
    let latest = tokio::spawn(persist_sticky_preferences_serialized(
        repository.clone(),
        autosave,
        lane,
        note.id,
        None,
        Some(false),
        now + ChronoDuration::seconds(2),
    ));
    tokio::task::yield_now().await;
    drop(gate);

    first.await.unwrap().unwrap();
    latest.await.unwrap().unwrap();
    assert!(
        !repository
            .get_note(note.id)
            .await
            .unwrap()
            .unwrap()
            .always_on_top
    );
}
