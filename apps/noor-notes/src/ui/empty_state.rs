use adw::prelude::*;

use crate::library::LibrarySection;

#[derive(Clone)]
pub struct EmptyState {
    pub widget: gtk::Box,
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
        icon.set_pixel_size(48);
        widget.append(&icon);
        let title = gtk::Label::new(None);
        title.add_css_class("nn-section-title");
        widget.append(&title);
        let description = gtk::Label::new(None);
        description.add_css_class("nn-metadata");
        description.set_wrap(true);
        description.set_justify(gtk::Justification::Center);
        widget.append(&description);
        Self {
            widget,
            title,
            description,
        }
    }

    pub fn update(&self, section: LibrarySection, searching: bool) {
        if searching {
            self.title.set_text("No notes found");
            self.description
                .set_text("Try another word or clear the search.");
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
        self.title.set_text(title);
        self.description.set_text(description);
    }
}

impl Default for EmptyState {
    fn default() -> Self {
        Self::new()
    }
}
