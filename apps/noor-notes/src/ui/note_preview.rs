use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use noor_domain::{EditorMode, Note, NoteId, NoteState};
use sourceview5::prelude::*;

use crate::appearance::{EffectiveTheme, try_global};
use crate::editor::{resolve_language, source_palette};
use crate::export::ExportFormat;
use crate::rich_buffer::RichBuffer;
use crate::save_as::save_note_as;
use crate::ui::editor_menu_bar::EditorMenuBar;
use crate::ui::editor_toolbar::EditorToolbar;

use super::adaptive_layout::{EditorLayoutDensity, editor_content_width, editor_layout_density};
use super::editor_canvas::configure_editor_canvas;

// NotePreview is the concrete implementation behind the shared
// `NoteEditorSurface` host boundary.

type BodyEditHandler = Rc<dyn Fn(Note)>;
type EditFinishedHandler = Rc<dyn Fn(NoteId)>;
type ReadOnlyHandler = Rc<dyn Fn(Note, bool)>;
type EditorModeRequestHandler = Rc<dyn Fn(Note, EditorMode)>;

#[derive(Clone)]
pub struct NotePreview {
    pub widget: gtk::ScrolledWindow,
    title: gtk::Label,
    title_entry: gtk::Entry,
    title_stack: gtk::Stack,
    heading: gtk::Box,
    metadata: gtk::Label,
    divider: gtk::Separator,
    body: gtk::Label,
    body_stack: gtk::Stack,
    editor: sourceview5::View,
    edit: gtk::Button,
    read_only: gtk::Button,
    toolbar: EditorToolbar,
    menu_bar: EditorMenuBar,
    margin_ruler: gtk::Box,
    left_margin: gtk::Scale,
    right_margin: gtk::Scale,
    document_clamp: adw::Clamp,
    available_width: Rc<Cell<i32>>,
    current: Rc<RefCell<Option<Note>>>,
    editing: Rc<Cell<bool>>,
    on_edit_finished: EditFinishedHandler,
    on_read_only_changed: Rc<RefCell<Option<ReadOnlyHandler>>>,
    on_editor_mode_requested: Rc<RefCell<Option<EditorModeRequestHandler>>>,
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
        let document_chrome = gtk::Box::new(gtk::Orientation::Vertical, 16);
        document_chrome.set_hexpand(true);

        let heading = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        heading.add_css_class("nn-preview-heading");
        let title = gtk::Label::new(Some("Select a note"));
        title.add_css_class("nn-display-title");
        title.add_css_class("nn-preview-title");
        title.add_css_class("nn-document-title");
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
        title_stack.set_visible_child_name("label");
        heading.append(&title_stack);
        let edit = gtk::Button::builder()
            .label("Edit")
            .icon_name("document-edit-symbolic")
            .tooltip_text("Edit note title and body")
            .visible(false)
            .build();
        edit.add_css_class("flat");
        edit.add_css_class("suggested-action");
        edit.add_css_class("nn-preview-edit");
        edit.update_property(&[gtk::accessible::Property::Label("Edit note body")]);
        edit.set_valign(gtk::Align::Start);
        let heading_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        heading_actions.add_css_class("nn-preview-heading-actions");
        heading_actions.set_halign(gtk::Align::Start);
        heading_actions.append(&edit);
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
        heading_actions.append(&read_only);
        heading.append(&heading_actions);
        document_chrome.append(&heading);

        let metadata = gtk::Label::new(Some("Your note preview will appear here"));
        metadata.add_css_class("nn-metadata");
        metadata.add_css_class("nn-preview-metadata");
        metadata.add_css_class("nn-text-meta");
        metadata.set_xalign(0.0);
        metadata.set_wrap(true);
        metadata.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        metadata.set_visible(false);
        document_chrome.append(&metadata);
        let divider = gtk::Separator::new(gtk::Orientation::Horizontal);
        divider.set_visible(false);
        document_chrome.append(&divider);

        let body = gtk::Label::new(Some(
            "Choose a note from the library to read it without opening another window.",
        ));
        body.add_css_class("nn-body");
        body.add_css_class("nn-preview-body");
        body.add_css_class("nn-text-body");
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

