use gtk::prelude::*;
use noor_notes::ui::note_preview::NotePreview;

#[test]
fn integrated_editor_adapts_document_and_controls_to_available_width() {
    gtk::init().unwrap();
    let preview = NotePreview::new();
    preview.toolbar().widget.set_visible(true);

    preview.set_available_width(900);
    assert_eq!(preview.content_maximum_width(), 828);
    assert!(!preview.is_compact());
    assert!(!preview.is_narrow());
    assert!(preview.toolbar().group_visible(1));
    assert!(preview.toolbar().group_visible(3));

    preview.set_available_width(620);
    assert_eq!(preview.content_maximum_width(), 620);
    assert!(preview.is_compact());
    assert!(!preview.is_narrow());
    assert!(!preview.toolbar().group_visible(1));
    assert!(!preview.toolbar().group_visible(3));

    preview.set_available_width(430);
    assert_eq!(preview.content_maximum_width(), 430);
    assert!(preview.is_compact());
    assert!(preview.is_narrow());
}
