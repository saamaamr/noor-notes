use adw::prelude::*;

/// A label/content/help stack that does not squeeze fields beside subtitles.
pub fn content_row(title: &str, help: &str, content: &impl IsA<gtk::Widget>) -> gtk::Box {
    let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    body.add_css_class("nn-settings-field");
    body.update_property(&[gtk::accessible::Property::Label(title)]);
    body.set_hexpand(true);
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    label.add_css_class("heading");
    content.set_valign(gtk::Align::Center);
    body.append(&label);
    body.append(content);
    if !help.is_empty() {
        let hint = gtk::Label::new(Some(help));
        hint.set_xalign(0.0);
        hint.set_wrap(true);
        hint.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        hint.add_css_class("nn-text-muted");
        hint.add_css_class("caption");
        body.append(&hint);
    }
    body
}
