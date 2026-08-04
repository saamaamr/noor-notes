const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const MANAGED_APP: &str = include_str!("../src/managed_app.rs");

#[test]
fn every_primary_toolbar_button_is_wired() {
    assert!(
        NOTE_WINDOW.contains(".new_note")
            && NOTE_WINDOW.contains("app.activate_action(\"new-note\""),
        "New Note must activate app.new-note"
    );
    assert!(
        NOTE_WINDOW.contains("toolbar.archive.connect_clicked"),
        "Archive must persist an Archived transition"
    );
    assert!(
        NOTE_WINDOW.contains("toolbar.trash.connect_clicked"),
        "Delete must open its confirmation flow"
    );
    assert!(
        NOTE_WINDOW.contains("Move to Trash"),
        "Delete must require explicit confirmation"
    );
    assert!(
        NOTE_WINDOW.contains("autosave.flush"),
        "state-changing actions must save before closing"
    );
    assert!(
        NOTE_WINDOW.contains("refresh-notes") && MANAGED_APP.contains("refresh-notes"),
        "saved lifecycle changes must refresh the main note lists"
    );
}
