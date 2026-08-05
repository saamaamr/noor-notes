use gtk::prelude::*;
use noor_notes::modern_toolbar::ModernToolbar;

#[test]
fn productivity_commands_are_discoverable_and_keyboard_focusable() {
    gtk::init().unwrap();
    let toolbar = ModernToolbar::new();
    for widget in [
        toolbar.find.upcast_ref::<gtk::Widget>(),
        toolbar.duplicate.upcast_ref(),
        toolbar.export_text.upcast_ref(),
        toolbar.export_markdown.upcast_ref(),
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
}
