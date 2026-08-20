use noor_domain::EditorMode;
use noor_notes::editor::AdapterCapabilities;
use noor_notes::editor_commands::{EditorCommand, is_available, spec, supports_command};

#[test]
fn every_editor_command_has_one_real_capability_contract() {
    let bold = spec(EditorCommand::Bold);
    assert_eq!(bold.id, "bold");
    assert_eq!(bold.label, "Bold");
    assert_eq!(bold.shortcut, Some("Ctrl+B"));
    assert!(bold.mutates_document);

    let emoji = spec(EditorCommand::InsertEmoji);
    assert_eq!(emoji.id, "insert-emoji");
    assert!(supports_command(&EditorMode::Rich, emoji.command));
    assert!(supports_command(&EditorMode::Markdown, emoji.command));
    assert!(!supports_command(&EditorMode::Code, emoji.command));
}

#[test]
fn read_only_blocks_every_document_mutation() {
    let capabilities = AdapterCapabilities::all();
    for command in [
        EditorCommand::Undo,
        EditorCommand::Redo,
        EditorCommand::Bold,
        EditorCommand::Italic,
        EditorCommand::Underline,
        EditorCommand::Strikethrough,
        EditorCommand::ToggleBulletList,
        EditorCommand::ToggleNumberedList,
        EditorCommand::ClearFormatting,
        EditorCommand::InsertEmoji,
        EditorCommand::FontSize,
    ] {
        assert!(spec(command).mutates_document);
        assert!(!is_available(
            command,
            &EditorMode::Rich,
            capabilities,
            false,
        ));
    }
}

#[test]
fn availability_requires_both_mode_and_adapter_capability() {
    let no_formatting = AdapterCapabilities {
        formatting: false,
        ..AdapterCapabilities::all()
    };
    assert!(!is_available(
        EditorCommand::Bold,
        &EditorMode::Rich,
        no_formatting,
        true,
    ));
    assert!(!is_available(
        EditorCommand::Bold,
        &EditorMode::PlainText,
        AdapterCapabilities::all(),
        true,
    ));
    assert!(is_available(
        EditorCommand::InsertEmoji,
        &EditorMode::Markdown,
        AdapterCapabilities::all(),
        true,
    ));
}

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
