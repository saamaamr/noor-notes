use chrono::Utc;
use gtk::prelude::*;
use noor_domain::{Note, TextMarks};
use noor_notes::rich_buffer::RichBuffer;
use noor_storage::SqliteNoteRepository;

#[tokio::test(flavor = "current_thread")]
async fn rich_formatting_persists_after_database_save_and_reopen() {
    gtk::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let mut note = Note::new(Utc::now());
    let buffer = gtk::TextBuffer::new(None);
    RichBuffer::load(&buffer, "Heading\nFirst item", None);
    buffer.select_range(&buffer.iter_at_offset(0), &buffer.iter_at_offset(7));
    RichBuffer::bold(&buffer);
    RichBuffer::italic(&buffer);
    RichBuffer::underline(&buffer);
    RichBuffer::foreground(&buffer, "#1A2B3C");
    RichBuffer::highlight(&buffer, "#F1E2D3");
    buffer.place_cursor(&buffer.iter_at_offset(8));
    RichBuffer::toggle_list(&buffer, noor_domain::ListKind::Bullet);
    let (content, document) = RichBuffer::snapshot(&buffer);
    note.content = content;
    note.rich_content = Some(document.clone());
    repository.save_note(&note).await.unwrap();
    drop(repository);

    let reopened = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let restored = reopened.get_note(note.id).await.unwrap().unwrap();
    assert_eq!(restored.rich_content, Some(document));
    let first = &restored.rich_content.unwrap().blocks[0].spans[0].marks;
    assert_eq!(
        first,
        &TextMarks {
            bold: true,
            italic: true,
            underline: true,
            foreground: Some("#1A2B3C".into()),
            highlight: Some("#F1E2D3".into()),
            ..TextMarks::default()
        }
    );
    assert!(restored.content.contains("• First item"));
}
