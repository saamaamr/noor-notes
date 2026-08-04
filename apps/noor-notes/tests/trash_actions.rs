const MAIN_WINDOW: &str = include_str!("../src/main_window.rs");
const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const TOOLBAR: &str = include_str!("../src/modern_toolbar.rs");

#[test]
fn trash_rows_expose_restore_delete_and_context_actions() {
    assert!(MAIN_WINDOW.contains("Restore"));
    assert!(MAIN_WINDOW.contains("Permanently Delete"));
    assert!(MAIN_WINDOW.contains("GestureClick"));
    assert!(MAIN_WINDOW.contains("delete_permanently"));
    assert!(MAIN_WINDOW.contains("repository.restore"));
}

#[test]
fn trashed_note_window_replaces_archive_and_trash_controls() {
    assert!(TOOLBAR.contains("pub restore"));
    assert!(TOOLBAR.contains("pub permanent_delete"));
    assert!(NOTE_WINDOW.contains("toolbar.restore.set_visible(is_trashed)"));
    assert!(NOTE_WINDOW.contains("toolbar.permanent_delete.set_visible(is_trashed)"));
    assert!(NOTE_WINDOW.contains("repository.delete_permanently"));
}
