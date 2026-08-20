use gtk::prelude::*;
use noor_notes::editor_actions;
use noor_notes::editor_commands::{EditorCommand, execute};
use noor_notes::rich_buffer::{RichBuffer, SavedTextRange};
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn editor_history_undoes_and_redoes_real_buffer_edits() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    RichBuffer::prepare(&buffer);
    buffer.insert_at_cursor("hello");
    assert!(RichBuffer::can_undo(&buffer));
    RichBuffer::undo(&buffer);
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        ""
    );
    assert!(RichBuffer::can_redo(&buffer));
    RichBuffer::redo(&buffer);
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        "hello"
    );

    buffer.set_text("alpha beta");
    buffer.select_range(&buffer.iter_at_offset(0), &buffer.iter_at_offset(5));
    let saved = SavedTextRange::capture(&buffer);
    buffer.place_cursor(&buffer.end_iter());
    saved.restore(&buffer);
    assert!(execute(EditorCommand::Bold, &buffer, None));
    buffer.place_cursor(&buffer.iter_at_offset(2));
    assert!(RichBuffer::marks_at_cursor(&buffer).bold);

    buffer.select_range(&buffer.iter_at_offset(0), &buffer.iter_at_offset(5));
    assert!(execute(EditorCommand::Underline, &buffer, None));
    assert!(execute(EditorCommand::Undo, &buffer, None));
    buffer.place_cursor(&buffer.iter_at_offset(2));
    assert!(!RichBuffer::marks_at_cursor(&buffer).underline);
    assert!(execute(EditorCommand::Redo, &buffer, None));
    buffer.place_cursor(&buffer.iter_at_offset(2));
    assert!(RichBuffer::marks_at_cursor(&buffer).underline);

    let editor = gtk::TextView::with_buffer(&buffer);
    let toolbar = EditorToolbar::new();
    editor_actions::connect(&toolbar, &buffer, &editor);
    buffer.place_cursor(&buffer.iter_at_offset(2));
    assert!(toolbar.bold.is_active());
    assert!(toolbar.quick_underline.is_active());
    buffer.place_cursor(&buffer.iter_at_offset(8));
    assert!(!toolbar.bold.is_active());
    assert!(!toolbar.quick_underline.is_active());

    let action_buffer = gtk::TextBuffer::new(None);
    RichBuffer::prepare(&action_buffer);
    action_buffer.set_text("toolbar action");
    let action_editor = gtk::TextView::with_buffer(&action_buffer);
    let action_toolbar = EditorToolbar::new();
    editor_actions::connect(&action_toolbar, &action_buffer, &action_editor);
    action_buffer.select_range(
        &action_buffer.iter_at_offset(0),
        &action_buffer.iter_at_offset(7),
    );
    action_toolbar.bold.set_active(true);
    assert!(RichBuffer::marks_at_cursor(&action_buffer).bold);
    assert!(action_toolbar.undo.is_sensitive());
    action_toolbar.undo.emit_clicked();
    action_buffer.place_cursor(&action_buffer.iter_at_offset(2));
    assert!(!RichBuffer::marks_at_cursor(&action_buffer).bold);
    assert!(action_toolbar.redo.is_sensitive());
    action_toolbar.redo.emit_clicked();
    action_buffer.place_cursor(&action_buffer.iter_at_offset(2));
    assert!(RichBuffer::marks_at_cursor(&action_buffer).bold);
}
