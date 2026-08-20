use chrono::Utc;
use gtk::prelude::*;
use noor_domain::{Alignment, Note, TextMarks};
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
    RichBuffer::strikethrough(&buffer);
    RichBuffer::font_size(&buffer, 24);
    RichBuffer::foreground(&buffer, "#1A2B3C");
    RichBuffer::highlight(&buffer, "#F1E2D3");
    RichBuffer::align(&buffer, Alignment::Center);
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
    assert_eq!(restored.rich_content.as_ref(), Some(&document));
    let restored_document = restored.rich_content.as_ref().unwrap();
    let first = &restored_document.blocks[0].spans[0].marks;
    assert_eq!(
        first,
        &TextMarks {
            bold: true,
            italic: true,
            underline: true,
            strikethrough: true,
            font_size: Some(24),
            foreground: Some("#1A2B3C".into()),
            highlight: Some("#F1E2D3".into()),
        }
    );
    assert_eq!(restored_document.blocks[0].alignment, Alignment::Center);
    assert!(restored.content.contains("• First item"));

    let round_trip = gtk::TextBuffer::new(None);
    RichBuffer::load(
        &round_trip,
        &restored.content,
        restored.rich_content.as_ref(),
    );
    assert_eq!(RichBuffer::snapshot(&round_trip).1, document);

    let clear_buffer = gtk::TextBuffer::new(None);
    RichBuffer::load(&clear_buffer, "Keep Clear", None);
    clear_buffer.select_range(&clear_buffer.start_iter(), &clear_buffer.end_iter());
    RichBuffer::bold(&clear_buffer);
    RichBuffer::font_size(&clear_buffer, 18);
    RichBuffer::foreground(&clear_buffer, "#1A2B3C");
    RichBuffer::highlight(&clear_buffer, "#F1E2D3");
    clear_buffer.select_range(&clear_buffer.iter_at_offset(5), &clear_buffer.end_iter());
    RichBuffer::clear_formatting(&clear_buffer);

    let (clear_content, clear_document) = RichBuffer::snapshot(&clear_buffer);
    assert_eq!(clear_document.blocks[0].spans.len(), 2);
    assert!(clear_document.blocks[0].spans[0].marks.bold);
    assert_eq!(clear_document.blocks[0].spans[0].marks.font_size, Some(18));
    assert_eq!(
        clear_document.blocks[0].spans[1].marks,
        TextMarks::default()
    );

    let clear_round_trip = gtk::TextBuffer::new(None);
    RichBuffer::load(&clear_round_trip, &clear_content, Some(&clear_document));
    assert_eq!(
        RichBuffer::snapshot(&clear_round_trip),
        (clear_content, clear_document)
    );
}
