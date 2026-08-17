use adw::prelude::*;
use noor_domain::Note;

#[derive(Clone)]
pub struct NotePreview {
    pub widget: gtk::ScrolledWindow,
    title: gtk::Label,
    metadata: gtk::Label,
    body: gtk::Label,
}

impl NotePreview {
    pub fn new() -> Self {
        let document = gtk::Box::new(gtk::Orientation::Vertical, 16);
        document.add_css_class("nn-preview");
        document.set_valign(gtk::Align::Start);
        let title = gtk::Label::new(Some("Select a note"));
        title.add_css_class("nn-display-title");
        title.add_css_class("nn-preview-title");
        title.set_xalign(0.0);
        title.set_wrap(true);
        title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        let title_attributes = gtk::pango::AttrList::new();
        title_attributes.insert(gtk::pango::AttrFloat::new_line_height(1.2));
        title.set_attributes(Some(&title_attributes));
        document.append(&title);
        let metadata = gtk::Label::new(Some("Your note preview will appear here"));
        metadata.add_css_class("nn-metadata");
        metadata.add_css_class("nn-preview-metadata");
        metadata.set_xalign(0.0);
        metadata.set_wrap(true);
        metadata.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        document.append(&metadata);
        document.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let body = gtk::Label::new(Some(
            "Choose a note from the library to read it without opening another window.",
        ));
        body.add_css_class("nn-body");
        body.add_css_class("nn-preview-body");
        body.set_xalign(0.0);
        body.set_yalign(0.0);
        body.set_wrap(true);
        body.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        let body_attributes = gtk::pango::AttrList::new();
        body_attributes.insert(gtk::pango::AttrFloat::new_line_height(1.6));
        body.set_attributes(Some(&body_attributes));
        body.set_selectable(true);
        document.append(&body);
        let clamp = adw::Clamp::builder()
            .maximum_size(860)
            .tightening_threshold(720)
            .child(&document)
            .build();
        clamp.set_hexpand(true);
        clamp.set_halign(gtk::Align::Start);
        let widget = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&clamp)
            .build();
        widget.add_css_class("nn-preview-surface");
        Self {
            widget,
            title,
            metadata,
            body,
        }
    }

    pub fn clear(&self) {
        self.title.set_text("Select a note");
        self.metadata.set_text("Your note preview will appear here");
        self.body
            .set_text("Choose a note from the library to read it without opening another window.");
    }

    pub fn show_note(&self, note: &Note) {
        self.title.set_text(note.display_title());
        self.metadata.set_text(&format!(
            "Edited {}{}",
            note.updated_at.format("%d %B %Y · %I:%M %p"),
            if note.tags.is_empty() {
                String::new()
            } else {
                format!(
                    "  ·  {}",
                    note.tags
                        .iter()
                        .map(|tag| format!("#{tag}"))
                        .collect::<Vec<_>>()
                        .join("  ")
                )
            }
        ));
        self.body.set_text(if note.content.trim().is_empty() {
            "This note is empty."
        } else {
            &note.content
        });
    }

    pub fn set_compact(&self, compact: bool) {
        if compact {
            self.widget.add_css_class("compact");
        } else {
            self.widget.remove_css_class("compact");
        }
    }
}

impl Default for NotePreview {
    fn default() -> Self {
        Self::new()
    }
}
