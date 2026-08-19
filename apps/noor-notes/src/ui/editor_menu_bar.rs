use adw::prelude::*;

use super::editor_toolbar::EditorToolbar;

/// Compact document-editor menu row. Menu items proxy the existing toolbar
/// controls so menu, toolbar, and keyboard paths keep one implementation.
#[derive(Clone)]
pub struct EditorMenuBar {
    pub widget: gtk::Box,
}

impl EditorMenuBar {
    pub fn new(toolbar: &EditorToolbar) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        widget.add_css_class("nn-editor-menu-bar");
        for (label, items) in [
            (
                "File",
                vec![
                    ("New note", proxy(&toolbar.new_note)),
                    ("Duplicate", proxy(&toolbar.duplicate)),
                    ("Export", proxy(&toolbar.export_text)),
                    ("Delete", proxy(&toolbar.trash)),
                ],
            ),
            (
                "Edit",
                vec![
                    ("Undo", proxy(&toolbar.undo)),
                    ("Redo", proxy(&toolbar.redo)),
                    ("Find", proxy_toggle(&toolbar.find)),
                ],
            ),
            (
                "View",
                vec![
                    ("Word wrap", proxy_toggle(&toolbar.word_wrap)),
                    ("Zoom in", proxy(&toolbar.zoom_in)),
                    ("Zoom out", proxy(&toolbar.zoom_out)),
                    ("Reset zoom", proxy(&toolbar.zoom_reset)),
                    ("View only", proxy(&toolbar.view_only)),
                ],
            ),
            ("Insert", vec![("Emoji", proxy_menu(&toolbar.emoji))]),
            (
                "Format",
                vec![
                    ("Bold", proxy_toggle(&toolbar.bold)),
                    ("Italic", proxy_toggle(&toolbar.italic)),
                    ("Underline", proxy_toggle(&toolbar.quick_underline)),
                    ("Strikethrough", proxy_toggle(&toolbar.quick_strikethrough)),
                    ("Bullet list", proxy_toggle(&toolbar.bullets)),
                    ("Numbered list", proxy_toggle(&toolbar.quick_numbered)),
                    ("More formatting…", proxy_menu(&toolbar.format)),
                ],
            ),
            (
                "Tools",
                vec![
                    ("Go to line", proxy(&toolbar.go_to_line)),
                    ("Editor mode", proxy_menu(&toolbar.more)),
                    ("More actions…", proxy_menu(&toolbar.more)),
                ],
            ),
        ] {
            widget.append(&menu_button(label, items));
        }
        Self { widget }
    }
}

fn menu_button(label: &str, items: Vec<(&str, gtk::Button)>) -> gtk::MenuButton {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
    content.add_css_class("nn-editor-menu-popover");
    for (item_label, button) in items {
        button.set_label(item_label);
        button.set_halign(gtk::Align::Fill);
        button.set_hexpand(true);
        content.append(&button);
    }
    let popover = gtk::Popover::builder().child(&content).build();
    let menu = gtk::MenuButton::builder()
        .label(label)
        .popover(&popover)
        .tooltip_text(label)
        .build();
    menu.add_css_class("nn-editor-menu-button");
    menu.update_property(&[gtk::accessible::Property::Label(label)]);
    menu
}

fn proxy(button: &gtk::Button) -> gtk::Button {
    let proxy = gtk::Button::new();
    let source = button.clone();
    proxy.connect_clicked(move |_| source.emit_clicked());
    proxy
}

fn proxy_toggle(button: &gtk::ToggleButton) -> gtk::Button {
    let proxy = gtk::Button::new();
    let source = button.clone();
    proxy.connect_clicked(move |_| source.set_active(!source.is_active()));
    proxy
}

fn proxy_menu(button: &gtk::MenuButton) -> gtk::Button {
    let proxy = gtk::Button::new();
    let source = button.clone();
    proxy.connect_clicked(move |_| source.popup());
    proxy
}
