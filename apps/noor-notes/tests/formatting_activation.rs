use adw::prelude::*;
use noor_notes::ui::editor_menu_bar::EditorMenuBar;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn toolbar_and_format_menu_open_the_same_live_formatting_popover() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    let menu_bar = EditorMenuBar::new_preview(&toolbar);
    let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header_space = gtk::Box::new(gtk::Orientation::Vertical, 0);
    header_space.set_height_request(300);
    layout.append(&header_space);
    layout.append(&menu_bar.widget);
    layout.append(&toolbar.widget);
    let window = gtk::Window::builder()
        .default_width(1000)
        .default_height(900)
        .child(&layout)
        .build();
    window.present();
    settle();

    toolbar.format.popup();
    settle();
    assert!(
        toolbar.formatting.widget.is_visible(),
        "toolbar Formatting button must open the live formatting popover"
    );
    toolbar.formatting.widget.popdown();
    settle();

    menu_bar.item("format.more").emit_clicked();
    settle();
    assert!(
        toolbar.formatting.widget.is_visible(),
        "Format > More Formatting must open the same live formatting popover"
    );

    window.close();
}

fn settle() {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
}
