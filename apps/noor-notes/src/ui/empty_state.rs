use adw::prelude::*;

use crate::library::LibrarySection;

#[derive(Clone)]
pub struct EmptyState {
    pub widget: gtk::Box,
    icon: gtk::Image,
    title: gtk::Label,
    description: gtk::Label,
}

impl EmptyState {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 12);
        widget.add_css_class("nn-empty-state");
        widget.set_valign(gtk::Align::Center);
        widget.set_halign(gtk::Align::Center);
        let icon = gtk::Image::from_icon_name("note-symbolic");
        icon.add_css_class("nn-empty-state-icon");
        icon.set_pixel_size(40);
        widget.append(&icon);
        let title = gtk::Label::new(None);
        title.add_css_class("nn-section-title");
        widget.append(&title);
        let description = gtk::Label::new(None);
        description.add_css_class("nn-metadata");
        description.set_wrap(true);
        description.set_justify(gtk::Justification::Center);
        description.set_max_width_chars(42);
        widget.append(&description);
        Self {
            widget,
            icon,
            title,
            description,
        }
    }

    pub fn update(&self, section: LibrarySection, searching: bool) {
        if searching {
            self.icon.set_icon_name(Some("system-search-symbolic"));
            self.title.set_text("No notes found");
            self.description
                .set_text("Try another word or clear the search.");
            self.update_accessible_label();
            return;
        }
        let (title, description) = match section {
            LibrarySection::AllNotes => ("No notes yet", "Create a note to begin writing."),
            LibrarySection::Pinned => ("No pinned notes", "Pin important notes for quick access."),
            LibrarySection::Favorites => {
                ("No favorite notes", "Favorite notes you want to revisit.")
            }
            LibrarySection::Tags => ("No tagged notes", "Add tags to organize your library."),
            LibrarySection::Archived => {
                ("Archive is empty", "Archived notes remain available here.")
            }
            LibrarySection::Trash => ("Trash is empty", "Deleted notes remain recoverable here."),
            LibrarySection::Recent => ("No recent notes", "Recently edited notes appear here."),
        };
        self.icon.set_icon_name(Some(section.icon_name()));
        self.title.set_text(title);
        self.description.set_text(description);
        self.update_accessible_label();
    }

    pub fn title_text(&self) -> String {
        self.title.text().to_string()
    }

    pub fn description_text(&self) -> String {
        self.description.text().to_string()
    }

    pub fn icon_name(&self) -> Option<gtk::glib::GString> {
        self.icon.icon_name()
    }

    fn update_accessible_label(&self) {
        self.icon
            .update_property(&[gtk::accessible::Property::Label(self.title.text().as_str())]);
    }
}

impl Default for EmptyState {
    fn default() -> Self {
        Self::new()
    }
}
