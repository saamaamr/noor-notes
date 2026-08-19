const TOOLBAR: &str = include_str!("../src/ui/editor_toolbar.rs");
const WINDOW: &str = include_str!("../src/sticky_note_window.rs");
const WINDOWING: &str = include_str!("../../../crates/windowing/src/controller.rs");

#[test]
fn sticky_window_has_explicit_always_on_top_and_read_only_controls() {
    assert!(TOOLBAR.contains("Always on Top"));
    assert!(TOOLBAR.contains("Read-only"));
    assert!(WINDOW.contains("GnomeWindowController::window_title"));
    assert!(WINDOWING.contains("set_always_on_top") || WINDOWING.contains("always_on_top"));
}
