use gtk::prelude::*;
use noor_domain::{RichDocument, TextMarks};
use noor_notes::rich_buffer::RichBuffer;
use noor_notes::{appearance::EffectiveTheme, editor::SourceEditorAdapter};
use sourceview5::prelude::*;

#[test]
fn rich_buffer_round_trip_preserves_bold_selection_and_emoji() {
    gtk::init().unwrap();
    let editor = SourceEditorAdapter::new_rich("Hello world", EffectiveTheme::Light);
    assert!(!editor.buffer().is_highlight_syntax());
    let buffer: gtk::TextBuffer = editor.buffer().clone().upcast();
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
    assert_eq!(RichBuffer::parse_font_size("37"), Some(37));
    assert_eq!(RichBuffer::parse_font_size("65536"), Some(65536));
    assert_eq!(RichBuffer::parse_font_size("0"), None);
    assert_eq!(RichBuffer::parse_font_size("-2"), None);
    assert_eq!(RichBuffer::parse_font_size("2.5"), None);
    assert_eq!(
        fallback
            .text(&fallback.start_iter(), &fallback.end_iter(), true)
            .as_str(),
        "safe plain text"
    );
    let clear_buffer = gtk::TextBuffer::new(None);
    RichBuffer::load(
        &clear_buffer,
        "Formatted",
        Some(&RichDocument::from_plain_text("Formatted")),
    );
    clear_buffer.select_range(&clear_buffer.start_iter(), &clear_buffer.end_iter());
    RichBuffer::bold(&clear_buffer);
    RichBuffer::foreground(&clear_buffer, "blue");
    RichBuffer::highlight(&clear_buffer, "green");
    RichBuffer::clear_formatting(&clear_buffer);
    let (_, cleared) = RichBuffer::snapshot(&clear_buffer);
    assert_eq!(cleared.blocks[0].spans[0].marks, TextMarks::default());
    let custom = gtk::TextBuffer::new(None);
    RichBuffer::load(
        &custom,
        "Custom",
        Some(&RichDocument::from_plain_text("Custom")),
    );
    custom.select_range(&custom.start_iter(), &custom.end_iter());
    RichBuffer::foreground(&custom, "#1a2b3c");
    RichBuffer::highlight(&custom, "#f1e2d3");
    let (custom_plain, custom_document) = RichBuffer::snapshot(&custom);
    assert_eq!(
        custom_document.blocks[0].spans[0]
            .marks
            .foreground
            .as_deref(),
        Some("#1A2B3C")
    );
    assert_eq!(
        custom_document.blocks[0].spans[0]
            .marks
            .highlight
            .as_deref(),
        Some("#F1E2D3")
    );

    let custom_reloaded = gtk::TextBuffer::new(None);
    RichBuffer::load(&custom_reloaded, &custom_plain, Some(&custom_document));
    let (_, custom_reloaded_document) = RichBuffer::snapshot(&custom_reloaded);
    assert_eq!(custom_reloaded_document, custom_document);
}
