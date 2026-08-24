use chrono::Utc;
use noor_domain::{Alignment, ListKind, Note, RichBlock, RichDocument, RichSpan, TextMarks};
use noor_notes::export::{ExportDocument, ExportFormat, render_export};
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[test]
fn docx_export_is_real_ooxml_and_does_not_mutate_rich_unicode_note() {
    let mut note = Note::new(Utc::now());
    note.title = "নূর পরিকল্পনা".into();
    note.content = "গুরুত্বপূর্ণ\nদ্বিতীয়".into();
    note.rich_content = Some(RichDocument {
        version: 1,
        blocks: vec![
            RichBlock {
                alignment: Alignment::Center,
                spans: vec![RichSpan {
                    text: "গুরুত্বপূর্ণ".into(),
                    marks: TextMarks {
                        bold: true,
                        italic: true,
                        underline: true,
                        strikethrough: true,
                        font_size: Some(24),
                        foreground: Some("blue".into()),
                        highlight: Some("yellow".into()),
                    },
                }],
                ..RichBlock::default()
            },
            RichBlock {
                list: Some(ListKind::Numbered),
                spans: vec![RichSpan {
                    text: "দ্বিতীয়".into(),
                    marks: TextMarks::default(),
                }],
                ..RichBlock::default()
            },
        ],
    });
    let original = note.clone();

    let bytes = render_export(&ExportDocument::from_note(&note), ExportFormat::Docx).unwrap();

    assert!(
        bytes.starts_with(b"PK\x03\x04"),
        "DOCX must be an OOXML ZIP"
    );
    assert!(
        bytes.windows(4).any(|window| window == b"PK\x05\x06"),
        "DOCX must contain a ZIP end-of-central-directory record"
    );
    assert!(bytes.len() > 1_000, "DOCX package is unexpectedly small");

    let mut archive = ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let document_xml = archive_text(&mut archive, "word/document.xml");
    let numbering_xml = archive_text(&mut archive, "word/numbering.xml");
    assert!(document_xml.contains("নূর পরিকল্পনা"));
    assert!(document_xml.contains("গুরুত্বপূর্ণ"));
    for formatting_tag in ["<w:b", "<w:i", "<w:u", "<w:strike", "<w:color", "<w:shd"] {
        assert!(
            document_xml.contains(formatting_tag),
            "missing DOCX formatting tag {formatting_tag}"
        );
    }
    assert!(document_xml.contains("w:val=\"center\""));
    assert!(document_xml.contains("<w:numPr>"));
    assert!(numbering_xml.contains("w:val=\"decimal\""));
    assert!(numbering_xml.contains("w:val=\"bullet\""));
    assert_eq!(note, original);
}

fn archive_text(archive: &mut ZipArchive<Cursor<&Vec<u8>>>, path: &str) -> String {
    let mut contents = String::new();
    archive
        .by_name(path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    contents
}
