const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const MAIN_WINDOW: &str = include_str!("../src/main_window.rs");

#[test]
fn titles_are_editable_renameable_and_used_in_library() {
    assert!(NOTE_WINDOW.contains("note-title-entry"));
    assert!(NOTE_WINDOW.contains("toolbar.rename.connect_clicked"));
    assert!(NOTE_WINDOW.contains("Rename note"));
    assert!(MAIN_WINDOW.contains("note.display_title()"));
}
