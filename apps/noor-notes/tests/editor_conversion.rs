use chrono::Utc;
use noor_domain::{EditorMode, Note, RichDocument, TextMarks};
use noor_notes::editor::{apply_conversion, preview_conversion};

#[test]
fn rich_conversion_warns_before_loss_and_preserves_original_until_applied() {
    let mut note = Note::new(Utc::now());
    note.content = "Important".into();
    let mut document = RichDocument::from_plain_text("Important");
    document.blocks[0].spans[0].marks = TextMarks {
        bold: true,
        foreground: Some("blue".into()),
        highlight: Some("green".into()),
        font_size: Some(24),
        ..TextMarks::default()
    };
    note.rich_content = Some(document);
    let original = note.clone();

    let preview = preview_conversion(&note, EditorMode::Markdown);
    assert!(preview.converted_content.contains("**Important**"));
    assert!(
        preview
            .warnings
            .iter()
            .any(|warning| warning.contains("colour"))
    );
    assert_eq!(note, original);

    apply_conversion(&mut note, preview);
    assert_eq!(note.editor_mode, EditorMode::Markdown);
    assert!(note.rich_content.is_none());
    assert!(note.content.contains("**Important**"));
}

#[test]
fn plain_to_rich_is_lossless_and_builds_native_document() {
    let mut note = Note::new(Utc::now());
    note.editor_mode = EditorMode::PlainText;
    note.content = "বাংলা\nplain".into();
    let preview = preview_conversion(&note, EditorMode::Rich);
    assert!(preview.warnings.is_empty());
    apply_conversion(&mut note, preview);
    assert_eq!(note.editor_mode, EditorMode::Rich);
    assert_eq!(note.rich_content.unwrap().plain_text(), "বাংলা\nplain");
}

#[test]
fn conversion_to_plain_text_does_not_invent_markup() {
    let mut note = Note::new(Utc::now());
    note.content = "<unsafe> & literal".into();
    let preview = preview_conversion(&note, EditorMode::PlainText);
    assert_eq!(preview.converted_content, "<unsafe> & literal");
    apply_conversion(&mut note, preview);
    assert_eq!(note.content, "<unsafe> & literal");
}
