use std::collections::HashMap;

use adw::prelude::*;

use crate::library::LibrarySection;

#[derive(Clone)]
pub struct LibrarySidebar {
    pub widget: gtk::Box,
    list: gtk::ListBox,
    counts: HashMap<LibrarySection, gtk::Label>,
}

impl LibrarySidebar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("nn-sidebar");
        widget.set_width_request(184);

        let heading = gtk::Label::new(Some("Library"));
        heading.add_css_class("nn-caption");
        heading.set_halign(gtk::Align::Start);
        heading.set_margin_start(12);
        widget.append(&heading);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.set_activate_on_single_click(true);
        list.add_css_class("navigation-sidebar");
        let mut counts = HashMap::new();
        for section in LibrarySection::NAVIGATION {
            let row = gtk::ListBoxRow::new();
            row.add_css_class("nn-sidebar-row");
            row.set_tooltip_text(Some(section.label()));
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            content.append(&gtk::Image::from_icon_name(section.icon_name()));
            let label = gtk::Label::new(Some(section.label()));
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            content.append(&label);
            let count = gtk::Label::new(Some("0"));
            count.add_css_class("nn-caption");
            content.append(&count);
            row.set_child(Some(&content));
            list.append(&row);
            counts.insert(section, count);
        }
        if let Some(first) = list.row_at_index(0) {
            list.select_row(Some(&first));
        }
        widget.append(&list);

        let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        widget.append(&spacer);
        let privacy = gtk::Label::new(Some("Private · Local only"));
        privacy.add_css_class("nn-caption");
        privacy.set_halign(gtk::Align::Start);
        privacy.set_margin_start(12);
        privacy.set_margin_bottom(8);
        widget.append(&privacy);
        Self {
            widget,
            list,
            counts,
        }
    }

    pub fn connect_selected<F: Fn(LibrarySection) + 'static>(&self, callback: F) {
        self.list.connect_row_selected(move |_, row| {
            let Some(row) = row else { return };
            let Some(section) = LibrarySection::NAVIGATION.get(row.index() as usize) else {
                return;
            };
            callback(*section);
        });
    }

    pub fn set_count(&self, section: LibrarySection, value: usize) {
        if let Some(label) = self.counts.get(&section) {
            label.set_text(&value.to_string());
        }
    }
}

impl Default for LibrarySidebar {
    fn default() -> Self {
        Self::new()
    }
}
