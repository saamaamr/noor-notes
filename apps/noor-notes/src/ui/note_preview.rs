use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use noor_domain::{EditorMode, Note, NoteId, NoteState};

use crate::appearance::{EffectiveTheme, try_global};
use crate::rich_buffer::RichBuffer;

use super::editor_canvas::configure_editor_canvas;

type BodyEditHandler = Rc<dyn Fn(Note)>;
type EditFinishedHandler = Rc<dyn Fn(NoteId)>;

#[derive(Clone)]
pub struct NotePreview {
    pub widget: gtk::ScrolledWindow,
    title: gtk::Label,
    metadata: gtk::Label,
    body: gtk::Label,
    body_stack: gtk::Stack,
    editor: gtk::TextView,
    edit: gtk::Button,
    current: Rc<RefCell<Option<Note>>>,
    editing: Rc<Cell<bool>>,
    on_edit_finished: EditFinishedHandler,
}

impl NotePreview {
    pub fn new() -> Self {
        Self::new_with_handlers(Rc::new(|_| {}), Rc::new(|_| {}))
    }

    pub fn new_with_handler(on_body_edited: BodyEditHandler) -> Self {
        Self::new_with_handlers(on_body_edited, Rc::new(|_| {}))
    }

    pub fn new_with_handlers(
        on_body_edited: BodyEditHandler,
        on_edit_finished: EditFinishedHandler,
    ) -> Self {
        let document = gtk::Box::new(gtk::Orientation::Vertical, 16);
        document.add_css_class("nn-preview");
        document.set_valign(gtk::Align::Start);

        let heading = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        heading.add_css_class("nn-preview-heading");
        let title = gtk::Label::new(Some("Select a note"));
        title.add_css_class("nn-display-title");
        title.add_css_class("nn-preview-title");
        title.set_xalign(0.0);
        title.set_hexpand(true);
        title.set_wrap(true);
        title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        let title_attributes = gtk::pango::AttrList::new();
        title_attributes.insert(gtk::pango::AttrFloat::new_line_height(1.2));
        title.set_attributes(Some(&title_attributes));
        heading.append(&title);
        let edit = gtk::Button::builder()
            .label("Edit")
            .icon_name("document-edit-symbolic")
            .tooltip_text("Edit note body")
            .visible(false)
            .build();
        edit.add_css_class("flat");
        edit.add_css_class("nn-preview-edit");
        edit.update_property(&[gtk::accessible::Property::Label("Edit note body")]);
        edit.set_valign(gtk::Align::Start);
        heading.append(&edit);
        document.append(&heading);

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

        let editor = gtk::TextView::new();
        editor.add_css_class("nn-preview-editor");
        editor.set_editable(false);
        editor.set_cursor_visible(false);
        editor.set_wrap_mode(gtk::WrapMode::WordChar);
        editor.set_hexpand(true);
        configure_editor_canvas(&editor, true);
        let buffer = editor.buffer();
        if let Some(appearance) = try_global() {
            let buffer_weak = buffer.downgrade();
            appearance.subscribe(move |_, theme| {
                if let Some(buffer) = buffer_weak.upgrade() {
                    RichBuffer::apply_color_theme(&buffer, theme);
                }
            });
        }

        let body_stack = gtk::Stack::new();
        body_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        body_stack.set_transition_duration(140);
        body_stack.add_named(&body, Some("preview"));
        body_stack.add_named(&editor, Some("editor"));
        body_stack.set_visible_child_name("preview");
        document.append(&body_stack);

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

        let current = Rc::new(RefCell::new(None::<Note>));
        let editing = Rc::new(Cell::new(false));
        {
            let current = current.clone();
            let editing = editing.clone();
            let body_stack = body_stack.clone();
            let editor = editor.clone();
            let on_body_edited = on_body_edited.clone();
            let on_edit_finished = on_edit_finished.clone();
            edit.connect_clicked(move |button| {
                let can_edit = current
                    .borrow()
                    .as_ref()
                    .is_some_and(|note| !matches!(note.state, NoteState::Trashed { .. }));
                if !can_edit {
                    return;
                }
                let enabled = !editing.get();
                let exited_view_only = if enabled {
                    let mut current = current.borrow_mut();
                    current.as_mut().and_then(|note| {
                        if note.editor_preferences.view_only {
                            note.editor_preferences.view_only = false;
                            Some(note.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };
                if let Some(note) = exited_view_only {
                    on_body_edited(note);
                }
                set_editing(&editing, &body_stack, &editor, button, enabled);
                if !enabled {
                    if let Some(id) = current.borrow().as_ref().map(|note| note.id) {
                        on_edit_finished(id);
                    }
                }
            });
        }
        {
            let current = current.clone();
            let editing = editing.clone();
            let body = body.clone();
            buffer.connect_changed(move |buffer| {
                if !editing.get() {
                    return;
                }
                let Some(mut note) = current.borrow().clone() else {
                    return;
                };
                if note.editor_mode == EditorMode::Rich {
                    let (content, rich_content) = RichBuffer::snapshot(buffer);
                    note.content = content;
                    note.rich_content = Some(rich_content);
                } else {
                    note.content = buffer
                        .text(&buffer.start_iter(), &buffer.end_iter(), true)
                        .to_string();
                    note.rich_content = None;
                }
                body.set_text(&note.content);
                current.replace(Some(note.clone()));
                on_body_edited(note);
            });
        }

        Self {
            widget,
            title,
            metadata,
            body,
            body_stack,
            editor,
            edit,
            current,
            editing,
            on_edit_finished,
        }
    }

    pub fn clear(&self) {
        self.finish_pending_edit();
        self.current.replace(None);
        set_editing(
            &self.editing,
            &self.body_stack,
            &self.editor,
            &self.edit,
            false,
        );
        self.edit.set_visible(false);
        self.title.set_text("Select a note");
        self.metadata.set_text("Your note preview will appear here");
        self.body
            .set_text("Choose a note from the library to read it without opening another window.");
        self.editor.buffer().set_text("");
    }

    pub fn show_note(&self, note: &Note) {
        self.finish_pending_edit();
        set_editing(
            &self.editing,
            &self.body_stack,
            &self.editor,
            &self.edit,
            false,
        );
        self.current.replace(Some(note.clone()));
        self.edit.set_visible(true);
        self.edit
            .set_sensitive(!matches!(note.state, NoteState::Trashed { .. }));
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

        let rich_mode = note.editor_mode == EditorMode::Rich;
        configure_editor_canvas(&self.editor, rich_mode);
        self.editor
            .set_monospace(note.editor_mode == EditorMode::Code);
        self.editor.set_wrap_mode(gtk::WrapMode::WordChar);
        let buffer = self.editor.buffer();
        if rich_mode {
            RichBuffer::load(&buffer, &note.content, note.rich_content.as_ref());
            RichBuffer::apply_color_theme(
                &buffer,
                try_global()
                    .map(|appearance| appearance.effective_theme())
                    .unwrap_or(EffectiveTheme::Light),
            );
        } else {
            buffer.set_text(&note.content);
        }
    }

    pub fn set_compact(&self, compact: bool) {
        if compact {
            self.widget.add_css_class("compact");
        } else {
            self.widget.remove_css_class("compact");
        }
    }

    fn finish_pending_edit(&self) {
        if self.editing.get() {
            if let Some(id) = self.current.borrow().as_ref().map(|note| note.id) {
                (self.on_edit_finished)(id);
            }
        }
    }
}

impl Default for NotePreview {
    fn default() -> Self {
        Self::new()
    }
}

fn set_editing(
    editing: &Cell<bool>,
    body_stack: &gtk::Stack,
    editor: &gtk::TextView,
    edit: &gtk::Button,
    enabled: bool,
) {
    editing.set(enabled);
    editor.set_editable(enabled);
    editor.set_cursor_visible(enabled);
    body_stack.set_visible_child_name(if enabled { "editor" } else { "preview" });
    edit.set_label(if enabled { "Done" } else { "Edit" });
    let accessible_label = if enabled {
        "Finish editing note body"
    } else {
        "Edit note body"
    };
    edit.set_tooltip_text(Some(accessible_label));
    edit.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
    if enabled {
        editor.grab_focus();
    }
}
