use chrono::Utc;
use noor_domain::{
    Alignment, EditorMode, ListKind, Note, RichBlock, RichDocument, RichSpan, TextMarks,
};
use noor_notes::export::{
    ExportBlockKind, ExportDocument, ExportFormat, export_markdown, export_plain, render_export,
};

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

#[test]
fn normalized_export_document_preserves_live_rich_structure() {
    let mut note = Note::new(Utc::now());
    note.title = "নূর & Notes".into();
    note.content = "গুরুত্বপূর্ণ\nদ্বিতীয়".into();
    note.rich_content = Some(RichDocument {
        version: 1,
        blocks: vec![
            RichBlock {
                alignment: Alignment::Center,
                list: Some(ListKind::Bullet),
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

    let document = ExportDocument::from_note(&note);
    assert_eq!(document.title, "নূর & Notes");
    assert_eq!(document.blocks.len(), 2);
    assert_eq!(
        document.blocks[0].kind,
        ExportBlockKind::ListItem {
            kind: ListKind::Bullet,
            ordinal: 1,
        }
    );
    assert_eq!(document.blocks[0].alignment, Alignment::Center);
    assert_eq!(document.blocks[0].runs[0].text, "গুরুত্বপূর্ণ");
    assert!(document.blocks[0].runs[0].marks.bold);
    assert_eq!(document.blocks[0].runs[0].marks.font_size, Some(24));
    assert_eq!(
        document.blocks[1].kind,
        ExportBlockKind::ListItem {
            kind: ListKind::Numbered,
            ordinal: 1,
        }
    );
}

#[test]
fn text_markdown_and_html_render_title_body_and_safe_formatting() {
    let mut note = Note::new(Utc::now());
    note.title = "<Plan & Review>".into();
    note.content = "one\ntwo".into();
    note.rich_content = Some(RichDocument {
        version: 1,
        blocks: vec![
            RichBlock {
                list: Some(ListKind::Bullet),
                spans: vec![RichSpan {
                    text: "one & <safe>".into(),
                    marks: TextMarks {
                        bold: true,
                        underline: true,
                        foreground: Some("blue".into()),
                        highlight: Some("yellow".into()),
                        ..TextMarks::default()
                    },
                }],
                ..RichBlock::default()
            },
            RichBlock {
                spans: vec![RichSpan {
                    text: "two".into(),
                    marks: TextMarks {
                        italic: true,
                        strikethrough: true,
                        ..TextMarks::default()
                    },
                }],
                ..RichBlock::default()
            },
        ],
    });
    let document = ExportDocument::from_note(&note);

    let text =
        String::from_utf8(render_export(&document, ExportFormat::PlainText).unwrap()).unwrap();
    assert_eq!(text, "<Plan & Review>\n\n• one & <safe>\ntwo\n");

    let markdown =
        String::from_utf8(render_export(&document, ExportFormat::Markdown).unwrap()).unwrap();
    assert!(markdown.starts_with("# <Plan & Review>\n\n"));
    assert!(markdown.contains("- **<u>one & <safe></u>**"));
    assert!(markdown.contains("*~~two~~*"));

    let html = String::from_utf8(render_export(&document, ExportFormat::Html).unwrap()).unwrap();
    assert!(html.contains("<title>&lt;Plan &amp; Review&gt;</title>"));
    assert!(html.contains("<h1>&lt;Plan &amp; Review&gt;</h1>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("one &amp; &lt;safe&gt;"));
    assert!(html.contains("<strong>"));
    assert!(html.contains("<u>"));
    assert!(html.contains("color: #1D4ED8"));
    assert!(html.contains("background-color: #FEF3C7"));
    assert!(!html.contains("one & <safe>"));
}

#[test]
fn source_modes_export_without_inventing_rich_document_state() {
    let mut markdown_note = Note::new(Utc::now());
    markdown_note.title = "Markdown".into();
    markdown_note.editor_mode = EditorMode::Markdown;
    markdown_note.content = "## Heading\n\n**bold**".into();
    let markdown = ExportDocument::from_note(&markdown_note);
    let markdown_bytes = render_export(&markdown, ExportFormat::Markdown).unwrap();
    let markdown_text = String::from_utf8(markdown_bytes).unwrap();
    assert!(markdown_text.contains("# Markdown\n\n## Heading\n\n**bold**"));

    let mut code_note = Note::new(Utc::now());
    code_note.title = "Rust".into();
    code_note.editor_mode = EditorMode::Code;
    code_note.content = "fn main() {\n    println!(\"নূর\");\n}".into();
    let code = ExportDocument::from_note(&code_note);
    assert_eq!(code.blocks[0].kind, ExportBlockKind::CodeBlock);
    let html = String::from_utf8(render_export(&code, ExportFormat::Html).unwrap()).unwrap();
    assert!(html.contains("<pre><code>fn main()"));
    assert!(html.contains("println!(&quot;নূর&quot;);"));
}
