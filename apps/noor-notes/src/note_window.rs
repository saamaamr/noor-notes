use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{EditorMode, Note, NoteColor, NoteState};
use noor_storage::SqliteNoteRepository;
use noor_windowing::{GnomeWindowController, NativeWindowId, WindowController};

use crate::appearance::global;
use crate::autosave::{AutosaveQueue, NoteDraft};
use crate::edit_save_gate::EditSaveGate;
use crate::editor::{SourceEditorAdapter, apply_conversion, preview_conversion, source_palette};
use crate::editor_status::{EditorStatistics, clamp_zoom, line_offset};
use crate::export::{export_markdown, export_plain};
use crate::note_actions;
use crate::note_find::{FindOptions, FindResults};
use crate::rich_buffer::RichBuffer;
use crate::safe_export::{ExportExtension, sanitize_export_name, set_owner_only};
use crate::save_status::SaveStatusIndicator;
use crate::services::trash_command;
use crate::ui::appearance_button::AppearanceButton;
use crate::ui::editor_presentation::EditorPresentation;
use crate::ui::editor_toolbar::EditorToolbar;

pub struct NoteWindow {
    pub window: adw::ApplicationWindow,
}

impl NoteWindow {
    pub fn new(
        app: &adw::Application,
        note: Note,
        autosave: AutosaveQueue,
        repository: SqliteNoteRepository,
        controller: Arc<dyn WindowController>,
    ) -> Self {
        let note = Rc::new(RefCell::new(note));
        let current = note.borrow().clone();
        let window_title = GnomeWindowController::window_title(&current.id.value().to_string());
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(&window_title)
            .default_width(current.geometry.width)
            .default_height(current.geometry.height)
            .build();
        window.add_css_class("nn-editor-window");
        window.add_css_class(current.color.css_class());
        window.set_opacity(current.style.opacity);
        let appearance = global();
        appearance.register_window(&window);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let header = adw::HeaderBar::new();
        let toolbar = EditorToolbar::new();
        let is_active = matches!(current.state, NoteState::Active);
        let is_trashed = matches!(current.state, NoteState::Trashed { .. });
        toolbar.archive.set_visible(is_active);
        toolbar.header_archive.set_visible(is_active);
        toolbar.trash.set_visible(!is_trashed);
        toolbar.header_trash.set_visible(!is_trashed);
        toolbar.view_only.set_visible(!is_trashed);
        toolbar.restore.set_visible(is_trashed);
        toolbar.permanent_delete.set_visible(is_trashed);
        let title_entry = gtk::Entry::builder()
            .text(current.display_title())
            .placeholder_text("Untitled note")
            .editable(!is_trashed)
            .build();
        title_entry.add_css_class("nn-editor-title");
        title_entry.set_hexpand(true);
        title_entry.set_width_chars(32);
        let save_status = SaveStatusIndicator::new();
        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        title_box.add_css_class("editor-title-box");
        title_box.append(&title_entry);
        title_box.append(&save_status.widget);
        header.set_title_widget(Some(&title_box));
        let library_pin = gtk::ToggleButton::builder()
            .icon_name("view-pin-symbolic")
            .tooltip_text("Pin note in the library")
            .active(current.pinned)
            .build();
        let favorite = gtk::ToggleButton::builder()
            .icon_name(if current.favorite {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            })
            .tooltip_text("Add to favorites")
            .active(current.favorite)
            .build();
        header.pack_end(&toolbar.header_trash);
        header.pack_end(&toolbar.header_archive);
        header.pack_end(&toolbar.appearance);
        let appearance_button = AppearanceButton::new(appearance.clone());
        header.pack_end(&appearance_button.button);
        header.pack_end(&favorite);
        header.pack_end(&library_pin);
        layout.append(&header);
        layout.append(&toolbar.widget);

        let metadata = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        metadata.add_css_class("nn-tag-strip");
        metadata.set_margin_start(24);
        metadata.set_margin_end(24);
        metadata.set_margin_top(8);
        metadata.set_margin_bottom(8);
        metadata.append(&gtk::Image::from_icon_name("tag-symbolic"));
        let tags_entry = gtk::Entry::builder()
            .text(current.tags.join(", "))
            .placeholder_text("Add tags, separated by commas")
            .editable(!is_trashed)
            .build();
        tags_entry.add_css_class("nn-tag-entry");
        tags_entry.set_hexpand(true);
        metadata.append(&tags_entry);
        layout.append(&metadata);

        let (buffer, editor, source_buffer) = if current.editor_mode == EditorMode::Rich {
            let buffer = gtk::TextBuffer::new(None);
            RichBuffer::load(&buffer, &current.content, current.rich_content.as_ref());
            let editor = gtk::TextView::with_buffer(&buffer);
            (buffer, editor, None)
        } else {
            let language = match current.editor_mode {
                EditorMode::PlainText => None,
                EditorMode::Markdown | EditorMode::Code => Some(&current.source_language),
                EditorMode::Rich => unreachable!(),
            };
            let adapter = SourceEditorAdapter::new_with_theme(
                &current.content,
                language,
                appearance.effective_theme(),
            );
            (
                adapter.buffer().clone().upcast::<gtk::TextBuffer>(),
                adapter.view().clone().upcast::<gtk::TextView>(),
                Some(adapter.buffer().clone()),
            )
        };
        if let Some(source_buffer) = source_buffer {
            let source_buffer = source_buffer.downgrade();
            appearance.subscribe(move |_, theme| {
                if let Some(buffer) = source_buffer.upgrade() {
                    source_palette::apply(&buffer, theme);
                }
            });
        } else {
            let rich_buffer = buffer.downgrade();
            appearance.subscribe(move |_, theme| {
                if let Some(buffer) = rich_buffer.upgrade() {
                    RichBuffer::apply_color_theme(&buffer, theme);
                }
            });
        }
        let rich_mode = current.editor_mode == EditorMode::Rich;
        toolbar.set_rich_formatting_enabled(rich_mode);
        toolbar
            .word_wrap
            .set_active(current.editor_preferences.word_wrap);
        let initial_wrap = if current.editor_preferences.word_wrap {
            gtk::WrapMode::WordChar
        } else {
            gtk::WrapMode::None
        };
        editor.set_buffer(Some(&buffer));
        editor.set_wrap_mode(initial_wrap);
        editor.set_left_margin(48);
        editor.set_right_margin(48);
        editor.set_top_margin(32);
        editor.set_bottom_margin(48);
        editor.set_accepts_tab(true);
        editor.add_css_class("nn-writing-canvas");
        if rich_mode {
            editor.add_css_class("nn-rich-writing-canvas");
        }
        editor.set_editable(!is_trashed);
        let find_entry = gtk::SearchEntry::builder()
            .placeholder_text("Find in note…")
            .hexpand(true)
            .build();
        let replace_entry = gtk::Entry::builder()
            .placeholder_text("Replace with…")
            .hexpand(true)
            .build();
        let match_case = gtk::CheckButton::with_label("Match case");
        let whole_word = gtk::CheckButton::with_label("Whole word");
        let replace = gtk::Button::with_label("Replace");
        let replace_all = gtk::Button::with_label("Replace All");
        let find_close = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text("Close find and replace (Escape)")
            .build();
        let find_previous = gtk::Button::builder()
            .icon_name("go-up-symbolic")
            .tooltip_text("Previous match")
            .build();
        let find_next = gtk::Button::builder()
            .icon_name("go-down-symbolic")
            .tooltip_text("Next match")
            .build();
        let find_count = gtk::Label::new(Some("0 of 0"));
        let find_bar = gtk::Box::new(gtk::Orientation::Vertical, 4);
        find_bar.add_css_class("nn-find-panel");
        let find_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        find_row.append(&find_entry);
        find_row.append(&find_count);
        find_row.append(&find_previous);
        find_row.append(&find_next);
        find_row.append(&find_close);
        let replace_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        replace_row.append(&replace_entry);
        replace_row.append(&replace);
        replace_row.append(&replace_all);
        replace_row.append(&match_case);
        replace_row.append(&whole_word);
        find_bar.append(&find_row);
        find_bar.append(&replace_row);
        find_bar.set_visible(false);
        layout.append(&find_bar);
        let find_results = Rc::new(RefCell::new(FindResults::default()));
        let find_options = Rc::new(Cell::new(FindOptions::default()));
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            let options = find_options.clone();
            find_entry.connect_search_changed(move |entry| {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
                results
                    .borrow_mut()
                    .update_with_options(&text, &entry.text(), options.get());
                select_find_result(&buffer, &results.borrow(), &count);
            });
        }
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            find_next.connect_clicked(move |_| {
                results.borrow_mut().next();
                select_find_result(&buffer, &results.borrow(), &count);
            });
        }
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            find_entry.connect_activate(move |_| {
                results.borrow_mut().next();
                select_find_result(&buffer, &results.borrow(), &count);
            });
        }
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            let keys = gtk::EventControllerKey::new();
            keys.connect_key_pressed(move |_, key, _, state| {
                if key == gtk::gdk::Key::Return
                    && state.contains(gtk::gdk::ModifierType::SHIFT_MASK)
                {
                    results.borrow_mut().previous();
                    select_find_result(&buffer, &results.borrow(), &count);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            });
            find_entry.add_controller(keys);
        }
        for option in [&match_case, &whole_word] {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            let query = find_entry.clone();
            let match_case = match_case.clone();
            let whole_word = whole_word.clone();
            let options = find_options.clone();
            option.connect_toggled(move |_| {
                options.set(FindOptions {
                    match_case: match_case.is_active(),
                    whole_word: whole_word.is_active(),
                });
                update_find(&buffer, &query, &results, &count, options.get());
            });
        }
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            let query = find_entry.clone();
            let replacement = replace_entry.clone();
            let options = find_options.clone();
            replace.connect_clicked(move |_| {
                let Some((start, end)) = results.borrow().current_range() else {
                    return;
                };
                let mut start = buffer.iter_at_offset(start as i32);
                let mut end = buffer.iter_at_offset(end as i32);
                buffer.begin_user_action();
                buffer.delete(&mut start, &mut end);
                buffer.insert(&mut start, &replacement.text());
                buffer.end_user_action();
                update_find(&buffer, &query, &results, &count, options.get());
            });
        }
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            let query = find_entry.clone();
            let replacement = replace_entry.clone();
            let options = find_options.clone();
            replace_all.connect_clicked(move |_| {
                let ranges = results.borrow().ranges().to_vec();
                if ranges.is_empty() {
                    return;
                }
                buffer.begin_user_action();
                for (start, end) in ranges.into_iter().rev() {
                    let mut start = buffer.iter_at_offset(start as i32);
                    let mut end = buffer.iter_at_offset(end as i32);
                    buffer.delete(&mut start, &mut end);
                    buffer.insert(&mut start, &replacement.text());
                }
                buffer.end_user_action();
                update_find(&buffer, &query, &results, &count, options.get());
            });
        }
        {
            let find_bar = find_bar.clone();
            let find_toggle = toolbar.find.clone();
            find_close.connect_clicked(move |_| {
                find_bar.set_visible(false);
                find_toggle.set_active(false);
            });
        }
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            find_previous.connect_clicked(move |_| {
                results.borrow_mut().previous();
                select_find_result(&buffer, &results.borrow(), &count);
            });
        }
        {
            let find_bar = find_bar.clone();
            let find_entry = find_entry.clone();
            toolbar.find.connect_toggled(move |button| {
                find_bar.set_visible(button.is_active());
                if button.is_active() {
                    find_entry.grab_focus();
                }
            });
        }

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&editor)
            .build();
        scroller.add_css_class("nn-canvas-scroller");
        layout.append(&scroller);
        let editor_status =
            gtk::Label::new(Some("Ln 1, Col 1  ·  0 words  ·  0 characters  ·  100%"));
        editor_status.set_halign(gtk::Align::Start);
        let mode_name = match current.editor_mode {
            EditorMode::Rich => "Rich Text",
            EditorMode::Markdown => "Markdown",
            EditorMode::PlainText => "Plain Text",
            EditorMode::Code => current.source_language.as_str(),
        };
        let mode_status = gtk::Label::new(Some(&format!("{mode_name}  ·  UTF-8")));
        mode_status.set_halign(gtk::Align::End);
        mode_status.set_hexpand(true);
        let status_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        status_bar.add_css_class("nn-statusbar");
        status_bar.append(&editor_status);
        status_bar.append(&mode_status);
        layout.append(&status_bar);
        window.set_content(Some(&layout));
        let presentation = EditorPresentation::new(
            &editor,
            is_trashed,
            vec![
                title_box.clone().upcast::<gtk::Widget>(),
                toolbar.widget.clone().upcast::<gtk::Widget>(),
                metadata.clone().upcast::<gtk::Widget>(),
                find_bar.clone().upcast::<gtk::Widget>(),
                status_bar.clone().upcast::<gtk::Widget>(),
                toolbar.appearance.clone().upcast::<gtk::Widget>(),
                appearance_button.button.clone().upcast::<gtk::Widget>(),
                favorite.clone().upcast::<gtk::Widget>(),
                library_pin.clone().upcast::<gtk::Widget>(),
                toolbar.header_trash.clone().upcast::<gtk::Widget>(),
                toolbar.header_archive.clone().upcast::<gtk::Widget>(),
            ],
        );
        presentation.set_view_only(current.editor_preferences.view_only && !is_trashed);
        let view_mode_busy = Rc::new(Cell::new(false));
        let exit_view_mode: Rc<dyn Fn()> = {
            let note = note.clone();
            let autosave = autosave.clone();
            let repository = repository.clone();
            let presentation = presentation.clone();
            let window = window.clone();
            let busy = view_mode_busy.clone();
            Rc::new(move || {
                request_view_mode(
                    note.clone(),
                    autosave.clone(),
                    repository.clone(),
                    presentation.clone(),
                    window.clone(),
                    busy.clone(),
                    false,
                );
            })
        };
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let repository = repository.clone();
            let presentation = presentation.clone();
            let window = window.clone();
            let busy = view_mode_busy.clone();
            toolbar.view_only.connect_clicked(move |_| {
                request_view_mode(
                    note.clone(),
                    autosave.clone(),
                    repository.clone(),
                    presentation.clone(),
                    window.clone(),
                    busy.clone(),
                    true,
                );
            });
        }
        {
            let presentation = presentation.clone();
            let exit = exit_view_mode.clone();
            let gesture = gtk::GestureClick::new();
            gesture.set_button(0);
            gesture.connect_released(move |_, presses, _, _| {
                if presses == 2 && presentation.is_view_only() {
                    exit();
                }
            });
            editor.add_controller(gesture);
        }
        {
            let keys = gtk::EventControllerKey::new();
            let note = note.clone();
            let autosave = autosave.clone();
            let buffer = buffer.clone();
            let wrap = toolbar.word_wrap.clone();
            let app = app.clone();
            let presentation = presentation.clone();
            let exit_view_mode = exit_view_mode.clone();
            keys.connect_key_pressed(move |_, key, _, state| {
                if key == gtk::gdk::Key::Escape && presentation.is_view_only() {
                    exit_view_mode();
                    return gtk::glib::Propagation::Stop;
                }
                if state.contains(gtk::gdk::ModifierType::ALT_MASK) && key == gtk::gdk::Key::z {
                    wrap.set_active(!wrap.is_active());
                    return gtk::glib::Propagation::Stop;
                }
                if !state.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                    return gtk::glib::Propagation::Proceed;
                }
                if key == gtk::gdk::Key::s {
                    save_editor_snapshot(&buffer, &note, &autosave);
                    let autosave = autosave.clone();
                    let id = note.borrow().id;
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = autosave.flush(id).await;
                    });
                    gtk::glib::Propagation::Stop
                } else if key == gtk::gdk::Key::n {
                    app.activate_action("new-note", None);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            });
            window.add_controller(keys);
        }
        crate::editor_actions::connect(&toolbar, &buffer, &editor);
        let zoom = Rc::new(Cell::new(current.editor_preferences.zoom_percent));
        apply_editor_zoom(&editor, zoom.get());
        update_editor_status(&buffer, &editor_status, zoom.get());
        {
            let status = editor_status.clone();
            let zoom = zoom.clone();
            buffer.connect_changed(move |buffer| update_editor_status(buffer, &status, zoom.get()));
        }
        {
            let status = editor_status.clone();
            let zoom = zoom.clone();
            buffer.connect_mark_set(move |buffer, _, _| {
                update_editor_status(buffer, &status, zoom.get())
            });
        }
        {
            let editor = editor.clone();
            let note = note.clone();
            let autosave = autosave.clone();
            toolbar.word_wrap.connect_toggled(move |button| {
                let enabled = button.is_active();
                editor.set_wrap_mode(if button.is_active() {
                    gtk::WrapMode::WordChar
                } else {
                    gtk::WrapMode::None
                });
                note.borrow_mut().editor_preferences.word_wrap = enabled;
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }
        connect_zoom(
            &toolbar.zoom_in,
            &editor,
            &editor_status,
            &buffer,
            zoom.clone(),
            10,
        );
        connect_zoom(
            &toolbar.zoom_out,
            &editor,
            &editor_status,
            &buffer,
            zoom.clone(),
            -10,
        );
        {
            let note = note.clone();
            for button in [&toolbar.zoom_in, &toolbar.zoom_out] {
                let note = note.clone();
                let autosave = autosave.clone();
                let zoom = zoom.clone();
                button.connect_clicked(move |_| {
                    note.borrow_mut()
                        .editor_preferences
                        .set_zoom_percent(zoom.get());
                    autosave.schedule(NoteDraft::from(note.borrow().clone()));
                });
            }
            let autosave = autosave.clone();
            let editor = editor.clone();
            let status = editor_status.clone();
            let buffer = buffer.clone();
            let zoom = zoom.clone();
            toolbar.zoom_reset.connect_clicked(move |_| {
                zoom.set(100);
                note.borrow_mut().editor_preferences.set_zoom_percent(100);
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                apply_editor_zoom(&editor, 100);
                update_editor_status(&buffer, &status, 100);
            });
        }
        {
            let window = window.clone();
            toolbar.fullscreen.connect_toggled(move |button| {
                if button.is_active() {
                    window.fullscreen();
                } else {
                    window.unfullscreen();
                }
            });
        }
        {
            let window = window.clone();
            let buffer = buffer.clone();
            let editor = editor.clone();
            toolbar
                .go_to_line
                .connect_clicked(move |_| show_go_to_line(&window, &buffer, &editor));
        }
        {
            let mut state = autosave.subscribe(current.id);
            let indicator = save_status.clone();
            indicator.set_state(&state.borrow_and_update().clone());
            gtk::glib::MainContext::default().spawn_local(async move {
                while state.changed().await.is_ok() {
                    indicator.set_state(&state.borrow_and_update().clone());
                }
            });
        }
        {
            let autosave = autosave.clone();
            let id = current.id;
            save_status.retry.connect_clicked(move |_| {
                let autosave = autosave.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    let _ = autosave.retry(id).await;
                });
            });
        }
        {
            let app = app.clone();
            toolbar
                .new_note
                .connect_clicked(move |_| app.activate_action("new-note", None));
        }

        {
            let app = app.clone();
            let repository = repository.clone();
            let controller = controller.clone();
            let autosave = autosave.clone();
            let source = note.clone();
            toolbar.duplicate.connect_clicked(move |_| {
                let app = app.clone();
                let repository = repository.clone();
                let controller = controller.clone();
                let autosave = autosave.clone();
                let id = source.borrow().id;
                gtk::glib::MainContext::default().spawn_local(async move {
                    if let Ok(copy) = repository.duplicate_note(id, Utc::now()).await {
                        NoteWindow::new(&app, copy, autosave, repository, controller).present();
                    }
                });
            });
        }
        connect_export(&toolbar.export_text, &window, note.clone(), false);
        let mode_context = ModeSwitchContext {
            window: window.clone(),
            app: app.clone(),
            buffer: buffer.clone(),
            note: note.clone(),
            autosave: autosave.clone(),
            repository: repository.clone(),
            controller: controller.clone(),
        };
        for (button, target) in [
            (&toolbar.mode_rich, EditorMode::Rich),
            (&toolbar.mode_markdown, EditorMode::Markdown),
            (&toolbar.mode_plain, EditorMode::PlainText),
            (&toolbar.mode_code, EditorMode::Code),
        ] {
            connect_editor_mode(button, target, mode_context.clone());
        }
        connect_export(&toolbar.export_markdown, &window, note.clone(), true);

        for (button, color) in toolbar.note_color_buttons.iter().zip([
            NoteColor::Yellow,
            NoteColor::Cream,
            NoteColor::Blue,
            NoteColor::Green,
            NoteColor::Rose,
            NoteColor::Lavender,
        ]) {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            button.connect_clicked(move |_| {
                for candidate in [
                    NoteColor::Yellow,
                    NoteColor::Cream,
                    NoteColor::Blue,
                    NoteColor::Green,
                    NoteColor::Rose,
                    NoteColor::Lavender,
                ] {
                    window.remove_css_class(candidate.css_class());
                }
                window.add_css_class(color.css_class());
                note.borrow_mut().color = color;
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }

        toolbar.pin.set_active(current.always_on_top);
        toolbar.all_workspaces.set_active(current.all_workspaces);
        toolbar.opacity.set_value(current.style.opacity);
        let capabilities = controller.capabilities();
        toolbar.pin.set_sensitive(capabilities.always_on_top);
        toolbar
            .all_workspaces
            .set_sensitive(capabilities.all_workspaces);
        if !capabilities.always_on_top {
            toolbar
                .pin
                .set_tooltip_text(Some("Always on Top is unavailable on this Wayland desktop"));
        }

        {
            let note = note.clone();
            let autosave = autosave.clone();
            tags_entry.connect_changed(move |entry| {
                note.borrow_mut()
                    .set_tags(entry.text().split(',').map(str::to_string).collect());
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }
        {
            {
                let note = note.clone();
                let autosave = autosave.clone();
                library_pin.connect_toggled(move |button| {
                    note.borrow_mut().pinned = button.is_active();
                    autosave.schedule(NoteDraft::from(note.borrow().clone()));
                });
            }
            {
                let note = note.clone();
                let autosave = autosave.clone();
                favorite.connect_toggled(move |button| {
                    let enabled = button.is_active();
                    note.borrow_mut().favorite = enabled;
                    button.set_icon_name(if enabled {
                        "starred-symbolic"
                    } else {
                        "non-starred-symbolic"
                    });
                    autosave.schedule(NoteDraft::from(note.borrow().clone()));
                });
            }
            let note = note.clone();
            let autosave = autosave.clone();
            title_entry.connect_changed(move |entry| {
                note.borrow_mut().title = entry.text().trim().to_string();
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let title_entry = title_entry.clone();
            toolbar.rename.connect_clicked(move |_| {
                let note = note.clone();
                let autosave = autosave.clone();
                let window = window.clone();
                let title_entry = title_entry.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    let rename_entry = gtk::Entry::builder()
                        .text(note.borrow().display_title())
                        .activates_default(true)
                        .build();
                    let dialog = adw::AlertDialog::builder()
                        .heading("Rename note")
                        .body("Choose a clear name for this note.")
                        .extra_child(&rename_entry)
                        .build();
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("rename", "Rename");
                    dialog.set_default_response(Some("rename"));
                    dialog.set_close_response("cancel");
                    if dialog.choose_future(Some(&window)).await == "rename" {
                        let title = rename_entry.text().trim().to_string();
                        note.borrow_mut().title = title.clone();
                        title_entry.set_text(&title);
                        autosave.schedule(NoteDraft::from(note.borrow().clone()));
                    }
                });
            });
        }
        let edit_save_gate = Rc::new(RefCell::new(EditSaveGate::default()));
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let edit_save_gate = edit_save_gate.clone();
            buffer.connect_changed(move |buffer| {
                if !edit_save_gate.borrow_mut().mark_changed() {
                    return;
                }
                let buffer = buffer.clone();
                let note = note.clone();
                let autosave = autosave.clone();
                let edit_save_gate = edit_save_gate.clone();
                gtk::glib::timeout_add_local_once(Duration::from_millis(250), move || {
                    if edit_save_gate.borrow_mut().take_snapshot() {
                        save_editor_snapshot(&buffer, &note, &autosave);
                    }
                });
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let controller = controller.clone();
            toolbar.pin.connect_toggled(move |button| {
                let enabled = button.is_active();
                note.borrow_mut().always_on_top = enabled;
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                if let Some(id) = native_window_id(&window) {
                    let controller = controller.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = controller.set_above(id, enabled).await;
                    });
                }
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let controller = controller.clone();
            toolbar.all_workspaces.connect_toggled(move |button| {
                let enabled = button.is_active();
                note.borrow_mut().all_workspaces = enabled;
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                if let Some(id) = native_window_id(&window) {
                    let controller = controller.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = controller.set_all_workspaces(id, enabled).await;
                    });
                }
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let controller = controller.clone();
            toolbar.opacity.connect_value_changed(move |scale| {
                let value = scale.value();
                window.set_opacity(value);
                note.borrow_mut().style.set_opacity(value);
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                if let Some(id) = native_window_id(&window) {
                    let controller = controller.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = controller.set_opacity(id, value).await;
                    });
                }
            });
        }
        {
            let repository = repository.clone();
            let window = window.clone();
            let id = current.id;
            toolbar.restore.connect_clicked(move |button| {
                button.set_sensitive(false);
                let button = button.clone();
                let repository = repository.clone();
                let window = window.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    if repository.restore(id, Utc::now()).await.is_ok() {
                        if let Some(application) = window.application() {
                            application.activate_action("refresh-notes", None);
                        }
                        window.close();
                    } else {
                        button.set_sensitive(true);
                        show_save_error(&window);
                    }
                });
            });
        }
        {
            let repository = repository.clone();
            let window = window.clone();
            let id = current.id;
            toolbar.permanent_delete.connect_clicked(move |button| {
                button.set_sensitive(false);
                let button = button.clone();
                let repository = repository.clone();
                let window = window.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    let dialog = adw::AlertDialog::new(Some("Permanently delete this note?"), Some("This cannot be undone. The note and its local history will be removed."));
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("delete", "Permanently Delete");
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");
                    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                    if dialog.choose_future(Some(&window)).await != "delete" { button.set_sensitive(true); return; }
                    if repository.delete_permanently(id).await.is_ok() {
                        if let Some(application) = window.application() { application.activate_action("refresh-notes", None); }
                        window.close();
                    } else { button.set_sensitive(true); show_save_error(&window); }
                });
            });
        }
        for button in [&toolbar.header_archive, &toolbar.archive] {
            connect_archive_button(
                button,
                &buffer,
                note.clone(),
                autosave.clone(),
                window.clone(),
            );
        }
        for button in [&toolbar.header_trash, &toolbar.trash] {
            connect_trash_button(
                button,
                note.clone(),
                autosave.clone(),
                repository.clone(),
                window.clone(),
            );
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            window.connect_notify_local(Some("width"), move |window, _| {
                note.borrow_mut().geometry.width = window.width();
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            window.connect_notify_local(Some("height"), move |window, _| {
                note.borrow_mut().geometry.height = window.height();
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }

        {
            let autosave = autosave.clone();
            let id = current.id;
            let allow_close = Rc::new(Cell::new(false));
            let buffer = buffer.clone();
            let note = note.clone();
            let edit_save_gate = edit_save_gate.clone();
            window.connect_close_request(move |window| {
                if allow_close.get() {
                    return gtk::glib::Propagation::Proceed;
                }
                if edit_save_gate.borrow_mut().take_snapshot() {
                    save_editor_snapshot(&buffer, &note, &autosave);
                }
                if !autosave.has_pending(id) {
                    return gtk::glib::Propagation::Proceed;
                }
                let autosave = autosave.clone();
                let window = window.clone();
                let allow_close = allow_close.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    if autosave.flush(id).await.is_ok() {
                        allow_close.set(true);
                        window.close();
                    } else {
                        show_save_error(&window);
                    }
                });
                gtk::glib::Propagation::Stop
            });
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn save_editor_snapshot(
    buffer: &gtk::TextBuffer,
    note: &Rc<RefCell<Note>>,
    autosave: &AutosaveQueue,
) {
    let mut note = note.borrow_mut();
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
    autosave.schedule(NoteDraft::from(note.clone()));
}

fn show_save_error(window: &adw::ApplicationWindow) {
    let dialog = adw::AlertDialog::new(
        Some("Could not save note"),
        Some("Noor Notes kept this window open. Please try again."),
    );
    dialog.add_response("ok", "OK");
    dialog.present(Some(window));
}

fn native_window_id(window: &adw::ApplicationWindow) -> Option<NativeWindowId> {
    let surface = window.surface()?;
    if let Ok(surface) = surface.downcast::<gdk4_x11::X11Surface>() {
        return Some(NativeWindowId::X11(surface.xid() as u32));
    }
    window
        .title()
        .map(|title| NativeWindowId::Wayland(title.to_string()))
}

fn update_find(
    buffer: &gtk::TextBuffer,
    entry: &gtk::SearchEntry,
    results: &Rc<RefCell<FindResults>>,
    count: &gtk::Label,
    options: FindOptions,
) {
    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
    results
        .borrow_mut()
        .update_with_options(&text, &entry.text(), options);
    select_find_result(buffer, &results.borrow(), count);
}

fn select_find_result(buffer: &gtk::TextBuffer, results: &FindResults, count: &gtk::Label) {
    if let Some((current, total)) = results.position() {
        count.set_text(&format!("{current} of {total}"));
    } else {
        count.set_text("0 of 0");
    }

    if let Some((start, end)) = results.current_range() {
        buffer.select_range(
            &buffer.iter_at_offset(start as i32),
            &buffer.iter_at_offset(end as i32),
        );
    }
}

fn update_editor_status(buffer: &gtk::TextBuffer, label: &gtk::Label, zoom: u16) {
    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
    let cursor = buffer.iter_at_mark(&buffer.get_insert()).offset().max(0) as usize;
    let selection = buffer
        .selection_bounds()
        .map(|(start, end)| (start.offset().max(0) as usize, end.offset().max(0) as usize));
    let stats = EditorStatistics::calculate(&text, cursor, selection, zoom);
    let selection = if stats.selection > 0 {
        format!("  ·  {} selected", stats.selection)
    } else {
        String::new()
    };
    label.set_text(&format!(
        "Ln {}, Col {}  ·  {} lines  ·  {} words  ·  {} characters{}  ·  {}%",
        stats.line, stats.column, stats.lines, stats.words, stats.characters, selection, stats.zoom
    ));
}

fn connect_zoom(
    button: &gtk::Button,
    editor: &gtk::TextView,
    status: &gtk::Label,
    buffer: &gtk::TextBuffer,
    zoom: Rc<Cell<u16>>,
    delta: i16,
) {
    let editor = editor.clone();
    let status = status.clone();
    let buffer = buffer.clone();
    button.connect_clicked(move |_| {
        let next = clamp_zoom((zoom.get() as i16 + delta).max(0) as u16);
        zoom.set(next);
        apply_editor_zoom(&editor, next);
        update_editor_status(&buffer, &status, next);
    });
}

fn apply_editor_zoom(editor: &gtk::TextView, zoom: u16) {
    for value in (50..=300).step_by(10) {
        editor.remove_css_class(&format!("zoom-{value}"));
    }
    editor.add_css_class(&format!("zoom-{}", (zoom / 10) * 10));
}

fn show_go_to_line(
    window: &adw::ApplicationWindow,
    buffer: &gtk::TextBuffer,
    editor: &gtk::TextView,
) {
    let entry = gtk::Entry::builder()
        .placeholder_text("Line number")
        .input_purpose(gtk::InputPurpose::Digits)
        .activates_default(true)
        .build();
    let dialog = adw::AlertDialog::builder()
        .heading("Go to line")
        .body("Enter a line number in this note.")
        .extra_child(&entry)
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("go", "Go");
    dialog.set_default_response(Some("go"));
    dialog.set_close_response("cancel");
    let window = window.clone();
    let buffer = buffer.clone();
    let editor = editor.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        if dialog.choose_future(Some(&window)).await != "go" {
            return;
        }
        let requested = entry.text().parse::<usize>().unwrap_or(1);
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        let offset = line_offset(&text, requested);
        buffer.place_cursor(&buffer.iter_at_offset(offset as i32));
        editor.grab_focus();
        editor.scroll_to_mark(&buffer.get_insert(), 0.15, true, 0.0, 0.5);
    });
}

fn connect_archive_button(
    button: &gtk::Button,
    buffer: &gtk::TextBuffer,
    note: Rc<RefCell<Note>>,
    autosave: AutosaveQueue,
    window: adw::ApplicationWindow,
) {
    let buffer = buffer.clone();
    button.connect_clicked(move |button| {
        save_editor_snapshot(&buffer, &note, &autosave);
        let button = button.clone();
        let previous = note.borrow().clone();
        note_actions::archive(&mut note.borrow_mut(), Utc::now());
        let changed = note.borrow().clone();
        let id = changed.id;
        autosave.schedule(NoteDraft::from(changed));
        let note = note.clone();
        let autosave = autosave.clone();
        let window = window.clone();
        button.set_sensitive(false);
        gtk::glib::MainContext::default().spawn_local(async move {
            if autosave.flush(id).await.is_ok() {
                if let Some(application) = window.application() {
                    application.activate_action("refresh-notes", None);
                }
                window.close();
            } else {
                note.replace(previous);
                button.set_sensitive(true);
                show_save_error(&window);
            }
        });
    });
}

fn connect_trash_button(
    button: &gtk::Button,
    note: Rc<RefCell<Note>>,
    autosave: AutosaveQueue,
    repository: SqliteNoteRepository,
    window: adw::ApplicationWindow,
) {
    button.connect_clicked(move |button| {
        let button = button.clone();
        let note = note.clone();
        let autosave = autosave.clone();
        let repository = repository.clone();
        let window = window.clone();
        button.set_sensitive(false);
        gtk::glib::MainContext::default().spawn_local(async move {
            if !trash_command::confirm_move_to_trash(&window).await {
                button.set_sensitive(true);
                return;
            }
            match trash_command::trash_open_note(&note, &autosave, &repository).await {
                Ok(()) => {
                    if let Some(application) = window.application() {
                        application.activate_action("refresh-notes", None);
                    }
                    window.close();
                }
                Err(_) => {
                    button.set_sensitive(true);
                    show_save_error(&window);
                }
            }
        });
    });
}

fn request_view_mode(
    note: Rc<RefCell<Note>>,
    autosave: AutosaveQueue,
    repository: SqliteNoteRepository,
    presentation: EditorPresentation,
    window: adw::ApplicationWindow,
    busy: Rc<Cell<bool>>,
    enabled: bool,
) {
    if busy.replace(true) {
        return;
    }
    gtk::glib::MainContext::default().spawn_local(async move {
        let id = note.borrow().id;
        if autosave.flush(id).await.is_err() {
            busy.set(false);
            show_save_error(&window);
            return;
        }
        let mut changed = note.borrow().clone();
        if changed.editor_preferences.view_only == enabled {
            presentation.set_view_only(enabled);
            busy.set(false);
            return;
        }
        changed.editor_preferences.view_only = enabled;
        changed.updated_at = Utc::now();
        changed.revision = changed.revision.next();
        match repository.save_note(&changed).await {
            Ok(()) => {
                note.replace(changed);
                presentation.set_view_only(enabled);
                if let Some(application) = window.application() {
                    application.activate_action("refresh-notes", None);
                }
            }
            Err(_) => show_save_error(&window),
        }
        busy.set(false);
    });
}

#[derive(Clone)]
struct ModeSwitchContext {
    window: adw::ApplicationWindow,
    app: adw::Application,
    buffer: gtk::TextBuffer,
    note: Rc<RefCell<Note>>,
    autosave: AutosaveQueue,
    repository: SqliteNoteRepository,
    controller: Arc<dyn WindowController>,
}

fn connect_editor_mode(button: &gtk::Button, target: EditorMode, context: ModeSwitchContext) {
    button.connect_clicked(move |_| {
        save_editor_snapshot(&context.buffer, &context.note, &context.autosave);
        let preview = preview_conversion(&context.note.borrow(), target.clone());
        if preview.from == preview.to {
            return;
        }
        let body = if preview.warnings.is_empty() {
            format!("Switch this note to {}?", editor_mode_name(&target))
        } else {
            format!(
                "{}\n\nA recovery copy will be created before conversion.",
                preview.warnings.join("\n")
            )
        };
        let dialog = adw::AlertDialog::new(Some("Change editor mode?"), Some(&body));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("convert", "Convert");
        dialog.set_response_appearance("convert", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("convert"));
        dialog.set_close_response("cancel");

        let context = context.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            if dialog.choose_future(Some(&context.window)).await != "convert" {
                return;
            }
            let id = context.note.borrow().id;
            if context.autosave.flush(id).await.is_err() {
                show_save_error(&context.window);
                return;
            }
            if !preview.warnings.is_empty()
                && context
                    .repository
                    .duplicate_note(id, Utc::now())
                    .await
                    .is_err()
            {
                show_save_error(&context.window);
                return;
            }
            let original = context.note.borrow().clone();
            apply_conversion(&mut context.note.borrow_mut(), preview);
            context.buffer.set_text(&context.note.borrow().content);
            context
                .autosave
                .schedule(NoteDraft::from(context.note.borrow().clone()));
            if context.autosave.flush(id).await.is_err() {
                *context.note.borrow_mut() = original;
                show_save_error(&context.window);
                return;
            }
            NoteWindow::new(
                &context.app,
                context.note.borrow().clone(),
                context.autosave.clone(),
                context.repository.clone(),
                context.controller.clone(),
            )
            .present();
            context.window.close();
        });
    });
}

