use std::sync::Arc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{Note, NoteId, NoteState};
use noor_storage::{NoteSort, SqliteNoteRepository};
use noor_windowing::WindowController;

use crate::autosave::AutosaveQueue;
use crate::library_preferences::LibraryPreferences;
use crate::note_window::NoteWindow;

#[derive(Clone)]
pub struct MainWindow {
    pub window: adw::ApplicationWindow,
    search: gtk::SearchEntry,
    sort: gtk::DropDown,
    active: gtk::ListBox,
    archived: gtk::ListBox,
    trash: gtk::ListBox,
    status: gtk::Label,
    repository: SqliteNoteRepository,
    autosave: AutosaveQueue,
    controller: Arc<dyn WindowController>,
    app: adw::Application,
}

impl MainWindow {
    pub fn new(
        app: &adw::Application,
        repository: SqliteNoteRepository,
        autosave: AutosaveQueue,
        controller: Arc<dyn WindowController>,
    ) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Noor Notes")
            .default_width(780)
            .default_height(560)
            .build();
        window.add_css_class("main-window");

        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Noor Notes", "Private notes, available offline");
        header.set_title_widget(Some(&title));
        let new_button = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("New note")
            .build();
        new_button.set_action_name(Some("app.new-note"));
        header.pack_start(&new_button);
        let import_button = gtk::Button::builder()
            .icon_name("document-open-symbolic")
            .tooltip_text("Import Xpad notes")
            .build();
        import_button.set_action_name(Some("app.import-xpad"));
        header.pack_end(&import_button);
        toolbar.add_top_bar(&header);

        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_top(12);
        page.set_margin_bottom(12);
        page.set_margin_start(18);
        page.set_margin_end(18);
        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search notes…")
            .hexpand(true)
            .build();
        let sort =
            gtk::DropDown::from_strings(&["Recently updated", "Title A–Z", "Newest created"]);
        let preferences = LibraryPreferences::for_current_user();
        sort.set_selected(match preferences.load_sort() {
            NoteSort::UpdatedDesc => 0,
            NoteSort::TitleAsc => 1,
            NoteSort::CreatedDesc => 2,
        });
        sort.set_tooltip_text(Some("Sort notes"));
        let filters = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        filters.append(&search);
        filters.append(&sort);
        page.append(&filters);

        let stack = adw::ViewStack::new();
        let active = note_list();
        let archived = note_list();
        let trash = note_list();
        stack.add_titled_with_icon(&active, Some("active"), "Notes", "note-symbolic");
        stack.add_titled_with_icon(&archived, Some("archived"), "Archived", "folder-symbolic");
        stack.add_titled_with_icon(&trash, Some("trash"), "Trash", "user-trash-symbolic");
        let switcher = adw::ViewSwitcher::builder()
            .stack(&stack)
            .policy(adw::ViewSwitcherPolicy::Wide)
            .build();
        page.append(&switcher);
        page.append(&stack);
        let status = gtk::Label::new(Some("Local only · All changes saved offline"));
        status.add_css_class("dim-label");
        status.set_halign(gtk::Align::Start);
        page.append(&status);
        toolbar.set_content(Some(&page));
        window.set_content(Some(&toolbar));

        let this = Self {
            window,
            search,
            sort,
            active,
            archived,
            trash,
            status,
            repository,
            autosave,
            controller,
            app: app.clone(),
        };
        {
            let this = this.clone();
            this.sort.clone().connect_selected_notify(move |dropdown| {
                let sort = sort_from_selected(dropdown.selected());
                let _ = LibraryPreferences::for_current_user().save_sort(sort);
                this.refresh();
            });
        }
        {
            let this = this.clone();
            this.search.clone().connect_search_changed(move |_| {
                this.refresh();
            });
        }
        this.refresh();
        this
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn focus_search(&self) {
        self.search.grab_focus();
    }

    pub fn set_status(&self, message: &str) {
        self.status.set_text(message);
    }

