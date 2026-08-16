use adw::prelude::*;
use gtk::gdk;
use noor_notes::writing_assistance::PredictionOverlay;

#[test]
fn ghost_text_is_transient_keyboard_accessible_and_bounded() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text("clear support ");
    buffer.place_cursor(&buffer.end_iter());
    let view = gtk::TextView::with_buffer(&buffer);
    let canvas = gtk::Overlay::new();
    canvas.set_child(Some(&view));
    let window = gtk::Window::new();
    window.set_child(Some(&canvas));
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}
    let overlay = PredictionOverlay::new(&canvas, &view);

    assert_eq!(
        overlay.handle_key(gdk::Key::Tab, gdk::ModifierType::empty()),
        gtk::glib::Propagation::Proceed
    );
    overlay.show(&[
        "helps".into(),
        "works".into(),
        "grows".into(),
        "extra".into(),
    ]);
    assert_eq!(overlay.suggestions().len(), 3);
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        "clear support "
    );

    assert_eq!(
        overlay.handle_key(gdk::Key::Down, gdk::ModifierType::ALT_MASK),
        gtk::glib::Propagation::Stop
    );
    overlay.handle_key(gdk::Key::Down, gdk::ModifierType::empty());
    overlay.handle_key(gdk::Key::Return, gdk::ModifierType::empty());
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        "clear support works"
    );
    buffer.undo();
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        "clear support "
    );

    overlay.show(&["helps".into()]);
    overlay.handle_key(gdk::Key::Escape, gdk::ModifierType::empty());
    assert!(!overlay.is_visible());
    assert_eq!(overlay.announcement(), "Suggestion dismissed");
    window.close();
}
