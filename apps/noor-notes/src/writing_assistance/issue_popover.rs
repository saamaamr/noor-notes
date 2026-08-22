use adw::prelude::*;

#[derive(Clone)]
pub struct IssuePopover {
    pub widget: gtk::Popover,
    text: String,
    pub replacements: Vec<gtk::Button>,
    pub ignore_once: gtk::Button,
}

impl IssuePopover {
    pub fn new(category: &str, message: &str, replacements: &[String]) -> Self {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        let category_label = gtk::Label::new(Some(category));
        category_label.add_css_class("heading");
        category_label.set_halign(gtk::Align::Start);
        let message_label = gtk::Label::new(Some(message));
        message_label.set_wrap(true);
        message_label.set_max_width_chars(42);
        message_label.set_halign(gtk::Align::Start);
        content.append(&category_label);
        content.append(&message_label);
        let mut buttons = Vec::new();
        for replacement in replacements.iter().take(5) {
            let display = if replacement.is_empty() {
                "Remove"
            } else {
                replacement
            };
            let button = gtk::Button::with_label(display);
            button.update_property(&[gtk::accessible::Property::Label(&format!(
                "Replace with {display}"
            ))]);
            content.append(&button);
            buttons.push(button);
        }
        let ignore_once = gtk::Button::with_label("Ignore once");
        content.append(&ignore_once);
        let widget = crate::ui::popover_primitives::themed_popover(&content);
        widget.set_focusable(true);
        let text = format!(
            "{category}\n{message}\n{}\nIgnore once",
            replacements.join("\n")
        );
        Self {
            widget,
            text,
            replacements: buttons,
            ignore_once,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}
