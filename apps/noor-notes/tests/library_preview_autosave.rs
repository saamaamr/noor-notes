use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use chrono::Utc;
use gtk::prelude::*;
use noor_domain::Note;
use noor_notes::autosave::AutosaveQueue;
use noor_notes::rich_buffer::RichBuffer;
use noor_notes::ui::library_window::preview_edit_handler;
use noor_notes::ui::note_preview::NotePreview;
use noor_storage::SqliteNoteRepository;

#[tokio::test(flavor = "current_thread")]
async fn preview_body_edit_updates_library_cache_and_existing_autosave_pipeline() {
    gtk::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let original = Note::new(Utc::now());
    let untouched = Note::new(Utc::now());
    repository.save_note(&original).await.unwrap();
    repository.save_note(&untouched).await.unwrap();
    let notes = Rc::new(RefCell::new(vec![original.clone(), untouched.clone()]));
    let collection_cache = Rc::new(RefCell::new(None::<Note>));
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_secs(60));
    let handle_edit = preview_edit_handler(notes.clone(), autosave.clone(), {
        let collection_cache = collection_cache.clone();
        Rc::new(move |note| {
            collection_cache.replace(Some(note.clone()));
        })
    });

    let mut edited = original.clone();
    edited.content = "Saved from the preview body".into();
    handle_edit(edited.clone());

    assert_eq!(notes.borrow()[0].content, "Saved from the preview body");
    assert_eq!(notes.borrow()[1], untouched);
    assert_eq!(
        collection_cache
            .borrow()
            .as_ref()
            .map(|note| note.content.as_str()),
        Some("Saved from the preview body")
    );
    assert!(autosave.has_pending(original.id));
    autosave.flush(original.id).await.unwrap();
    let persisted = repository.get_note(original.id).await.unwrap().unwrap();
    assert_eq!(persisted.content, "Saved from the preview body");

    let preview = NotePreview::new_with_handler(handle_edit);
    preview.show_note(&persisted);
    preview.begin_editing();
    let buffer = preview.editor().buffer();
    buffer.select_range(&buffer.start_iter(), &buffer.end_iter());
    RichBuffer::font_size(&buffer, 24);
    RichBuffer::foreground(&buffer, "#1A2B3C");
    RichBuffer::highlight(&buffer, "#F1E2D3");
    assert!(autosave.has_pending(original.id));
    autosave.flush(original.id).await.unwrap();

    let reopened = repository.get_note(original.id).await.unwrap().unwrap();
    let marks = &reopened.rich_content.unwrap().blocks[0].spans[0].marks;
    assert_eq!(marks.font_size, Some(24));
    assert_eq!(marks.foreground.as_deref(), Some("#1A2B3C"));
    assert_eq!(marks.highlight.as_deref(), Some("#F1E2D3"));
}
