use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::appearance::global;
use adw::prelude::*;
use chrono::{DateTime, Utc};
use noor_domain::{Note, NoteId};
use noor_storage::{NoteSort, SqliteNoteRepository, StorageError};

use super::adaptive_layout::{LibraryLayoutMode, allocation_for_width, apply_library_layout};
use super::app_header::AppHeader;
use crate::autosave::{AutosaveQueue, NoteDraft};
use crate::library::{LibrarySection, LibraryState};
use crate::library_preferences::LibraryPreferences;
use crate::services::trash_command;
use crate::sticky_note_window::StickyNoteWindow;
use crate::writing_assistance::WritingAssistanceRuntime;

use super::empty_state::EmptyState;
use super::library_sidebar::LibrarySidebar;
use super::note_card::CardAction;
use super::note_collection::NoteCollection;
use super::note_preview::NotePreview;
use noor_windowing::WindowController;

type CardActionHandler = Rc<dyn Fn(NoteId, CardAction)>;
type PreviewCacheHandler = Rc<dyn Fn(&Note)>;

pub fn preview_edit_handler(
    notes: Rc<RefCell<Vec<Note>>>,
    autosave: AutosaveQueue,
    update_collection_cache: PreviewCacheHandler,
) -> Rc<dyn Fn(Note)> {
    Rc::new(move |edited| {
        if let Some(current) = notes
            .borrow_mut()
            .iter_mut()
            .find(|note| note.id == edited.id)
        {
            *current = edited.clone();
        }
        update_collection_cache(&edited);
        autosave.schedule(NoteDraft::from(edited));
    })
}

#[derive(Clone)]
pub struct MainWindow {
    pub window: adw::ApplicationWindow,
    search_bar: gtk::SearchBar,
    search: gtk::SearchEntry,
    app_header: AppHeader,
    sort: gtk::DropDown,
    sidebar: LibrarySidebar,
    panes: gtk::Paned,
    navigation: gtk::Box,
    sidebar_separator: gtk::Separator,
    back: gtk::Button,
    collection: NoteCollection,
    collection_stack: gtk::Stack,
    empty: EmptyState,
    preview: NotePreview,
    status: gtk::Label,
    results: gtk::Label,
    repository: SqliteNoteRepository,
    autosave: AutosaveQueue,
    writing_runtime: WritingAssistanceRuntime,
    controller: Arc<dyn WindowController>,
    sticky_window: Rc<RefCell<Option<StickyNoteWindow>>>,
    notes: Rc<RefCell<Vec<Note>>>,
    section: Rc<Cell<LibrarySection>>,
    showing_content: Rc<Cell<bool>>,
    refresh_generation: Rc<Cell<u64>>,
}

impl MainWindow {
    pub fn new(
        app: &adw::Application,
        repository: SqliteNoteRepository,
        autosave: AutosaveQueue,
        controller: Arc<dyn WindowController>,
        writing_runtime: WritingAssistanceRuntime,
    ) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(crate::identity::display_name())
            .default_width(1180)
            .default_height(760)
            .width_request(620)
            .height_request(480)
            .build();
        window.add_css_class("nn-library-window");
        let appearance = global();
        appearance.register_window(&window);

        let toolbar = adw::ToolbarView::new();
        let app_header = AppHeader::new(
            appearance,
            LibraryPreferences::for_current_user().load_sort(),
        );
        let back = app_header.back.clone();
        let search_button = app_header.search_toggle.clone();
        let sort = app_header.sort.clone();
        toolbar.add_top_bar(&app_header.widget);

        let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search titles, text, and tags…")
            .hexpand(true)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(16)
            .margin_end(16)
            .build();
        search.add_css_class("nn-search-entry");
        let search_bar = gtk::SearchBar::new();
        search_bar.connect_entry(&search);
        search_bar.set_child(Some(&search));
        search_bar.set_search_mode(false);
        search_button
            .bind_property("active", &search_bar, "search-mode-enabled")
            .bidirectional()
            .sync_create()
            .build();
        page.append(&search_bar);

