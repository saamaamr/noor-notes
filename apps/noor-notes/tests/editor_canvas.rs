use adw::prelude::*;
use noor_notes::ui::editor_canvas::{build_editor_canvas, configure_editor_canvas};

#[test]
fn rich_writing_is_clamped_while_source_modes_keep_full_editor_width() {
    gtk::init().unwrap();
    let rich_editor = gtk::TextView::new();
    configure_editor_canvas(&rich_editor, true);
    assert_eq!(rich_editor.left_margin(), 8);
    assert_eq!(rich_editor.right_margin(), 8);
    assert_eq!(rich_editor.top_margin(), 5);
    assert_eq!(rich_editor.bottom_margin(), 5);
    assert!(rich_editor.has_css_class("nn-rich-writing-canvas"));
    let rich_canvas = build_editor_canvas(&rich_editor, true);
    let clamp = rich_canvas
        .downcast::<adw::Clamp>()
        .expect("rich writing canvas should use an Adwaita reading-width clamp");
    assert_eq!(clamp.maximum_size(), 860);

    let source_editor = gtk::TextView::new();
    configure_editor_canvas(&source_editor, false);
    assert_eq!(source_editor.left_margin(), 16);
    assert_eq!(source_editor.right_margin(), 16);
    assert_eq!(source_editor.top_margin(), 16);
    assert_eq!(source_editor.bottom_margin(), 24);
    assert!(source_editor.has_css_class("nn-source-canvas"));
    let source_canvas = build_editor_canvas(&source_editor, false);
    assert!(source_canvas.is::<gtk::TextView>());
}
