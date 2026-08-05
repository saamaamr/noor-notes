use noor_notes::safe_export::{ExportExtension, sanitize_export_name};

#[test]
fn export_names_remove_controls_and_path_separators() {
    let name = sanitize_export_name("../secret\nname\\bad", ExportExtension::Markdown);
    assert_eq!(name, "secret name bad.md");
    assert!(name.chars().count() <= 123);
}
