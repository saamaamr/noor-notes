const PREVIEW: &str = include_str!("../src/ui/note_preview.rs");
const LIBRARY: &str = include_str!("../src/ui/library_window.rs");
const LIB: &str = include_str!("../src/lib.rs");

#[test]
fn preview_owns_shared_editor_surface_and_read_only_sticky_flow() {
    assert!(PREVIEW.contains("NoteEditorSurface"));
    assert!(PREVIEW.contains("on_read_only_changed"));
    assert!(PREVIEW.contains("title_entry"));
    assert!(PREVIEW.contains("EditorToolbar"));
    assert!(PREVIEW.contains("toolbar.format.set_tooltip_text(Some(\"Formatting\"))"));
    assert!(PREVIEW.contains("note.editor_mode = EditorMode::Rich"));
    assert!(PREVIEW.contains("set_sticky_read_only"));
    assert!(LIB.contains("sticky_note_window"));
    assert!(LIBRARY.contains("StickyNoteWindow"));
}

#[test]
fn library_actions_never_close_main_window() {
    let action_handler = LIBRARY
        .split("fn handle_card_action")
        .nth(1)
        .expect("MainWindow action handler");
    assert!(!action_handler.contains("window.close()"));
    assert!(!action_handler.contains("app.quit"));
}
