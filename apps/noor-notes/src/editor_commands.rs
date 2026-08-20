use crate::editor::AdapterCapabilities;
use crate::rich_buffer::RichBuffer;
use noor_domain::EditorMode;
use noor_domain::ListKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorCommand {
    Undo,
    Redo,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    ToggleBulletList,
    ToggleNumberedList,
    ClearFormatting,
    InsertEmoji,
    FontSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorCommandSpec {
    pub command: EditorCommand,
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub mutates_document: bool,
}

pub const fn spec(command: EditorCommand) -> EditorCommandSpec {
    let (id, label, shortcut) = match command {
        EditorCommand::Undo => ("undo", "Undo", Some("Ctrl+Z")),
        EditorCommand::Redo => ("redo", "Redo", Some("Ctrl+Shift+Z")),
        EditorCommand::Bold => ("bold", "Bold", Some("Ctrl+B")),
        EditorCommand::Italic => ("italic", "Italic", Some("Ctrl+I")),
        EditorCommand::Underline => ("underline", "Underline", Some("Ctrl+U")),
        EditorCommand::Strikethrough => ("strikethrough", "Strikethrough", None),
        EditorCommand::ToggleBulletList => ("bullet-list", "Bullet List", None),
        EditorCommand::ToggleNumberedList => ("numbered-list", "Numbered List", None),
        EditorCommand::ClearFormatting => ("clear-formatting", "Clear Formatting", None),
        EditorCommand::InsertEmoji => ("insert-emoji", "Emoji", None),
        EditorCommand::FontSize => ("font-size", "Font Size", None),
    };
    EditorCommandSpec {
        command,
        id,
        label,
        shortcut,
        mutates_document: true,
    }
}

pub fn supports_command(mode: &EditorMode, command: EditorCommand) -> bool {
    match command {
        EditorCommand::Undo | EditorCommand::Redo => true,
        EditorCommand::Bold
        | EditorCommand::Italic
        | EditorCommand::Underline
        | EditorCommand::Strikethrough
        | EditorCommand::ToggleBulletList
        | EditorCommand::ToggleNumberedList
        | EditorCommand::ClearFormatting
        | EditorCommand::FontSize => matches!(mode, EditorMode::Rich),
        EditorCommand::InsertEmoji => !matches!(mode, EditorMode::Code),
    }
}

pub fn is_available(
    command: EditorCommand,
    mode: &EditorMode,
    capabilities: AdapterCapabilities,
    editable: bool,
) -> bool {
    let command_spec = spec(command);
    if command_spec.mutates_document && !editable {
        return false;
    }
    if !supports_command(mode, command) {
        return false;
    }

    match command {
        EditorCommand::Undo => capabilities.undo,
        EditorCommand::Redo => capabilities.redo,
        EditorCommand::Bold
        | EditorCommand::Italic
        | EditorCommand::Underline
        | EditorCommand::Strikethrough
        | EditorCommand::ToggleBulletList
        | EditorCommand::ToggleNumberedList
        | EditorCommand::ClearFormatting
        | EditorCommand::FontSize => capabilities.formatting,
        EditorCommand::InsertEmoji => true,
    }
}

pub fn execute(command: EditorCommand, buffer: &gtk::TextBuffer, argument: Option<&str>) -> bool {
    match command {
        EditorCommand::Undo => RichBuffer::undo(buffer),
        EditorCommand::Redo => RichBuffer::redo(buffer),
        EditorCommand::Bold => RichBuffer::bold(buffer),
        EditorCommand::Italic => RichBuffer::italic(buffer),
        EditorCommand::Underline => RichBuffer::underline(buffer),
        EditorCommand::Strikethrough => RichBuffer::strikethrough(buffer),
        EditorCommand::ToggleBulletList => RichBuffer::toggle_list(buffer, ListKind::Bullet),
        EditorCommand::ToggleNumberedList => RichBuffer::toggle_list(buffer, ListKind::Numbered),
        EditorCommand::ClearFormatting => RichBuffer::clear_formatting(buffer),
        EditorCommand::InsertEmoji => {
            let Some(value) = argument else {
                return false;
            };
            RichBuffer::insert_emoji(buffer, value);
        }
        EditorCommand::FontSize => {
            let Some(value) = argument.and_then(RichBuffer::parse_font_size) else {
                return false;
            };
            RichBuffer::font_size(buffer, value);
        }
    }
    true
}
