use noor_notes::export::ExportFormat;
use noor_notes::safe_export::{ExportExtension, ensure_export_extension, sanitize_export_name};
use noor_notes::save_as::{SaveAsError, validate_export_path};
use std::path::Path;

#[test]
fn export_names_remove_controls_and_path_separators() {
    let name = sanitize_export_name("../secret\nname\\bad", ExportExtension::Markdown);
    assert_eq!(name, "secret name bad.md");
    assert!(name.chars().count() <= 123);
}

#[test]
fn every_supported_export_format_has_one_safe_file_contract() {
    let cases = [
        (
            ExportFormat::Docx,
            ExportExtension::Docx,
            "docx",
            "Word Document",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        (
            ExportFormat::Pdf,
            ExportExtension::Pdf,
            "pdf",
            "PDF Document",
            "application/pdf",
        ),
        (
            ExportFormat::Html,
            ExportExtension::Html,
            "html",
            "HTML Document",
            "text/html",
        ),
        (
            ExportFormat::PlainText,
            ExportExtension::PlainText,
            "txt",
            "Plain Text",
            "text/plain",
        ),
        (
            ExportFormat::Markdown,
            ExportExtension::Markdown,
            "md",
            "Markdown",
            "text/markdown",
        ),
    ];

    for (format, extension, suffix, label, mime_type) in cases {
        assert_eq!(format.extension(), extension);
        assert_eq!(format.extension().as_str(), suffix);
        assert_eq!(format.label(), label);
        assert_eq!(format.mime_type(), mime_type);
        assert_eq!(
            sanitize_export_name("নূর নোট", extension),
            format!("নূর নোট.{suffix}")
        );
    }
}

#[test]
fn empty_and_very_long_export_titles_remain_bounded() {
    assert_eq!(
        sanitize_export_name("... / \\ \n", ExportExtension::Pdf),
        "Untitled.pdf"
    );

    let long_title = "ন".repeat(300);
    let name = sanitize_export_name(&long_title, ExportExtension::Docx);
    assert_eq!(name.chars().count(), 125);
    assert!(name.ends_with(".docx"));
}

#[test]
fn selected_export_path_always_uses_the_chosen_format() {
    assert_eq!(
        ensure_export_extension(Path::new("/tmp/Noor Plan"), ExportExtension::Docx),
        Path::new("/tmp/Noor Plan.docx")
    );
    assert_eq!(
        ensure_export_extension(Path::new("/tmp/Noor Plan.PDF"), ExportExtension::Pdf),
        Path::new("/tmp/Noor Plan.PDF")
    );
    assert_eq!(
        ensure_export_extension(Path::new("/tmp/Noor Plan.txt"), ExportExtension::Markdown),
        Path::new("/tmp/Noor Plan.md")
    );
}

#[test]
fn save_as_rejects_a_path_that_does_not_match_the_selected_format() {
    let accepted = validate_export_path(Path::new("/tmp/Noor Plan.PDF"), ExportFormat::Pdf)
        .expect("matching extension should remain accepted");
    assert_eq!(accepted, Path::new("/tmp/Noor Plan.PDF"));

    let error = validate_export_path(Path::new("/tmp/Noor Plan.txt"), ExportFormat::Markdown)
        .expect_err("a mismatched extension must not silently target another file");
    assert!(matches!(error, SaveAsError::WrongExtension("md")));
}