        let source_buffer = sourceview5::Buffer::builder()
            .enable_undo(true)
            .highlight_syntax(false)
            .highlight_matching_brackets(false)
            .build();
        let editor = sourceview5::View::with_buffer(&source_buffer);
        editor.add_css_class("nn-preview-editor");
        editor.add_css_class("nn-text-body");
        editor.add_css_class("nn-radius-8");
        editor.add_css_class("nn-focus-ring");
        editor.set_editable(false);
        editor.set_cursor_visible(false);
        editor.set_wrap_mode(gtk::WrapMode::WordChar);
        editor.set_hexpand(true);
        editor.set_vexpand(true);
        configure_editor_canvas(editor.upcast_ref(), true);
        let buffer: gtk::TextBuffer = source_buffer.clone().upcast();
        RichBuffer::prepare(&buffer);
        if let Some(appearance) = try_global() {
            let buffer_weak = buffer.downgrade();
            let source_buffer_weak = source_buffer.downgrade();
            appearance.subscribe(move |_, theme| {
                if let Some(buffer) = buffer_weak.upgrade() {
                    RichBuffer::apply_color_theme(&buffer, theme);
                }
                if let Some(buffer) = source_buffer_weak.upgrade() {
                    source_palette::apply(&buffer, theme);
                }
            });
        }

