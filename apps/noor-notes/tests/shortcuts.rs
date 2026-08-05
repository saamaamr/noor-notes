use adw::prelude::*;
use noor_notes::shortcuts::shortcuts_window;

#[test]
fn shortcuts_reference_is_a_titled_focusable_dialog() {
    gtk::init().unwrap();
    let dialog = shortcuts_window();
    assert_eq!(dialog.title().as_deref(), Some("Keyboard Shortcuts"));
    assert!(dialog.can_focus());
}
