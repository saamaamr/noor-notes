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
