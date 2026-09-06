use adw::prelude::*;
use noor_notes::appearance::{EffectiveTheme, semantic_stylesheet};
use noor_notes::ui::editor_toolbar::EditorToolbar;
use std::time::Duration;

#[test]
fn editor_controls_are_compact_readable_and_theme_safe() {
    adw::init().unwrap();
    let css = gtk::CssProvider::new();
    css.connect_parsing_error(|_, _, error| panic!("Invalid design system CSS: {error}"));
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    let toolbar = EditorToolbar::new();
    let container = gtk::Box::new(gtk::Orientation::Vertical, 12);
    container.set_margin_top(24);
    container.set_margin_start(24);
    container.append(&toolbar.widget);
    let window = gtk::Window::builder()
        .default_width(1100)
        .default_height(600)
        .child(&container)
        .build();
    window.present();
    for theme in EffectiveTheme::ALL {
        adw::StyleManager::default().set_color_scheme(if theme.is_light() {
            adw::ColorScheme::ForceLight
        } else {
            adw::ColorScheme::ForceDark
        });
        css.load_from_string(&semantic_stylesheet(theme));
        settle();
        for control in [
            toolbar.undo.upcast_ref::<gtk::Widget>(),
            toolbar.bold.upcast_ref(),
            toolbar.format.upcast_ref(),
            toolbar.emoji.upcast_ref(),
        ] {
            assert!(
                control.width() <= 40,
                "{} toolbar width {} exceeds compact icon target",
                control.type_(),
                control.width()
            );
            assert!(
                control.height() <= 40,
                "{} toolbar height {} exceeds compact target",
                control.type_(),
                control.height()
            );
        }
        toolbar.format.popup();
        settle();
        assert!(toolbar.formatting.widget.is_visible());
        assert!(toolbar.formatting.custom_font_size.grab_focus());
        toolbar.formatting.widget.popdown();
        toolbar.emoji.popup();
        settle();
        assert!(toolbar.emoji_popover.is_visible());
        toolbar.emoji_popover.popdown();
    }
    window.close();
}

fn settle() {
    for _ in 0..20 {
        while gtk::glib::MainContext::default().iteration(false) {}
        std::thread::sleep(Duration::from_millis(5));
    }
}