        let preview_body_clamp = adw::Clamp::builder()
            .maximum_size(860)
            .tightening_threshold(720)
            .child(&body)
            .build();
        preview_body_clamp.set_hexpand(true);
        preview_body_clamp.set_vexpand(true);
        let preview_scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&preview_body_clamp)
            .build();
        preview_scroll.add_css_class("nn-preview-body-scroll");
        let editor_scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&editor)
            .build();
        editor_scroll.add_css_class("nn-preview-body-scroll");
        let body_stack = gtk::Stack::new();
        body_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        body_stack.set_transition_duration(140);
        body_stack.set_hexpand(true);
        body_stack.set_vexpand(true);
        body_stack.add_named(&preview_scroll, Some("preview"));
        body_stack.add_named(&editor_scroll, Some("editor"));
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
        toolbar.widget.set_hexpand(true);
        toolbar.widget.set_halign(gtk::Align::Fill);
        crate::editor_actions::connect(&toolbar, &buffer, editor.upcast_ref());
        toolbar.set_rich_formatting_enabled(false);
        let menu_bar = EditorMenuBar::new_preview(&toolbar);
        menu_bar.widget.set_visible(false);
        menu_bar.widget.set_hexpand(true);
        menu_bar.widget.set_halign(gtk::Align::Fill);
        let margin_ruler = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        margin_ruler.add_css_class("nn-editor-margin-ruler");
        margin_ruler.set_hexpand(true);
        margin_ruler.set_visible(false);
        let ruler_label = gtk::Label::new(Some("Margins"));
        ruler_label.add_css_class("nn-editor-ruler-label");
        let left_margin = margin_scale("Left margin", "nn-editor-left-margin", false);
        let right_margin = margin_scale("Right margin", "nn-editor-right-margin", true);
        let reset_margins = gtk::Button::builder()
            .icon_name("edit-clear-symbolic")
            .tooltip_text("Reset margins")
            .build();
        reset_margins.add_css_class("flat");
        reset_margins.update_property(&[gtk::accessible::Property::Label("Reset margins")]);
        margin_ruler.append(&ruler_label);
        margin_ruler.append(&left_margin);
        margin_ruler.append(&reset_margins);
        margin_ruler.append(&right_margin);
        {
            let editor = editor.clone();
            left_margin.connect_value_changed(move |scale| {
                let padding = editor_horizontal_padding(&editor);
                editor.set_left_margin(padding + scale.value().round() as i32);
            });
        }
        {
            let editor = editor.clone();
            right_margin.connect_value_changed(move |scale| {
                let padding = editor_horizontal_padding(&editor);
                editor.set_right_margin(padding + scale.value().round() as i32);
            });
        }
        {
            let left_margin = left_margin.clone();
            let right_margin = right_margin.clone();
            reset_margins.connect_clicked(move |_| {
                left_margin.set_value(0.0);
                right_margin.set_value(0.0);
            });
        }
        document_chrome.append(&menu_bar.widget);
        document_chrome.append(&toolbar.widget);
        document_chrome.append(&margin_ruler);

        document.append(&document_chrome);
        document.append(&body_stack);
        let widget = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .child(&document)
            .build();
        widget.add_css_class("nn-preview-surface");

        let current = Rc::new(RefCell::new(None::<Note>));
        let editing = Rc::new(Cell::new(false));
        let on_read_only_changed = Rc::new(RefCell::new(None::<ReadOnlyHandler>));
        let on_editor_mode_requested = Rc::new(RefCell::new(None::<EditorModeRequestHandler>));
        {
            let current = current.clone();
            let editing = editing.clone();
            let on_body_edited = on_body_edited.clone();
            let title = title.clone();
            title_entry.connect_changed(move |entry| {
                if !editing.get() {
                    return;
                }
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
            let margin_ruler = margin_ruler.clone();
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
                    current.as_mut().and_then(|note| {
                        if !note.editor_preferences.view_only {
                            return None;
                        }
                        note.editor_preferences.view_only = false;
                        Some(note.clone())
                    })
                } else {
                    None
                };
                if let Some(note) = changed_note {
                    on_body_edited(note);
                }
                let mode = current
                    .borrow()
                    .as_ref()
                    .map(|note| note.editor_mode.clone())
                    .unwrap_or(EditorMode::Rich);
                set_editing(
                    &editing,
                    &body_stack,
                    &editor,
                    button,
                    &title_entry,
                    &title_stack,
                    &toolbar,
                    &menu_bar,
                    &margin_ruler,
                    &mode,
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
                set_read_only_button(&read_only, enabled);
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
        for (button, target) in [
            (&toolbar.mode_rich, EditorMode::Rich),
            (&toolbar.mode_markdown, EditorMode::Markdown),
            (&toolbar.mode_plain, EditorMode::PlainText),
            (&toolbar.mode_code, EditorMode::Code),
        ] {
            let current = current.clone();
            let editing = editing.clone();
            let on_editor_mode_requested = on_editor_mode_requested.clone();
            button.connect_clicked(move |_| {
                if !editing.get() {
                    return;
                }
                let Some(note) = current.borrow().clone() else {
                    return;
                };
                if note.editor_mode == target {
                    return;
                }
                if let Some(handler) = on_editor_mode_requested.borrow().as_ref() {
                    handler(note, target.clone());
                }
            });
        }
        let export_buttons = vec![
            toolbar.export_docx.clone(),
            toolbar.export_pdf.clone(),
            toolbar.export_html.clone(),
            toolbar.export_text.clone(),
            toolbar.export_markdown.clone(),
        ];
        let export_busy = Rc::new(Cell::new(false));
        for (button, format) in [
            (&toolbar.export_docx, ExportFormat::Docx),
            (&toolbar.export_pdf, ExportFormat::Pdf),
            (&toolbar.export_html, ExportFormat::Html),
            (&toolbar.export_text, ExportFormat::PlainText),
            (&toolbar.export_markdown, ExportFormat::Markdown),
        ] {
            connect_preview_export(
                button,
                format,
                &widget,
                current.clone(),
                export_buttons.clone(),
                export_busy.clone(),
            );
        }

        Self {
            widget,
            title,
            title_entry,
            title_stack,
            heading,
            metadata,
            divider,
            body,
            body_stack,
            editor,
            edit,
            read_only,
            toolbar,
            menu_bar,
            margin_ruler,
            left_margin,
            right_margin,
            document_clamp: preview_body_clamp,
            available_width: Rc::new(Cell::new(0)),
            current,
            editing,
            on_edit_finished,
            on_read_only_changed,
            on_editor_mode_requested,
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
            &self.margin_ruler,
            &EditorMode::Rich,
            false,
        );
        self.edit.set_visible(false);
        self.read_only.set_visible(false);
        self.toolbar.widget.set_visible(false);
        self.title_stack.set_visible(true);
        self.title_stack.set_visible_child_name("label");
        self.title_entry.set_editable(false);
        self.title.set_text("Select a note");
        self.metadata.set_text("Your note preview will appear here");
        self.metadata.set_visible(false);
        self.divider.set_visible(false);
        self.body
            .set_text("Choose a note from the library to read it without opening another window.");
        self.editor.buffer().set_text("");
    }

    pub fn show_note(&self, note: &Note) {
        self.finish_pending_edit();
        let changed_note =
            self.current.borrow().as_ref().map(|current| current.id) != Some(note.id);
        if changed_note {
            self.left_margin.set_value(0.0);
            self.right_margin.set_value(0.0);
        }
        set_editing(
            &self.editing,
            &self.body_stack,
            &self.editor,
            &self.edit,
            &self.title_entry,
            &self.title_stack,
            &self.toolbar,
            &self.menu_bar,
            &self.margin_ruler,
            &note.editor_mode,
            false,
        );
        self.current.replace(Some(note.clone()));
        self.title_entry.set_text(note.display_title());
        self.title_stack.set_visible(true);
        self.title_stack.set_visible_child_name("label");
        self.title_entry.set_editable(false);
        self.toolbar.widget.set_visible(false);
        self.edit.set_visible(true);
        self.read_only.set_visible(true);
        let read_only_enabled = note.editor_preferences.view_only;
        set_read_only_button(&self.read_only, read_only_enabled);
        self.edit
            .set_sensitive(!matches!(note.state, NoteState::Trashed { .. }));
        self.title.set_text(note.display_title());
        self.metadata.set_text(
            &note
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join("  "),
        );
        self.metadata.set_visible(!note.tags.is_empty());
        self.divider.set_visible(false);
        self.body.set_text(if note.content.trim().is_empty() {
            "This note is empty."
        } else {
            &note.content
        });

        configure_note_mode(&self.editor, &self.toolbar, note);
        self.apply_editor_margins();
        let source_buffer = preview_source_buffer(&self.editor);
        let buffer: gtk::TextBuffer = source_buffer.clone().upcast();
        RichBuffer::load(
            &buffer,
            &note.content,
            (note.editor_mode == EditorMode::Rich)
                .then_some(note.rich_content.as_ref())
                .flatten(),
        );
        RichBuffer::apply_color_theme(
            &buffer,
            try_global()
                .map(|appearance| appearance.effective_theme())
                .unwrap_or(EffectiveTheme::Snow),
        );
    }

    pub fn set_compact(&self, compact: bool) {
        self.toolbar.set_compact(compact);
        self.menu_bar.set_compact(compact);
        if compact {
            self.widget.add_css_class("compact");
        } else {
            self.widget.remove_css_class("compact");
        }
    }

    pub fn set_available_width(&self, available_width: i32) {
        self.available_width.set(available_width);
        let content_width = editor_content_width(available_width).max(1);
        self.document_clamp.set_maximum_size(content_width);
        self.document_clamp
            .set_tightening_threshold((content_width * 85 + 50) / 100);
        let maximum_margin = ((available_width - 240).max(0) / 2).min(320) as f64;
        self.left_margin.set_range(0.0, maximum_margin);
        self.right_margin.set_range(0.0, maximum_margin);

        let density = editor_layout_density(available_width);
        self.set_compact(density != EditorLayoutDensity::Spacious);
        let narrow = density == EditorLayoutDensity::Narrow;
        self.heading.set_orientation(if narrow {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        });
        if narrow {
            self.widget.add_css_class("narrow");
        } else {
            self.widget.remove_css_class("narrow");
        }
        self.apply_editor_margins();
    }

    fn apply_editor_margins(&self) {
        let padding = editor_horizontal_padding(&self.editor);
        self.editor
            .set_left_margin(padding + self.left_margin.value().round() as i32);
        self.editor
            .set_right_margin(padding + self.right_margin.value().round() as i32);
    }

    pub fn content_maximum_width(&self) -> i32 {
        self.document_clamp.maximum_size()
    }

    pub fn is_narrow(&self) -> bool {
        self.widget.has_css_class("narrow")
    }

    pub fn is_compact(&self) -> bool {
        self.widget.has_css_class("compact")
    }

    pub fn connect_read_only_changed<F: Fn(Note, bool) + 'static>(&self, callback: F) {
        self.on_read_only_changed.replace(Some(Rc::new(callback)));
    }

    pub fn connect_editor_mode_requested<F: Fn(Note, EditorMode) + 'static>(&self, callback: F) {
        self.on_editor_mode_requested
            .replace(Some(Rc::new(callback)));
    }

    pub fn set_mode_controls_sensitive(&self, sensitive: bool) {
        for button in [
            &self.toolbar.mode_rich,
            &self.toolbar.mode_markdown,
            &self.toolbar.mode_plain,
            &self.toolbar.mode_code,
        ] {
            button.set_sensitive(sensitive);
        }
    }

    pub fn set_read_only_open(&self, enabled: bool) {
        if let Some(note) = self.current.borrow_mut().as_mut() {
            note.editor_preferences.view_only = enabled;
        }
        set_read_only_button(&self.read_only, enabled);
    }

    pub fn is_read_only_open(&self) -> bool {
        self.current
            .borrow()
            .as_ref()
            .is_some_and(|note| note.editor_preferences.view_only)
    }

    pub fn read_only_label(&self) -> gtk::glib::GString {
        self.read_only.label().unwrap_or_default()
    }

    pub fn current_note_id(&self) -> Option<NoteId> {
        self.current.borrow().as_ref().map(|note| note.id)
    }

    pub fn title_stack_child_name(&self) -> Option<gtk::glib::GString> {
        self.title_stack.visible_child_name()
    }

    pub fn toolbar_visible(&self) -> bool {
        self.toolbar.widget.is_visible() && self.menu_bar.widget.is_visible()
    }

    pub fn editor(&self) -> gtk::TextView {
        self.editor.clone().upcast()
    }

    pub fn source_view(&self) -> sourceview5::View {
        self.editor.clone()
    }

    pub fn source_buffer(&self) -> sourceview5::Buffer {
        preview_source_buffer(&self.editor)
    }

    pub fn toolbar(&self) -> EditorToolbar {
        self.toolbar.clone()
    }

    pub fn active_mode(&self) -> EditorMode {
        self.current
            .borrow()
            .as_ref()
            .map(|note| note.editor_mode.clone())
            .unwrap_or(EditorMode::Rich)
    }

    pub fn begin_editing(&self) {
        if !self.editing.get() {
            self.edit.emit_clicked();
        }
    }

    pub fn finish_editing(&self) {
        if self.editing.get() {
            self.edit.emit_clicked();
        }
    }

    pub fn set_sticky_read_only(&self) {
        self.finish_pending_edit();
        self.edit.set_visible(false);
        self.read_only.set_visible(false);
        self.title.remove_css_class("nn-display-title");
        self.title.remove_css_class("nn-preview-title");
        self.title_stack.set_visible(false);
        self.metadata.set_visible(false);
        self.divider.set_visible(false);
        self.toolbar.widget.set_visible(false);
        let mode = self.active_mode();
        set_editing(
            &self.editing,
            &self.body_stack,
            &self.editor,
            &self.edit,
            &self.title_entry,
            &self.title_stack,
            &self.toolbar,
            &self.menu_bar,
            &self.margin_ruler,
            &mode,
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

fn connect_preview_export(
    button: &gtk::Button,
    format: ExportFormat,
    preview: &gtk::ScrolledWindow,
    current: Rc<RefCell<Option<Note>>>,
    export_buttons: Vec<gtk::Button>,
    busy: Rc<Cell<bool>>,
) {
    let preview = preview.clone();
    button.connect_clicked(move |_| {
        if busy.replace(true) {
            return;
        }
        let Some(note) = current.borrow().clone() else {
            busy.set(false);
            return;
        };
        let Some(parent) = preview.root().and_downcast::<gtk::Window>() else {
            busy.set(false);
            return;
        };
        for button in &export_buttons {
            button.set_sensitive(false);
        }

        let export_buttons = export_buttons.clone();
        let busy = busy.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            if let Err(error) = save_note_as(&parent, note, format).await {
                super::dialog_primitives::show_error(
                    &parent,
                    "Could not save exported copy",
                    &format!("Noor Notes did not change your note. {error}"),
                );
            }
            for button in &export_buttons {
                button.set_sensitive(true);
            }
            busy.set(false);
        });
    });
}

