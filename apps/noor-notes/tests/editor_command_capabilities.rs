use noor_domain::EditorMode;
use noor_notes::editor_commands::{EditorCommand, supports_command};

#[test]
fn formatting_commands_are_only_available_in_rich_text() {
    assert!(supports_command(&EditorMode::Rich, EditorCommand::Bold));
    assert!(supports_command(
        &EditorMode::Rich,
        EditorCommand::ToggleBulletList
    ));
    assert!(!supports_command(
        &EditorMode::PlainText,
        EditorCommand::Bold
    ));
    assert!(!supports_command(
        &EditorMode::Code,
        EditorCommand::ToggleBulletList
    ));
}

#[test]
fn history_commands_are_available_for_all_editor_modes() {
    for mode in [
        EditorMode::Rich,
        EditorMode::Markdown,
        EditorMode::PlainText,
        EditorMode::Code,
    ] {
        assert!(supports_command(&mode, EditorCommand::Undo));
        assert!(supports_command(&mode, EditorCommand::Redo));
    }
}
