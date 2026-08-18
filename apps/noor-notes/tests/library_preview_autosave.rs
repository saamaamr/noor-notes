use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use chrono::Utc;
use noor_domain::Note;
use noor_notes::autosave::AutosaveQueue;
use noor_notes::ui::library_window::preview_edit_handler;
use noor_storage::SqliteNoteRepository;

#[tokio::test]
async fn preview_body_edit_updates_library_cache_and_existing_autosave_pipeline() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let original = Note::new(Utc::now());
    let untouched = Note::new(Utc::now());
    repository.save_note(&original).await.unwrap();
    repository.save_note(&untouched).await.unwrap();
    let notes = Rc::new(RefCell::new(vec![original.clone(), untouched.clone()]));
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_secs(60));
    let handle_edit = preview_edit_handler(notes.clone(), autosave.clone());

    let mut edited = original.clone();
    edited.content = "Saved from the preview body".into();
    handle_edit(edited.clone());

    assert_eq!(notes.borrow()[0].content, "Saved from the preview body");
    assert_eq!(notes.borrow()[1], untouched);
    assert!(autosave.has_pending(original.id));
    autosave.flush(original.id).await.unwrap();
    let persisted = repository.get_note(original.id).await.unwrap().unwrap();
    assert_eq!(persisted.content, "Saved from the preview body");
}
