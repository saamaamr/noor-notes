use gtk::prelude::*;
use noor_notes::rich_buffer::RichBuffer;

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
}