        let sidebar = LibrarySidebar::new();
        let empty = EmptyState::new();
        let action_holder = Rc::new(RefCell::new(None::<CardActionHandler>));
        let action_proxy: CardActionHandler = {
            let action_holder = action_holder.clone();
            Rc::new(move |id, action| {
                if let Some(handler) = action_holder.borrow().as_ref() {
                    handler(id, action);
                }
            })
        };
        let collection = NoteCollection::new(action_proxy);
        let list_scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&collection.widget)
            .build();
        let collection_stack = gtk::Stack::new();
        collection_stack.add_named(&list_scroll, Some("notes"));
        collection_stack.add_named(&empty.widget, Some("empty"));
        collection_stack.set_visible_child_name("empty");
        let notes = Rc::new(RefCell::new(Vec::new()));
        let sticky_window = Rc::new(RefCell::new(None));
        let collection_cache = collection.clone();
        let finish_holder = Rc::new(RefCell::new(None::<Rc<dyn Fn(NoteId)>>));
        let finish_proxy = {
            let finish_holder = finish_holder.clone();
            Rc::new(move |id| {
                if let Some(handler) = finish_holder.borrow().as_ref() {
                    handler(id);
                }
            })
        };
        let preview = NotePreview::new_with_handlers(
            preview_edit_handler(
                notes.clone(),
                autosave.clone(),
                Rc::new(move |note| collection_cache.update_note(note)),
            ),
            finish_proxy,
        );

        let panes = gtk::Paned::new(gtk::Orientation::Horizontal);
        let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        navigation.append(&sidebar.widget);
        let sidebar_separator = gtk::Separator::new(gtk::Orientation::Vertical);
        sidebar_separator.add_css_class("nn-pane-separator");
        navigation.append(&sidebar_separator);
        navigation.append(&collection_stack);
        panes.set_start_child(Some(&navigation));
        panes.set_end_child(Some(&preview.widget));
        panes.set_resize_start_child(false);
        panes.set_shrink_start_child(false);
        panes.set_vexpand(true);
        page.append(&panes);

        let footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        footer.add_css_class("nn-statusbar");
        let results = gtk::Label::new(Some("Loading library…"));
        results.set_halign(gtk::Align::Start);
        footer.append(&results);
        let status = gtk::Label::new(Some("Local only"));
        status.set_halign(gtk::Align::End);
        status.set_hexpand(true);
        footer.append(&status);
        page.append(&footer);
        toolbar.set_content(Some(&page));
        window.set_content(Some(&toolbar));
        search_bar.set_key_capture_widget(Some(&window));

        let this = Self {
            window,
            search_bar,
            search,
            app_header,
            sort,
            sidebar,
            panes,
            navigation,
            sidebar_separator,
            back,
            collection,
            collection_stack,
            empty,
            preview,
            status,
            results,
            repository,
            autosave,
            writing_runtime,
            controller,
            sticky_window: sticky_window.clone(),
            notes,
            section: Rc::new(Cell::new(LibrarySection::AllNotes)),
            showing_content: Rc::new(Cell::new(false)),
            refresh_generation: Rc::new(Cell::new(0)),
        };
        {
            let this = this.clone();
            let preview = this.preview.clone();
            preview.connect_read_only_changed(move |note, enabled| {
                this.autosave.schedule(NoteDraft::from(note.clone()));
                this.preview.show_note(&note);
                if enabled {
                    if let Some(previous) = this.sticky_window.borrow_mut().take() {
                        previous.close();
                    }
                    let Some(app) = this.window.application() else {
                        return;
                    };
                    let sticky = StickyNoteWindow::new(&app, note.clone(), this.controller.clone());
                    {
                        let this = this.clone();
                        let note = note.clone();
                        sticky.connect_closed(move || {
                            this.sticky_window.borrow_mut().take();
                            let mut note = note.clone();
                            note.editor_preferences.view_only = false;
                            this.autosave.schedule(NoteDraft::from(note.clone()));
                            this.preview.show_note(&note);
                        });
                    }
                    sticky.present();
                    this.sticky_window.replace(Some(sticky));
                } else if let Some(sticky) = this.sticky_window.borrow_mut().take() {
                    sticky.close();
                }
            });
        }
        {
            let this = this.clone();
            *action_holder.borrow_mut() = Some(Rc::new(move |id, action| {
                this.handle_card_action(id, action);
            }));
        }
        {
            let this = this.clone();
            *finish_holder.borrow_mut() = Some(Rc::new(move |id| {
                let this = this.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    match this.autosave.flush(id).await {
                        Ok(()) => this.set_status("Private · Saved locally"),
                        Err(error) => {
                            this.set_status(&format!("Could not save preview edit: {error}"))
                        }
                    }
                });
            }));
        }
        {
            let this = this.clone();
            let sidebar = this.sidebar.clone();
            sidebar.connect_selected(move |section| {
                this.section.set(section);
                this.render_current();
            });
        }
        {
            let this = this.clone();
            this.collection.clone().connect_selected(move |note| {
                if let Some(note) = note {
                    this.preview.show_note(&note);
                    if LibraryLayoutMode::for_window_width(
                        this.window.width(),
                        this.window.default_width(),
                    ) == LibraryLayoutMode::Narrow
                    {
                        this.showing_content.set(true);
                        this.apply_layout();
                    }
                }
            });
        }
        {
            let this = this.clone();
            this.sort.clone().connect_selected_notify(move |dropdown| {
                let sort = sort_from_selected(dropdown.selected());
                let _ = LibraryPreferences::for_current_user().save_sort(sort);
                this.render_current();
            });
        }
        {
            let this = this.clone();
            this.search.clone().connect_search_changed(move |_| {
                let generation = this.refresh_generation.get().wrapping_add(1);
                this.refresh_generation.set(generation);
                let this = this.clone();
                gtk::glib::timeout_add_local_once(Duration::from_millis(180), move || {
                    if this.refresh_generation.get() == generation {
                        this.render_current();
                    }
                });
            });
        }
        {
            let this = this.clone();
            this.window
                .clone()
                .connect_notify_local(Some("width"), move |_, _| this.apply_layout());
        }
        {
            let this = this.clone();
            this.window
                .clone()
                .connect_map(move |_| this.apply_layout());
        }
        {
            let this = this.clone();
            this.back.clone().connect_clicked(move |_| {
                this.showing_content.set(false);
                this.apply_layout();
            });
        }
        this.apply_layout();
        this.refresh();
        this
    }

    fn apply_layout(&self) {
        let allocated_width = self.window.width();
        let window_width = if allocated_width <= 1 {
            self.window.default_width()
        } else {
            allocated_width
        };
        let mode = LibraryLayoutMode::for_width(window_width);
        let visibility = mode.visibility(self.showing_content.get());
        let allocation = allocation_for_width(mode, window_width, self.showing_content.get());
        self.sidebar.set_allocated_width(allocation.sidebar);
        self.sidebar_separator.set_visible(visibility.sidebar);
        self.app_header
            .set_compact(mode == LibraryLayoutMode::Narrow);
        self.preview.set_compact(mode == LibraryLayoutMode::Narrow);
        self.back.set_visible(visibility.back);
        apply_library_layout(
            &self.panes,
            &self.navigation,
            &self.sidebar.widget,
            &self.collection_stack,
            &self.preview.widget,
            mode,
            window_width,
            self.showing_content.get(),
        );
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn focus_search(&self) {
        self.search_bar.set_search_mode(true);
        self.search.grab_focus();
    }

    pub fn set_status(&self, message: &str) {
        self.status.set_text(message);
    }

    pub fn refresh(&self) {
        let generation = self.refresh_generation.get().wrapping_add(1);
        self.refresh_generation.set(generation);
        let repository = self.repository.clone();
        let this = self.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match repository
                .search_notes_sorted("", NoteSort::UpdatedDesc)
                .await
            {
                Ok(notes) if this.refresh_generation.get() == generation => {
                    this.notes.replace(notes);
                    this.render_current();
                }
                Ok(_) => {}
                Err(error) => this.set_status(&format!("Could not load notes: {error}")),
            }
        });
    }

    pub fn create_note(&self) {
        let note = Note::new(Utc::now());
        let repository = self.repository.clone();
        let this = self.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match repository.save_note(&note).await {
                Ok(()) => {
                    this.notes.borrow_mut().push(note.clone());
                    this.section.set(LibrarySection::AllNotes);
                    this.render_current();
                    this.preview.show_note(&note);
                    this.set_status("Private · Saved locally");
                }
                Err(error) => this.set_status(&format!("Could not create note: {error}")),
            }
        });
    }

    fn render_current(&self) {
        let state = LibraryState::new(self.notes.borrow().clone());
        for section in LibrarySection::NAVIGATION {
            self.sidebar.set_count(section, state.count(section));
        }
        let section = self.section.get();
        let projected = state.project(
            section,
            self.search.text().as_str(),
            sort_from_selected(self.sort.selected()),
        );
        // Release the notes borrow before updating GTK widgets. `show_note`
        // synchronizes the editable title, which can emit `changed` and feed
        // back into the preview edit handler; keeping this borrow alive would
        // trigger a RefCell borrow panic during the initial render.
        let visible: Vec<Note> = {
            let notes = self.notes.borrow();
            projected
                .iter()
                .filter_map(|item| notes.iter().find(|note| note.id == item.id).cloned())
                .collect()
        };
        self.collection.set_notes(&visible);
        let searching = !self.search.text().trim().is_empty();
        if visible.is_empty() {
            self.empty.update(section, searching);
            self.collection_stack.set_visible_child_name("empty");
            self.preview.clear();
        } else {
            self.collection_stack.set_visible_child_name("notes");
            self.preview.show_note(&visible[0]);
        }
        self.results
            .set_text(&library_result_summary(section, visible.len(), searching));
        self.set_status("Private · Saved locally");
    }

    fn handle_card_action(&self, id: NoteId, action: CardAction) {
        let this = self.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            if action == CardAction::Trash
                && !trash_command::confirm_move_to_trash(&this.window).await
            {
                return;
            }
            if action == CardAction::DeletePermanently {
                let dialog = adw::AlertDialog::new(
                    Some("Permanently delete this note?"),
                    Some("This cannot be undone. The note and its local history will be removed."),
                );
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Delete Permanently");
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                if dialog.choose_future(Some(&this.window)).await != "delete" {
                    return;
                }
            }
            let result = apply_saved_card_action(&this.repository, id, action, Utc::now()).await;
            match result {
                Ok(()) => {
                    this.writing_runtime
                        .schedule_model_rebuild(Duration::from_secs(5));
                    this.refresh();
                }
                Err(error) => this.set_status(&format!("Note action failed: {error}")),
            }
        });
    }
}

pub fn library_result_summary(
    section: LibrarySection,
    visible_count: usize,
    searching: bool,
) -> String {
    if searching {
        return match visible_count {
            0 => "No results".to_string(),
            1 => "1 result".to_string(),
            count => format!("{count} results"),
        };
    }

    let noun = if visible_count == 1 { "note" } else { "notes" };
    format!("{} · {visible_count} {noun}", section.label())
}

pub async fn apply_saved_card_action(
    repository: &SqliteNoteRepository,
    id: NoteId,
    action: CardAction,
    now: DateTime<Utc>,
) -> Result<(), StorageError> {
    match action {
        CardAction::Archive => repository.archive(id, now).await,
        CardAction::Trash => repository.trash(id, now).await,
        CardAction::Restore => repository.restore(id, now).await,
        CardAction::DeletePermanently => repository.delete_permanently(id).await,
    }
}

fn sort_from_selected(selected: u32) -> NoteSort {
    match selected {
        1 => NoteSort::CreatedDesc,
        2 => NoteSort::TitleAsc,
        3 => NoteSort::TitleDesc,
        _ => NoteSort::UpdatedDesc,
    }
}
