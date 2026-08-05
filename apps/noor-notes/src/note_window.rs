use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{Note, NoteState};
use noor_storage::SqliteNoteRepository;
use noor_windowing::{GnomeWindowController, NativeWindowId, WindowController};

use crate::autosave::{AutosaveQueue, NoteDraft};
use crate::export::{export_markdown, export_plain};
use crate::modern_toolbar::ModernToolbar;
use crate::note_actions;
use crate::note_find::FindResults;
use crate::rich_buffer::RichBuffer;
use crate::save_status::SaveStatusIndicator;

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
        window.add_css_class("noor-note");
        window.set_opacity(current.style.opacity);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let header = adw::HeaderBar::new();
        header.add_css_class("flat");
        let toolbar = ModernToolbar::new();
        let is_trashed = matches!(current.state, NoteState::Trashed { .. });
        toolbar.archive.set_visible(!is_trashed);
        toolbar.trash.set_visible(!is_trashed);
        toolbar.restore.set_visible(is_trashed);
        toolbar.permanent_delete.set_visible(is_trashed);
        header.pack_end(&toolbar.widget);
        layout.append(&header);
        let title_entry = gtk::Entry::builder()
            .text(current.display_title())
            .placeholder_text("Untitled note")
            .editable(!is_trashed)
            .build();
        title_entry.add_css_class("note-title-entry");
        title_entry.set_hexpand(true);
        let save_status = SaveStatusIndicator::new();
        let title_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        title_row.add_css_class("note-title-row");
        title_row.append(&title_entry);
        title_row.append(&save_status.widget);
        layout.append(&title_row);

        let buffer = gtk::TextBuffer::new(None);
        RichBuffer::load(&buffer, &current.content, current.rich_content.as_ref());
        let editor = gtk::TextView::builder()
            .buffer(&buffer)
            .wrap_mode(gtk::WrapMode::WordChar)
            .left_margin(22)
            .right_margin(22)
            .top_margin(18)
            .bottom_margin(22)
            .accepts_tab(true)
            .build();
        editor.add_css_class("note-editor");
        editor.set_editable(!is_trashed);
        let find_entry = gtk::SearchEntry::builder()
            .placeholder_text("Find in note…")
            .hexpand(true)
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
        let find_bar = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        find_bar.add_css_class("find-bar");
        find_bar.append(&find_entry);
        find_bar.append(&find_count);
        find_bar.append(&find_previous);
        find_bar.append(&find_next);
        find_bar.set_visible(false);
        layout.append(&find_bar);
        let find_results = Rc::new(RefCell::new(FindResults::default()));
        {
            let buffer = buffer.clone();
            let results = find_results.clone();
            let count = find_count.clone();
            find_entry.connect_search_changed(move |entry| {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
                results.borrow_mut().update(&text, &entry.text());
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
        layout.append(&scroller);
        window.set_content(Some(&layout));
        crate::editor_actions::connect(&toolbar, &buffer, &editor);
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
        connect_export(&toolbar.export_markdown, &window, note.clone(), true);

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
        {
            let note = note.clone();
            let autosave = autosave.clone();
            buffer.connect_changed(move |buffer| {
                let mut note = note.borrow_mut();
                let (content, rich_content) = RichBuffer::snapshot(buffer);
                note.content = content;
                note.rich_content = Some(rich_content);
                autosave.schedule(NoteDraft::from(note.clone()));
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
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            toolbar.archive.connect_clicked(move |button| {
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
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            toolbar.trash.connect_clicked(move |button| {
                let button = button.clone();
                let note = note.clone();
                let autosave = autosave.clone();
                let window = window.clone();
                button.set_sensitive(false);
                gtk::glib::MainContext::default().spawn_local(async move {
                    let dialog = adw::AlertDialog::new(
                        Some("Move this note to Trash?"),
                        Some("The note will remain recoverable from the Trash section."),
                    );
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("trash", "Move to Trash");
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");
                    dialog.set_response_appearance("trash", adw::ResponseAppearance::Destructive);
                    if dialog.choose_future(Some(&window)).await != "trash" {
                        button.set_sensitive(true);
                        return;
                    }
                    let previous = note.borrow().clone();
                    note_actions::trash(&mut note.borrow_mut(), Utc::now());
                    let changed = note.borrow().clone();
                    let id = changed.id;
                    autosave.schedule(NoteDraft::from(changed));
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
            window.connect_close_request(move |window| {
                if allow_close.get() || !autosave.has_pending(id) {
                    return gtk::glib::Propagation::Proceed;
                }
                let autosave = autosave.clone();
                let window = window.clone();
                let allow_close = allow_close.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    if autosave.flush(id).await.is_ok() {
                        allow_close.set(true);
                        window.close();
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
            let extension = if markdown { "md" } else { "txt" };
            let dialog = gtk::FileDialog::builder()
                .title("Export note")
                .initial_name(format!("{}.{}", note.display_title(), extension))
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
                }
            }
        });
    });
}
