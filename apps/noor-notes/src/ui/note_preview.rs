use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use noor_domain::{EditorMode, Note, NoteId, NoteState};

use crate::appearance::{EffectiveTheme, try_global};
use crate::rich_buffer::RichBuffer;
use crate::ui::editor_menu_bar::EditorMenuBar;
use crate::ui::editor_toolbar::EditorToolbar;

use super::editor_canvas::configure_editor_canvas;

// NotePreview is the concrete implementation behind the shared
// `NoteEditorSurface` host boundary.

type BodyEditHandler = Rc<dyn Fn(Note)>;
type EditFinishedHandler = Rc<dyn Fn(NoteId)>;
type ReadOnlyHandler = Rc<dyn Fn(Note, bool)>;

#[derive(Clone)]
pub struct NotePreview {
    pub widget: gtk::ScrolledWindow,
    title: gtk::Label,
    title_entry: gtk::Entry,
    title_stack: gtk::Stack,
    metadata: gtk::Label,
    divider: gtk::Separator,
    body: gtk::Label,
    body_stack: gtk::Stack,
    editor: gtk::TextView,
    edit: gtk::Button,
    read_only: gtk::Button,
    toolbar: EditorToolbar,
    menu_bar: EditorMenuBar,
    current: Rc<RefCell<Option<Note>>>,
    editing: Rc<Cell<bool>>,
    on_edit_finished: EditFinishedHandler,
    on_read_only_changed: Rc<RefCell<Option<ReadOnlyHandler>>>,
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
        Self::new_with_all_handlers(on_body_edited, on_edit_finished)
    }

    fn new_with_all_handlers(
        on_body_edited: BodyEditHandler,
        on_edit_finished: EditFinishedHandler,
    ) -> Self {
        let document = gtk::Box::new(gtk::Orientation::Vertical, 16);
        document.add_css_class("nn-preview");
        document.set_valign(gtk::Align::Fill);
        document.set_vexpand(true);

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
        let title_entry = gtk::Entry::builder()
            .hexpand(true)
            .placeholder_text("Note title")
            .visible(true)
            .build();
        title_entry.add_css_class("nn-preview-title-entry");
        let title_stack = gtk::Stack::new();
        title_stack.add_named(&title, Some("label"));
        title_stack.add_named(&title_entry, Some("entry"));
        title_stack.set_hexpand(true);
        title_stack.set_visible_child_name("entry");
        heading.append(&title_stack);
        let edit = gtk::Button::builder()
            .label("Edit note")
            .icon_name("document-edit-symbolic")
            .tooltip_text("Edit note title and body")
            .visible(false)
            .build();
        edit.add_css_class("flat");
        edit.add_css_class("suggested-action");
        edit.add_css_class("nn-preview-edit");
        edit.update_property(&[gtk::accessible::Property::Label("Edit note body")]);
        edit.set_valign(gtk::Align::Start);
        heading.append(&edit);
        {
            let edit = edit.clone();
            let click = gtk::GestureClick::new();
            click.connect_released(move |_, _, _, _| edit.emit_clicked());
            title.add_controller(click);
            title.set_tooltip_text(Some("Click to edit title and note body"));
        }
        let read_only = gtk::Button::with_label("Read-only");
        read_only.set_tooltip_text(Some("Open this note in a read-only sticky window"));
        read_only.add_css_class("flat");
        read_only.add_css_class("secondary");
        read_only.add_css_class("nn-preview-read-only-button");
        read_only.add_css_class("nn-preview-read-only");
        read_only.update_property(&[gtk::accessible::Property::Label(
            "Open read-only sticky window",
        )]);
        read_only.set_valign(gtk::Align::Start);
        heading.append(&read_only);
        document.append(&heading);

        let metadata = gtk::Label::new(Some("Your note preview will appear here"));
        metadata.add_css_class("nn-metadata");
        metadata.add_css_class("nn-preview-metadata");
        metadata.set_xalign(0.0);
        metadata.set_wrap(true);
        metadata.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        document.append(&metadata);
        let divider = gtk::Separator::new(gtk::Orientation::Horizontal);
        document.append(&divider);

        let body = gtk::Label::new(Some(
            "Choose a note from the library to read it without opening another window.",
        ));
        body.add_css_class("nn-body");
        body.add_css_class("nn-preview-body");
        body.set_xalign(0.0);
        body.set_yalign(0.0);
        body.set_hexpand(true);
        body.set_vexpand(true);
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
        editor.set_vexpand(true);
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
        body_stack.set_hexpand(true);
        body_stack.set_vexpand(true);
        body_stack.add_named(&body, Some("preview"));
        body_stack.add_named(&editor, Some("editor"));
        body_stack.set_visible_child_name("preview");
        let toolbar = EditorToolbar::new();
        toolbar.widget.set_visible(false);
        // Preview exposes only controls that are wired to this editor surface.
        // Search, note settings, and multi-action menus remain available in the
        // full editor window and are intentionally not shown as dead buttons here.
        toolbar.find.set_visible(false);
        toolbar.more.set_visible(false);
        toolbar.new_note.set_visible(false);
        toolbar.format.set_tooltip_text(Some("Formatting"));
        toolbar.widget.add_css_class("nn-preview-format-toolbar");
        toolbar.widget.set_hexpand(false);
        toolbar.widget.set_halign(gtk::Align::Start);
        crate::editor_actions::connect(&toolbar, &buffer, &editor);
        toolbar.set_rich_formatting_enabled(false);
        let menu_bar = EditorMenuBar::new_preview(&toolbar);
        menu_bar.widget.set_visible(false);
        document.append(&menu_bar.widget);
        document.append(&toolbar.widget);
        document.append(&body_stack);

        let clamp = adw::Clamp::builder()
            .maximum_size(860)
            .tightening_threshold(720)
            .child(&document)
            .build();
        clamp.set_hexpand(true);
        clamp.set_vexpand(true);
        let widget = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&clamp)
            .build();
        widget.add_css_class("nn-preview-surface");

        let current = Rc::new(RefCell::new(None::<Note>));
        let editing = Rc::new(Cell::new(false));
        let on_read_only_changed = Rc::new(RefCell::new(None::<ReadOnlyHandler>));
        {
            let current = current.clone();
            let on_body_edited = on_body_edited.clone();
            let title = title.clone();
            title_entry.connect_changed(move |entry| {
                let Some(mut note) = current.borrow().clone() else {
                    return;
                };
                if note.editor_preferences.view_only
                    || matches!(note.state, NoteState::Trashed { .. })
                {
                    return;
                }
                note.title = entry.text().trim().to_string();
                title.set_text(if note.title.trim().is_empty() {
                    "Untitled note"
                } else {
                    note.title.as_str()
                });
                current.replace(Some(note.clone()));
                on_body_edited(note);
            });
        }
        {
            let current = current.clone();
            let editing = editing.clone();
            let body_stack = body_stack.clone();
            let editor = editor.clone();
            let title_entry = title_entry.clone();
            let title_stack = title_stack.clone();
            let toolbar = toolbar.clone();
            let menu_bar = menu_bar.clone();
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
                let changed_note = if enabled {
                    let mut current = current.borrow_mut();
                    current.as_mut().map(|note| {
                        note.editor_mode = EditorMode::Rich;
                        note.editor_preferences.view_only = false;
                        note.clone()
                    })
                } else {
                    None
                };
                if let Some(note) = changed_note {
                    on_body_edited(note);
                }
                set_editing(
                    &editing,
                    &body_stack,
                    &editor,
                    button,
                    &title_entry,
                    &title_stack,
                    &toolbar,
                    &menu_bar,
                    enabled,
                );
                if !enabled {
                    if let Some(id) = current.borrow().as_ref().map(|note| note.id) {
                        on_edit_finished(id);
                    }
                }
            });
        }
        {
            let current = current.clone();
            let read_only = read_only.clone();
            let read_only_button = read_only.clone();
            let on_read_only_changed = on_read_only_changed.clone();
            read_only_button.connect_clicked(move |_| {
                let Some(mut note) = current.borrow().clone() else {
                    return;
                };
                if matches!(note.state, NoteState::Trashed { .. }) {
                    return;
                }
                let enabled = !note.editor_preferences.view_only;
                note.editor_preferences.view_only = enabled;
                current.replace(Some(note.clone()));
                read_only.set_label(if enabled {
                    "Exit read-only"
                } else {
                    "Read-only"
                });
                read_only.update_property(&[gtk::accessible::Property::Label(if enabled {
                    "Close read-only sticky window"
                } else {
                    "Open read-only sticky window"
                })]);
                if let Some(handler) = on_read_only_changed.borrow().as_ref() {
                    handler(note, enabled);
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
            title_entry,
            title_stack,
            metadata,
            divider,
            body,
            body_stack,
            editor,
            edit,
            read_only,
            toolbar,
            menu_bar,
            current,
            editing,
            on_edit_finished,
            on_read_only_changed,
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
            &self.title_entry,
            &self.title_stack,
            &self.toolbar,
            &self.menu_bar,
            false,
        );
        self.edit.set_visible(false);
        self.read_only.set_visible(false);
        self.toolbar.widget.set_visible(false);
        self.title_stack.set_visible(true);
        self.title_stack.set_visible_child_name("entry");
        self.title_entry.set_editable(false);
        self.title.set_text("Select a note");
        self.metadata.set_text("Your note preview will appear here");
        self.metadata.set_visible(true);
        self.divider.set_visible(true);
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
            &self.title_entry,
            &self.title_stack,
            &self.toolbar,
            &self.menu_bar,
            false,
        );
        self.current.replace(Some(note.clone()));
        self.title_entry.set_text(note.display_title());
        self.title_stack.set_visible(true);
        self.title_stack.set_visible_child_name("entry");
        self.title_entry.set_editable(
            !note.editor_preferences.view_only && !matches!(note.state, NoteState::Trashed { .. }),
        );
        self.toolbar.widget.set_visible(false);
        self.edit.set_visible(true);
        self.read_only.set_visible(true);
        let read_only_enabled = note.editor_preferences.view_only;
        self.read_only.set_label(if read_only_enabled {
            "Exit read-only"
        } else {
            "Read-only"
        });
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
        self.metadata.set_visible(true);
        self.divider.set_visible(true);
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

    pub fn connect_read_only_changed<F: Fn(Note, bool) + 'static>(&self, callback: F) {
        self.on_read_only_changed.replace(Some(Rc::new(callback)));
    }

    pub fn set_sticky_read_only(&self) {
        self.finish_pending_edit();
        self.edit.set_visible(false);
        self.read_only.set_visible(false);
        self.title_stack.set_visible(false);
        self.metadata.set_visible(false);
        self.divider.set_visible(false);
        self.toolbar.widget.set_visible(false);
        set_editing(
            &self.editing,
            &self.body_stack,
            &self.editor,
            &self.edit,
            &self.title_entry,
            &self.title_stack,
            &self.toolbar,
            &self.menu_bar,
            false,
        );
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
    title_entry: &gtk::Entry,
    title_stack: &gtk::Stack,
    toolbar: &EditorToolbar,
    menu_bar: &EditorMenuBar,
    enabled: bool,
) {
    editing.set(enabled);
    editor.set_editable(enabled);
    toolbar.set_editable(enabled);
    editor.set_cursor_visible(enabled);
    if enabled {
        editor.set_monospace(false);
    }
    body_stack.set_visible_child_name(if enabled { "editor" } else { "preview" });
    edit.set_label(if enabled { "Done" } else { "Edit note" });
    let accessible_label = if enabled {
        "Finish editing note body"
    } else {
        "Edit note title and body"
    };
    edit.set_tooltip_text(Some(accessible_label));
    edit.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
    title_stack.set_visible_child_name("entry");
    title_entry.set_editable(enabled || title_entry.is_editable());
    toolbar.widget.set_visible(enabled);
    menu_bar.widget.set_visible(enabled);
    if enabled {
        title_entry.set_text(title_entry.text().as_str());
        toolbar.set_rich_formatting_enabled(true);
        editor.grab_focus();
    } else {
        toolbar.set_rich_formatting_enabled(false);
    }
}