fn editor_mode_name(mode: &EditorMode) -> &'static str {
    match mode {
        EditorMode::Rich => "Rich Text",
        EditorMode::Markdown => "Markdown",
        EditorMode::PlainText => "Plain Text",
        EditorMode::Code => "Code",
    }
}

fn connect_export(
    button: &gtk::Button,
    window: &adw::ApplicationWindow,
    note: Rc<RefCell<Note>>,
    markdown: bool,
) {
    let window = window.clone();
    button.connect_clicked(move |_| {
        let window = window.clone();
        let note = note.borrow().clone();

        gtk::glib::MainContext::default().spawn_local(async move {
            let extension = if markdown {
                ExportExtension::Markdown
            } else {
                ExportExtension::PlainText
            };
            let dialog = gtk::FileDialog::builder()
                .title("Export unencrypted note")
                .initial_name(sanitize_export_name(note.display_title(), extension))
                .build();

            if let Ok(file) = dialog.save_future(Some(&window)).await {
                let contents = if markdown {
                    export_markdown(&note)
                } else {
                    export_plain(&note)
                };
                if file
                    .replace_contents_future(
                        contents.into_bytes(),
                        None,
                        false,
                        gtk::gio::FileCreateFlags::REPLACE_DESTINATION,
                    )
                    .await
                    .is_err()
                {
                    show_save_error(&window);
                } else if let Some(path) = file.path() {
                    let _ = set_owner_only(&path);
                }
            }
        });
    });
}
