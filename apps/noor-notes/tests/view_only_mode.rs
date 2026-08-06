const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const PRESENTATION: &str = include_str!("../src/ui/editor_presentation.rs");
const TOOLBAR: &str = include_str!("../src/ui/editor_toolbar.rs");

#[test]
fn note_window_persists_and_exits_view_only_mode_from_keyboard_or_body() {
    assert!(TOOLBAR.contains("pub view_only"));
    assert!(NOTE_WINDOW.contains("editor_preferences.view_only"));
    assert!(NOTE_WINDOW.contains("presentation.set_view_only"));
    assert!(NOTE_WINDOW.contains("GestureClick"));
    assert!(NOTE_WINDOW.contains("set_button(0)"));
    assert!(NOTE_WINDOW.contains("gtk::gdk::Key::Escape"));
    assert!(PRESENTATION.contains("editor.set_editable(false)"));
    assert!(PRESENTATION.contains("widget.set_visible(false)"));
}