impl Default for NotePreview {
    fn default() -> Self {
        Self::new()
    }
}

fn set_read_only_button(button: &gtk::Button, enabled: bool) {
    button.set_label(if enabled {
        "Exit read-only"
    } else {
        "Read-only"
    });
    button.update_property(&[gtk::accessible::Property::Label(if enabled {
        "Close read-only sticky window"
    } else {
        "Open read-only sticky window"
    })]);
}

#[allow(
    clippy::too_many_arguments,
    reason = "preview edit mode updates one synchronized set of concrete GTK controls"
)]
fn set_editing(
    editing: &Cell<bool>,
    body_stack: &gtk::Stack,
    editor: &sourceview5::View,
    edit: &gtk::Button,
    title_entry: &gtk::Entry,
    title_stack: &gtk::Stack,
    toolbar: &EditorToolbar,
    menu_bar: &EditorMenuBar,
    margin_ruler: &gtk::Box,
    mode: &EditorMode,
    enabled: bool,
) {
    editing.set(enabled);
    editor.set_editable(enabled);
    toolbar.set_editor_mode(mode.clone());
    menu_bar.set_editor_mode(mode.clone());
    toolbar.set_editable(enabled);
    editor.set_cursor_visible(enabled);
    body_stack.set_visible_child_name(if enabled { "editor" } else { "preview" });
    edit.set_label(if enabled { "Done" } else { "Edit" });
    let accessible_label = if enabled {
        "Finish editing note body"
    } else {
        "Edit note title and body"
    };
    edit.set_tooltip_text(Some(accessible_label));
    edit.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
    title_stack.set_visible_child_name(if enabled { "entry" } else { "label" });
    title_entry.set_editable(enabled);
    toolbar.widget.set_visible(enabled);
    menu_bar.widget.set_visible(enabled);
    margin_ruler.set_visible(enabled);
    if enabled {
        editor.grab_focus();
    }
}

