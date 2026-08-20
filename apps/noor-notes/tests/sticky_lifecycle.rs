use chrono::{Duration, Utc};
use noor_domain::{Note, NoteState};
use noor_notes::ui::library_window::{apply_saved_card_action, persist_sticky_preferences};
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
        now + Duration::seconds(1),
    )
    .await
    .unwrap();
    let archived = persist_sticky_preferences(
        &repository,
        note.id,
        Some(false),
        None,
        now + Duration::seconds(2),
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
        now + Duration::seconds(3),
    )
    .await
    .unwrap();
    let trashed = persist_sticky_preferences(
        &repository,
        note.id,
        Some(false),
        None,
        now + Duration::seconds(4),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(matches!(trashed.state, NoteState::Trashed { .. }));

    apply_saved_card_action(
        &repository,
        note.id,
        CardAction::DeletePermanently,
        now + Duration::seconds(5),
    )
    .await
    .unwrap();
    assert!(
        persist_sticky_preferences(
            &repository,
            note.id,
            Some(false),
            None,
            now + Duration::seconds(6),
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
        now + Duration::seconds(1),
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
