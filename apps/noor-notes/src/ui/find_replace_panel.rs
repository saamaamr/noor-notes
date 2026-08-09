use adw::prelude::*;

#[derive(Clone)]
pub struct FindReplacePanel {
    pub widget: gtk::Box,
    pub find_entry: gtk::SearchEntry,
    pub replace_entry: gtk::Entry,
    pub match_case: gtk::CheckButton,
    pub whole_word: gtk::CheckButton,
    pub replace: gtk::Button,
    pub replace_all: gtk::Button,
    pub close: gtk::Button,
    pub previous: gtk::Button,
    pub next: gtk::Button,
    pub count: gtk::Label,
}

impl FindReplacePanel {
    pub fn new() -> Self {
        let find_entry = gtk::SearchEntry::builder()
            .placeholder_text("Find in note…")
            .tooltip_text("Find text")
            .hexpand(true)
            .build();
        find_entry.update_property(&[gtk::accessible::Property::Label("Find text")]);
        let replace_entry = gtk::Entry::builder()
            .placeholder_text("Replace with…")
            .tooltip_text("Replacement text")
            .hexpand(true)
            .build();
        replace_entry.update_property(&[gtk::accessible::Property::Label("Replacement text")]);

        let match_case = gtk::CheckButton::with_label("Match case");
        let whole_word = gtk::CheckButton::with_label("Whole word");
        let replace = gtk::Button::with_label("Replace");
        let replace_all = gtk::Button::with_label("Replace All");
        let close = icon_button("window-close-symbolic", "Close find and replace (Escape)");
        let previous = icon_button("go-up-symbolic", "Previous match");
        let next = icon_button("go-down-symbolic", "Next match");
        let count = gtk::Label::new(Some("0 of 0"));
        count.add_css_class("nn-caption");

        let find_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        find_row.append(&find_entry);
        find_row.append(&count);
        find_row.append(&previous);
        find_row.append(&next);
        find_row.append(&close);

        let replace_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        replace_row.append(&replace_entry);
        replace_row.append(&replace);
        replace_row.append(&replace_all);
        replace_row.append(&match_case);
        replace_row.append(&whole_word);

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
        widget.add_css_class("nn-find-panel");
        widget.append(&find_row);
        widget.append(&replace_row);
        widget.set_visible(false);

        Self {
            widget,
            find_entry,
            replace_entry,
            match_case,
            whole_word,
            replace,
            replace_all,
            close,
            previous,
            next,
            count,
        }
    }
}

impl Default for FindReplacePanel {
    fn default() -> Self {
        Self::new()
    }
}

fn icon_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(label)
        .build();
    button.add_css_class("flat");
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}
