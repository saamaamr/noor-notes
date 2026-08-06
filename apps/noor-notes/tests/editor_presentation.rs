use adw::prelude::*;
use noor_notes::ui::editor_presentation::EditorPresentation;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn view_only_hides_editor_chrome_and_preserves_reading_access() {
    gtk::init().unwrap();
    let editor = gtk::TextView::new();
    let title = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let metadata = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let find = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let status = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    find.set_visible(false);
    let presentation = EditorPresentation::new(
        &editor,
        false,
        vec![
            title.clone().upcast(),
            toolbar.clone().upcast(),
            metadata.clone().upcast(),
            find.clone().upcast(),
            status.clone().upcast(),
        ],
    );

    presentation.set_view_only(true);
    assert!(presentation.is_view_only());
    assert!(!editor.is_editable());
    assert!(editor.is_cursor_visible());
    assert!(!title.is_visible());
    assert!(!toolbar.is_visible());
    assert!(!metadata.is_visible());
    assert!(!find.is_visible());
    assert!(!status.is_visible());

    presentation.set_view_only(false);
    assert!(!presentation.is_view_only());
    assert!(editor.is_editable());
    assert!(title.is_visible());
    assert!(toolbar.is_visible());
    assert!(metadata.is_visible());
    assert!(!find.is_visible());
    assert!(status.is_visible());

    let editor = gtk::TextView::new();
    let presentation = EditorPresentation::new(&editor, true, Vec::new());
    presentation.set_view_only(false);
    assert!(!editor.is_editable());

    let toolbar = EditorToolbar::new();
    assert_eq!(toolbar.view_only.label().as_deref(), Some("View Only"));
    assert_eq!(
        toolbar.view_only.tooltip_text().as_deref(),
        Some("Read this note without editing controls")
    );
}
