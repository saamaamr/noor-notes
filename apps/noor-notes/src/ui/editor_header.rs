use adw::prelude::*;
use noor_domain::Note;

use crate::save_status::SaveStatusIndicator;

use super::editor_toolbar::EditorToolbar;

#[derive(Clone)]
pub struct EditorHeader {
    pub widget: adw::HeaderBar,
    pub title_box: gtk::Box,
    pub title_entry: gtk::Entry,
    pub save_status: SaveStatusIndicator,
    pub library_pin: gtk::ToggleButton,
    pub favorite: gtk::ToggleButton,
    pub appearance_button: gtk::Button,
}

impl EditorHeader {
    pub fn new(
        note: &Note,
        toolbar: &EditorToolbar,
        appearance_button: &gtk::Button,
        is_trashed: bool,
    ) -> Self {
        let widget = adw::HeaderBar::new();
        widget.add_css_class("nn-editor-header");

        let title_entry = gtk::Entry::builder()
            .text(note.display_title())
            .placeholder_text("Untitled note")
            .tooltip_text("Note title")
            .editable(!is_trashed)
            .hexpand(true)
            .width_chars(32)
            .build();
        title_entry.add_css_class("nn-editor-title");
        title_entry.update_property(&[gtk::accessible::Property::Label("Note title")]);

        let save_status = SaveStatusIndicator::new();
        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        title_box.add_css_class("nn-editor-title-box");
        title_box.set_hexpand(true);
        title_box.append(&title_entry);
        title_box.append(&save_status.widget);
        widget.set_title_widget(Some(&title_box));

        let library_pin = gtk::ToggleButton::builder()
            .icon_name("view-pin-symbolic")
            .tooltip_text("Pin note in the library")
            .active(note.pinned)
            .build();
        library_pin.update_property(&[gtk::accessible::Property::Label(
            "Pin note in the library",
        )]);
        let favorite = gtk::ToggleButton::builder()
            .icon_name(if note.favorite {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            })
            .tooltip_text("Add to favorites")
            .active(note.favorite)
            .build();
        favorite.update_property(&[gtk::accessible::Property::Label("Add to favorites")]);

        widget.pack_end(&toolbar.header_trash);
        widget.pack_end(&toolbar.header_archive);
        widget.pack_end(&toolbar.appearance);
        widget.pack_end(appearance_button);
        widget.pack_end(&favorite);
        widget.pack_end(&library_pin);

        Self {
            widget,
            title_box,
            title_entry,
            save_status,
            library_pin,
            favorite,
            appearance_button: appearance_button.clone(),
        }
    }
}
