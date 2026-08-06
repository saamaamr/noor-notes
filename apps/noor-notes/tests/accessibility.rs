use gtk::prelude::*;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn primary_controls_have_accessible_descriptions_and_colour_choices() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    for button in [
        &toolbar.new_note,
        &toolbar.undo,
        &toolbar.redo,
        &toolbar.rename,
        &toolbar.duplicate,
        &toolbar.archive,
        &toolbar.trash,
    ] {
        assert!(button.tooltip_text().is_some());
        assert!(button.can_focus());
    }
    assert_eq!(toolbar.note_color_buttons.len(), 6);
    for button in &toolbar.note_color_buttons {
        assert!(button.tooltip_text().is_some());
    }
}
