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
    buffer.place_cursor(&buffer.end_iter());
    RichBuffer::insert_emoji(&buffer, "✨");

    let (plain, document) = RichBuffer::snapshot(&buffer);

    assert_eq!(plain, "Hello world✨");
    assert_eq!(document.blocks[0].spans[0].text, "Hello");
    assert_eq!(
        document.blocks[0].spans[0].marks,
        TextMarks {
            bold: true,
            ..TextMarks::default()
        }
    );
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
