const MAIN_WINDOW: &str = include_str!("../src/ui/library_window.rs");
const NOTE_CARD: &str = include_str!("../src/ui/note_card.rs");
const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const TOOLBAR: &str = include_str!("../src/ui/editor_toolbar.rs");

#[test]
fn trash_rows_expose_restore_delete_and_context_actions() {
    assert!(NOTE_CARD.contains("Restore"));
    assert!(NOTE_CARD.contains("Delete permanently"));
    assert!(NOTE_CARD.contains("Popover"));
    assert!(NOTE_CARD.contains("CardAction::Trash"));
    assert!(NOTE_CARD.contains("gesture.set_button(3)"));
    assert!(NOTE_CARD.contains("popover.popup()"));
    assert!(MAIN_WINDOW.contains("CardAction::Trash"));
    assert!(MAIN_WINDOW.contains("delete_permanently"));
    assert!(MAIN_WINDOW.contains("repository.restore"));
}

#[test]
fn trashed_note_window_replaces_archive_and_trash_controls() {
    assert!(TOOLBAR.contains("pub restore"));
    assert!(TOOLBAR.contains("pub permanent_delete"));
    assert!(TOOLBAR.contains("pub header_trash"));
    assert!(NOTE_WINDOW.contains("toolbar.header_trash"));
    assert!(NOTE_WINDOW.contains("toolbar.restore.set_visible(is_trashed)"));
    assert!(NOTE_WINDOW.contains("toolbar.permanent_delete.set_visible(is_trashed)"));
    assert!(NOTE_WINDOW.contains("repository.delete_permanently"));
}