fn margin_scale(label: &str, css_class: &str, inverted: bool) -> gtk::Scale {
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 320.0, 8.0);
    scale.add_css_class("nn-editor-ruler-scale");
    scale.add_css_class(css_class);
    scale.set_hexpand(true);
    scale.set_draw_value(true);
    scale.set_digits(0);
    scale.set_value_pos(gtk::PositionType::Top);
    scale.set_inverted(inverted);
    scale.set_tooltip_text(Some(label));
    scale.update_property(&[gtk::accessible::Property::Label(label)]);
    for value in [0.0, 80.0, 160.0, 240.0, 320.0] {
        scale.add_mark(value, gtk::PositionType::Bottom, None);
    }
    scale
}

fn editor_horizontal_padding(editor: &sourceview5::View) -> i32 {
    if editor.has_css_class("nn-source-canvas") {
        16
    } else {
        8
    }
}

fn configure_note_mode(editor: &sourceview5::View, toolbar: &EditorToolbar, note: &Note) {
    let rich = note.editor_mode == EditorMode::Rich;
    let source_buffer = preview_source_buffer(editor);
    let manager = sourceview5::LanguageManager::default();
    let language = match &note.editor_mode {
        EditorMode::Markdown | EditorMode::Code => {
            resolve_language(&manager, &note.source_language)
        }
        EditorMode::Rich | EditorMode::PlainText => None,
    };
    source_buffer.set_language(language.as_ref());
    source_buffer.set_highlight_syntax(matches!(
        &note.editor_mode,
        EditorMode::Markdown | EditorMode::Code
    ));
    source_buffer.set_highlight_matching_brackets(!rich);
    source_palette::apply(
        &source_buffer,
        try_global()
            .map(|appearance| appearance.effective_theme())
            .unwrap_or(EffectiveTheme::Snow),
    );
    editor.set_show_line_numbers(!rich);
    editor.set_highlight_current_line(!rich);
    editor.set_auto_indent(!rich);
    editor.set_monospace(note.editor_mode == EditorMode::Code);
    editor.set_wrap_mode(if note.editor_preferences.word_wrap {
        gtk::WrapMode::WordChar
    } else {
        gtk::WrapMode::None
    });
    configure_editor_canvas(editor.upcast_ref(), rich);
    toolbar.set_editor_mode(note.editor_mode.clone());
}

fn preview_source_buffer(editor: &sourceview5::View) -> sourceview5::Buffer {
    editor
        .buffer()
        .downcast::<sourceview5::Buffer>()
        .expect("NotePreview always owns a GtkSourceView buffer")
}
