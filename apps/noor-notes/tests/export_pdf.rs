use chrono::Utc;
use noor_domain::{Alignment, Note, RichBlock, RichDocument, RichSpan, TextMarks};
use noor_notes::export::{ExportDocument, ExportFormat, render_export};

#[test]
fn pdf_export_is_a_real_paginated_unicode_document_without_note_mutation() {
    let mut note = Note::new(Utc::now());
    note.title = "নূর PDF পরিকল্পনা".into();
    note.content = "গুরুত্বপূর্ণ লেখা".into();
    note.rich_content = Some(RichDocument {
        version: 1,
        blocks: (0..120)
            .map(|index| RichBlock {
                alignment: if index == 0 {
                    Alignment::Center
                } else {
                    Alignment::Start
                },
                spans: vec![RichSpan {
                    text: format!("গুরুত্বপূর্ণ লেখা {index} — Noor Notes Unicode export"),
                    marks: TextMarks {
                        bold: index == 0,
                        italic: index == 1,
                        underline: index == 2,
                        strikethrough: index == 3,
                        font_size: (index == 4).then_some(24),
                        foreground: (index == 5).then(|| "blue".into()),
                        highlight: (index == 6).then(|| "yellow".into()),
                    },
                }],
                ..RichBlock::default()
            })
            .collect(),
    });
    let original = note.clone();

    let bytes = render_export(&ExportDocument::from_note(&note), ExportFormat::Pdf).unwrap();

    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.windows(5).any(|window| window == b"%%EOF"));
    assert!(bytes.len() > 4_000, "PDF package is unexpectedly small");
    assert!(
        bytes
            .windows(b"/Type /Page".len())
            .filter(|window| *window == b"/Type /Page")
            .count()
            >= 2,
        "long notes must paginate"
    );
    assert_eq!(note, original);
}
