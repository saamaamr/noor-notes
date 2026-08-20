use gtk::prelude::*;
use noor_notes::editor_actions;
use noor_notes::rich_buffer::RichBuffer;
use noor_notes::ui::editor_toolbar::EditorToolbar;

fn text(buffer: &gtk::TextBuffer) -> String {
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), true)
        .to_string()
}

#[test]
fn emoji_inserts_at_preserved_cursor_closes_picker_and_is_undoable() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    RichBuffer::load(&buffer, "Hello  world", None);
    let editor = gtk::TextView::with_buffer(&buffer);
    let toolbar = EditorToolbar::new();
    editor_actions::connect(&toolbar, &buffer, &editor);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&toolbar.widget);
    content.append(&editor);
    let window = gtk::Window::builder().child(&content).build();
    window.present();
    settle();
    buffer.place_cursor(&buffer.iter_at_offset(6));

    toolbar.emoji.popup();
    settle();
    assert!(toolbar.emoji_popover.is_visible());
    toolbar.emoji_buttons[0].emit_clicked();

    assert_eq!(text(&buffer), "Hello 😀 world");
    assert_eq!(buffer.iter_at_mark(&buffer.get_insert()).offset(), 7);
    assert!(!toolbar.emoji_popover.is_visible());
    toolbar.undo.emit_clicked();
    assert_eq!(text(&buffer), "Hello  world");
    window.close();
}

fn settle() {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
}
