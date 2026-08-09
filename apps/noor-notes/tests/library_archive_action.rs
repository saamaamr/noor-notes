use chrono::{Duration, TimeZone, Utc};
use noor_domain::{Note, NoteState};
use noor_notes::ui::library_window::apply_saved_card_action;
use noor_notes::ui::note_card::CardAction;
use noor_storage::SqliteNoteRepository;

#[tokio::test]
async fn archive_card_action_persists_only_the_selected_note_as_archived() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let created = Utc.with_ymd_and_hms(2026, 8, 9, 10, 0, 0).unwrap();
    let archived_at = created + Duration::minutes(3);
    let selected = Note::new(created);
    let untouched = Note::new(created);
    repository.save_note(&selected).await.unwrap();
    repository.save_note(&untouched).await.unwrap();

    apply_saved_card_action(&repository, selected.id, CardAction::Archive, archived_at)
        .await
        .unwrap();

    let selected = repository.get_note(selected.id).await.unwrap().unwrap();
    let untouched = repository.get_note(untouched.id).await.unwrap().unwrap();
    assert_eq!(selected.state, NoteState::Archived);
    assert_eq!(selected.updated_at, archived_at);
    assert_eq!(selected.revision.value(), 1);
    assert_eq!(untouched.state, NoteState::Active);
    assert_eq!(untouched.revision.value(), 0);
}
