use std::cell::Cell;
use std::collections::HashMap;

use adw::prelude::*;

use crate::library::LibrarySection;

#[derive(Clone)]
pub struct LibrarySidebar {
    pub widget: gtk::Box,
    list: gtk::ListBox,
    counts: HashMap<LibrarySection, gtk::Label>,
    labels: Vec<gtk::Label>,
    heading: gtk::Label,
    privacy: gtk::Label,
    collapsed: Cell<bool>,
    expanded_width: Cell<i32>,
}

impl LibrarySidebar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("nn-sidebar");

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
        let mut labels = Vec::new();
        for section in LibrarySection::NAVIGATION {
            let row = gtk::ListBoxRow::new();
            row.add_css_class("nn-sidebar-row");
            row.set_height_request(40);
            row.set_tooltip_text(Some(section.label()));
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            let icon = gtk::Image::from_icon_name(section.icon_name());
            icon.add_css_class("nn-sidebar-icon");
            content.append(&icon);
            let label = gtk::Label::new(Some(section.label()));
            label.add_css_class("nn-sidebar-label");
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            content.append(&label);
            labels.push(label.clone());
            let count = gtk::Label::new(Some("0"));
            count.add_css_class("nn-caption");
            count.add_css_class("nn-sidebar-count");
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
            labels,
            heading,
            privacy,
            collapsed: Cell::new(false),
            expanded_width: Cell::new(160),
        }
    }

    pub fn set_allocated_width(&self, width: i32) {
        if width > 0 {
            self.expanded_width.set(width);
        }
        if !self.collapsed.get() {
            self.widget.set_width_request(width);
        }
    }

    pub fn set_collapsed(&self, collapsed: bool) {
        self.collapsed.set(collapsed);
        self.widget.set_width_request(if collapsed {
            64
        } else {
            self.expanded_width.get()
        });
        self.heading.set_visible(!collapsed);
        self.privacy.set_visible(!collapsed);
        for label in &self.labels {
            label.set_visible(!collapsed);
        }
        for count in self.counts.values() {
            count.set_visible(!collapsed);
        }
        if collapsed {
            self.widget.add_css_class("collapsed");
        } else {
            self.widget.remove_css_class("collapsed");
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
