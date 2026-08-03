use gtk::prelude::*;
use noor_domain::{RichDocument, TextMarks};
use noor_notes::rich_buffer::RichBuffer;

#[test]
fn rich_buffer_round_trip_preserves_bold_selection_and_emoji() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    RichBuffer::load(
        &buffer,
        "Hello world",
        Some(&RichDocument::from_plain_text("Hello world")),
    );
    buffer.select_range(&buffer.iter_at_offset(0), &buffer.iter_at_offset(5));
    RichBuffer::bold(&buffer);
    RichBuffer::font_size(&buffer, 18);
    RichBuffer::foreground(&buffer, "blue");
    RichBuffer::highlight(&buffer, "green");
    buffer.place_cursor(&buffer.iter_at_offset(2));
    RichBuffer::align(&buffer, noor_domain::Alignment::Center);
    buffer.place_cursor(&buffer.end_iter());
    RichBuffer::insert_emoji(&buffer, "✨");

    let (plain, document) = RichBuffer::snapshot(&buffer);

    assert_eq!(plain, "Hello world✨");
    assert_eq!(document.blocks[0].alignment, noor_domain::Alignment::Center);
    assert_eq!(document.blocks[0].spans[0].text, "Hello");
    assert_eq!(
        document.blocks[0].spans[0].marks,
        TextMarks {
            bold: true,
            font_size: Some(18),
            foreground: Some("blue".to_string()),
            highlight: Some("green".to_string()),
            ..TextMarks::default()
        }
    );
    let reloaded = gtk::TextBuffer::new(None);
    RichBuffer::load(&reloaded, &plain, Some(&document));
    let (_, reloaded_document) = RichBuffer::snapshot(&reloaded);
    assert_eq!(reloaded_document, document);

    let fallback = gtk::TextBuffer::new(None);
    let mut unsupported = RichDocument::from_plain_text("stale");
    unsupported.version = 99;
    RichBuffer::load(&fallback, "safe plain text", Some(&unsupported));
    assert_eq!(
        fallback
            .text(&fallback.start_iter(), &fallback.end_iter(), true)
            .as_str(),
        "safe plain text"
    );
}