    pub fn refresh(&self) {
        let query = self.search.text().to_string();
        let sort = sort_from_selected(self.sort.selected());
        let repository = self.repository.clone();
        let this = self.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match repository.search_notes_sorted(&query, sort).await {
                Ok(notes) => this.render(notes, !query.trim().is_empty()),
                Err(error) => this.set_status(&format!("Could not load notes: {error}")),
            }
        });
    }

    fn connect_restore(&self, button: &gtk::Button, id: NoteId) {
        let this = self.clone();
        button.connect_clicked(move |button| {
            button.set_sensitive(false);
            let button = button.clone();
            let this = this.clone();
            let repository = this.repository.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                match repository.restore(id, Utc::now()).await {
                    Ok(()) => this.refresh(),
                    Err(error) => {
                        button.set_sensitive(true);
                        this.set_status(&format!("Could not restore note: {error}"));
                    }
                }
            });
        });
    }

    fn connect_permanent_delete(&self, button: &gtk::Button, id: NoteId) {
        let this = self.clone();
        button.connect_clicked(move |button| {
            button.set_sensitive(false);
            let button = button.clone();
            let this = this.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                let dialog = adw::AlertDialog::new(
                    Some("Permanently delete this note?"),
                    Some("This cannot be undone. The note and its local history will be removed."),
                );
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Permanently Delete");
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                if dialog.choose_future(Some(&this.window)).await != "delete" {
                    button.set_sensitive(true);
                    return;
                }
                match this.repository.delete_permanently(id).await {
                    Ok(()) => this.refresh(),
                    Err(error) => {
                        button.set_sensitive(true);
                        this.set_status(&format!("Could not permanently delete note: {error}"));
                    }
                }
            });
        });
    }

    fn attach_trash_actions(&self, row: &adw::ActionRow, id: NoteId) {
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let restore = gtk::Button::with_label("Restore");
        restore.add_css_class("suggested-action");
        let delete = gtk::Button::with_label("Permanently Delete");
        delete.add_css_class("destructive-action");
        self.connect_restore(&restore, id);
        self.connect_permanent_delete(&delete, id);
        actions.append(&restore);
        actions.append(&delete);
        row.add_suffix(&actions);

        let menu_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        menu_box.set_margin_top(6);
        menu_box.set_margin_bottom(6);
        menu_box.set_margin_start(6);
        menu_box.set_margin_end(6);
        let menu_restore = gtk::Button::with_label("Restore");
        let menu_delete = gtk::Button::with_label("Permanently Delete");
        menu_delete.add_css_class("destructive-action");
        self.connect_restore(&menu_restore, id);
        self.connect_permanent_delete(&menu_delete, id);
        menu_box.append(&menu_restore);
        menu_box.append(&menu_delete);
        let popover = gtk::Popover::builder().child(&menu_box).build();
        popover.set_parent(row);
        let gesture = gtk::GestureClick::new();
        gesture.set_button(3);
        gesture.connect_pressed(move |_, _, _, _| popover.popup());
        row.add_controller(gesture);
    }

    fn render(&self, notes: Vec<Note>, searching: bool) {
        clear_list(&self.active);
        clear_list(&self.archived);
        clear_list(&self.trash);
        for note in notes {
            let target = match note.state {
                NoteState::Active => &self.active,
                NoteState::Archived => &self.archived,
                NoteState::Trashed { .. } => &self.trash,
            };
            let row = adw::ActionRow::builder()
                .title(note.display_title())
                .subtitle(note_subtitle(&note))
                .activatable(true)
                .build();
            if matches!(note.state, NoteState::Trashed { .. }) {
                self.attach_trash_actions(&row, note.id);
            }
            let app = self.app.clone();
            let autosave = self.autosave.clone();
            let controller = self.controller.clone();
            let repository = self.repository.clone();
            row.connect_activated(move |_| {
                NoteWindow::new(
                    &app,
                    note.clone(),
                    autosave.clone(),
                    repository.clone(),
                    controller.clone(),
                )
                .present();
            });
            target.append(&row);
        }
        append_empty_state(
            &self.active,
            if searching {
                "No matching active notes"
            } else {
                "No active notes yet"
            },
        );
        append_empty_state(
            &self.archived,
            if searching {
                "No matching archived notes"
            } else {
                "No archived notes"
            },
        );
        append_empty_state(
            &self.trash,
            if searching {
                "No matching trashed notes"
            } else {
                "Trash is empty"
            },
        );
        self.set_status("Local only · All changes saved offline");
    }
}

fn note_list() -> gtk::ListBox {
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list
}

fn clear_list(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn sort_from_selected(selected: u32) -> NoteSort {
    match selected {
        1 => NoteSort::TitleAsc,
        2 => NoteSort::CreatedDesc,
        _ => NoteSort::UpdatedDesc,
    }
}

fn note_subtitle(note: &Note) -> String {
    let date = note.updated_at.format("%d %b %Y · %I:%M %p");

    if note.tags.is_empty() {
        date.to_string()
    } else {
        format!(
            "{} · {}",
            date,
            note.tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join("  ")
        )
    }
}

fn append_empty_state(list: &gtk::ListBox, message: &str) {
    if list.first_child().is_none() {
        let row = adw::ActionRow::builder()
            .title(message)
            .subtitle("Create or restore a note to see it here")
            .activatable(false)
            .build();
        row.add_css_class("empty-state");
        list.append(&row);
    }
}
