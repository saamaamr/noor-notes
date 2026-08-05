use gtk::prelude::*;
use noor_domain::ListKind;
use noor_notes::rich_buffer::RichBuffer;

fn text(buffer: &gtk::TextBuffer) -> String {
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), true)
        .to_string()
}

#[test]
fn list_toggles_and_switches() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text("alpha\nbeta");
    buffer.select_range(&buffer.start_iter(), &buffer.end_iter());
    RichBuffer::toggle_list(&buffer, ListKind::Bullet);
    assert_eq!(text(&buffer), "• alpha\n• beta");
    buffer.select_range(&buffer.start_iter(), &buffer.end_iter());
    RichBuffer::toggle_list(&buffer, ListKind::Bullet);
    assert_eq!(text(&buffer), "alpha\nbeta");
    buffer.select_range(&buffer.start_iter(), &buffer.end_iter());
    RichBuffer::toggle_list(&buffer, ListKind::Numbered);
    assert_eq!(text(&buffer), "1. alpha\n2. beta");
    buffer.select_range(&buffer.start_iter(), &buffer.end_iter());
    RichBuffer::toggle_list(&buffer, ListKind::Bullet);
    assert_eq!(text(&buffer), "• alpha\n• beta");
    verify_enter_continues_and_exits();
}

fn verify_enter_continues_and_exits() {
    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text("3. item");
    buffer.place_cursor(&buffer.end_iter());
    assert!(RichBuffer::continue_list(&buffer));
    assert_eq!(text(&buffer), "3. item\n4. ");
    assert!(RichBuffer::continue_list(&buffer));
    assert_eq!(text(&buffer), "3. item\n");
}
