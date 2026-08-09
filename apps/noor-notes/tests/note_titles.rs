use chrono::Utc;
use gtk::prelude::*;
use noor_domain::Note;
use noor_notes::ui::editor_header::EditorHeader;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn title_editor_remains_editable_after_header_extraction() {
    gtk::init().unwrap();
    let mut note = Note::new(Utc::now());
    note.title = "Design system".into();
    let toolbar = EditorToolbar::new();
    let appearance = gtk::Button::new();
    let header = EditorHeader::new(&note, &toolbar, &appearance, false);

    assert!(header.title_entry.has_css_class("nn-editor-title"));
    assert!(header.title_entry.is_editable());
    assert_eq!(header.title_entry.text(), "Design system");
    header.title_entry.set_text("Renamed note");
    assert_eq!(header.title_entry.text(), "Renamed note");
}
