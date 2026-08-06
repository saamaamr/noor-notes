use gtk::prelude::*;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn productivity_commands_are_discoverable_and_keyboard_focusable() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    for widget in [
        toolbar.find.upcast_ref::<gtk::Widget>(),
        toolbar.duplicate.upcast_ref(),
        toolbar.export_text.upcast_ref(),
        toolbar.export_markdown.upcast_ref(),
        toolbar.word_wrap.upcast_ref(),
        toolbar.zoom_in.upcast_ref(),
        toolbar.zoom_out.upcast_ref(),
        toolbar.zoom_reset.upcast_ref(),
        toolbar.go_to_line.upcast_ref(),
        toolbar.fullscreen.upcast_ref(),
    ] {
        assert!(widget.is_sensitive());
        assert!(widget.can_focus());
    }
    assert!(toolbar.find.tooltip_text().unwrap().contains("Ctrl+F"));
    assert!(
        toolbar
            .duplicate
            .tooltip_text()
            .unwrap()
            .contains("Duplicate")
    );
    assert!(
        toolbar
            .go_to_line
            .tooltip_text()
            .unwrap()
            .contains("Ctrl+G")
    );
}
