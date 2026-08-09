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
    assert_eq!(
        toolbar.header_archive.tooltip_text().as_deref(),
        Some("Archive note")
    );
    assert!(toolbar.header_archive.can_focus());
    assert_eq!(toolbar.note_color_buttons.len(), 6);
    for button in &toolbar.note_color_buttons {
        assert!(button.tooltip_text().is_some());
    }
    toolbar.set_rich_formatting_enabled(false);
    assert!(!toolbar.format.is_sensitive());
    assert!(!toolbar.foreground_palette.widget.is_sensitive());
    assert!(!toolbar.highlight_palette.widget.is_sensitive());
    assert!(!toolbar.font_size.is_sensitive());
    assert!(!toolbar.custom_font_size.is_sensitive());
    assert!(!toolbar.clear_formatting.is_sensitive());

    for palette in [&toolbar.foreground_palette, &toolbar.highlight_palette] {
        assert!(palette.reset.can_focus());
        assert!(palette.reset.tooltip_text().is_some());
        assert!(palette.custom.can_focus());
        assert!(palette.custom.tooltip_text().is_some());
        for button in &palette.preset_buttons {
            assert!(button.can_focus());
            assert!(button.tooltip_text().is_some());
        }
    }
}
