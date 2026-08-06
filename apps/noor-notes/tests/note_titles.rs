const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const NOTE_CARD: &str = include_str!("../src/ui/note_card.rs");

#[test]
fn titles_are_editable_renameable_and_used_in_library() {
    assert!(NOTE_WINDOW.contains("nn-editor-title"));
    assert!(NOTE_WINDOW.contains("toolbar.rename.connect_clicked"));
    assert!(NOTE_WINDOW.contains("Rename note"));
    assert!(NOTE_CARD.contains("note.display_title()"));
}
