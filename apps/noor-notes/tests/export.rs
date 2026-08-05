use chrono::Utc;
use noor_domain::{ListKind, Note, RichBlock, RichDocument, RichSpan, TextMarks};
use noor_notes::export::{export_markdown, export_plain};

#[test]
fn exports_plain_and_markdown_without_mutating_note() {
    let mut note = Note::new(Utc::now());
    note.title = "Plan".into();
    note.content = "one\ntwo".into();
    note.rich_content = Some(RichDocument {
        version: 1,
        blocks: vec![
            RichBlock {
                list: Some(ListKind::Bullet),
                spans: vec![RichSpan {
                    text: "one".into(),
                    marks: TextMarks {
                        bold: true,
                        ..Default::default()
                    },
                }],
                ..Default::default()
            },
            RichBlock {
                spans: vec![RichSpan {
                    text: "two".into(),
                    marks: TextMarks {
                        italic: true,
                        ..Default::default()
                    },
                }],
                ..Default::default()
            },
        ],
    });
    assert_eq!(export_plain(&note), "one\ntwo");
    assert_eq!(export_markdown(&note), "- **one**\n*two*\n");
    assert_eq!(note.content, "one\ntwo");
}
