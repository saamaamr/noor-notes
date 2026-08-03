use chrono::Utc;
use noor_domain::{Alignment, ListKind, Note, RichBlock, RichDocument, RichSpan, TextMarks};

#[test]
fn plain_text_becomes_one_block_per_line_and_round_trips() {
    let document = RichDocument::from_plain_text("Heading\nSecond line");
    assert_eq!(document.version, 1);
    assert_eq!(document.blocks.len(), 2);
    assert_eq!(document.plain_text(), "Heading\nSecond line");
}

#[test]
fn styled_document_json_preserves_blocks_and_marks() {
    let document = RichDocument {
        version: 1,
        blocks: vec![RichBlock {
            alignment: Alignment::Center,
            list: Some(ListKind::Bullet),
            spans: vec![RichSpan {
                text: "Important".into(),
                marks: TextMarks {
                    bold: true,
                    italic: true,
                    font_size: Some(18),
                    foreground: Some("#174a7e".into()),
                    highlight: Some("#fff0a6".into()),
                    ..TextMarks::default()
                },
            }],
        }],
    };
    let json = serde_json::to_string(&document).unwrap();
    let restored: RichDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, document);
    assert_eq!(restored.plain_text(), "Important");
}

#[test]
fn old_note_json_without_rich_content_remains_readable() {
    let note = Note::new(Utc::now());
    let mut value = serde_json::to_value(note).unwrap();
    value.as_object_mut().unwrap().remove("rich_content");
    let restored: Note = serde_json::from_value(value).unwrap();
    assert!(restored.rich_content.is_none());
}
