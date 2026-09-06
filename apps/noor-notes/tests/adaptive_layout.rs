use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::appearance::{AppearanceManager, AppearanceStore, install_global};
use noor_notes::autosave::AutosaveQueue;
use noor_notes::key_store::InMemoryKeyStore;
use noor_notes::library::LibrarySection;
use noor_notes::ui::adaptive_layout::{
    EditorLayoutDensity, LibraryLayoutMode, LibraryPaneVisibility, allocation_for_width,
    apply_library_layout, editor_content_width, editor_layout_density,
};
use noor_notes::ui::app_header::AppHeader;
use noor_notes::ui::library_window::MainWindow;
use noor_notes::writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore};
use noor_storage::{DatabaseKey, SqliteNoteRepository};
use noor_windowing::FallbackWindowController;

#[test]
fn wide_allocation_targets_ten_eighteen_and_remaining_document_width() {
    let standard = allocation_for_width(LibraryLayoutMode::Wide, 1_180, false);
    assert_eq!(standard.sidebar, 160);
    assert_eq!(standard.collection, 280);
    assert_eq!(standard.navigation, 440);

    let large = allocation_for_width(LibraryLayoutMode::Wide, 1_920, false);
    assert_eq!(large.sidebar, 192);
    assert_eq!(large.collection, 346);
    assert_eq!(large.navigation, 538);
}

#[test]
fn medium_and_narrow_allocations_prioritize_the_visible_destination() {
    let medium = allocation_for_width(LibraryLayoutMode::Medium, 900, false);
    assert_eq!(medium.sidebar, 0);
    assert_eq!(medium.collection, 306);
    assert_eq!(medium.navigation, 306);

    assert_eq!(
        allocation_for_width(LibraryLayoutMode::Narrow, 620, true).navigation,
        0
    );
    assert_eq!(
        allocation_for_width(LibraryLayoutMode::Narrow, 620, false).navigation,
        620
    );
}

#[test]
fn width_breakpoints_choose_one_stable_library_mode() {
    assert_eq!(LibraryLayoutMode::for_width(1_536), LibraryLayoutMode::Wide);
    assert_eq!(LibraryLayoutMode::for_width(1_200), LibraryLayoutMode::Wide);
    assert_eq!(
        LibraryLayoutMode::for_width(1_199),
        LibraryLayoutMode::Medium
    );
    assert_eq!(
        LibraryLayoutMode::for_width(1_180),
        LibraryLayoutMode::Medium
    );
    assert_eq!(
        LibraryLayoutMode::for_width(1_079),
        LibraryLayoutMode::Medium
    );
    assert_eq!(LibraryLayoutMode::for_width(760), LibraryLayoutMode::Medium);
    assert_eq!(LibraryLayoutMode::for_width(759), LibraryLayoutMode::Narrow);
    assert_eq!(
        LibraryLayoutMode::for_window_width(720, 1_180),
        LibraryLayoutMode::Narrow
    );
    assert_eq!(
        LibraryLayoutMode::for_window_width(1, 1_180),
        LibraryLayoutMode::Medium
    );
}

#[test]
fn editor_content_width_follows_the_available_pane_ratio() {
    assert_eq!(editor_content_width(520), 520);
    assert_eq!(editor_content_width(760), 699);
    assert_eq!(editor_content_width(1_200), 860);
    assert_eq!(editor_content_width(1_600), 860);

    assert_eq!(editor_layout_density(900), EditorLayoutDensity::Spacious);
    assert_eq!(editor_layout_density(620), EditorLayoutDensity::Compact);
    assert_eq!(editor_layout_density(430), EditorLayoutDensity::Narrow);
}

#[test]
fn wide_and_medium_modes_keep_content_visible_without_squeezing_navigation() {
    assert_eq!(
        LibraryLayoutMode::Wide.visibility(false),
        LibraryPaneVisibility {
            sidebar: true,
            collection: true,
            content: true,
            back: false,
        }
    );
    assert_eq!(
        LibraryLayoutMode::Medium.visibility(false),
        LibraryPaneVisibility {
            sidebar: false,
            collection: true,
            content: true,
            back: false,
        }
    );
    assert_eq!(LibraryLayoutMode::Wide.pane_position(1_180, false), 440);
    assert_eq!(LibraryLayoutMode::Medium.pane_position(900, false), 306);
}

#[test]
fn narrow_mode_switches_between_collection_and_content_with_back_navigation() {
    assert_eq!(
        LibraryLayoutMode::Narrow.visibility(false),
        LibraryPaneVisibility {
            sidebar: false,
            collection: true,
            content: false,
            back: false,
        }
    );
    assert_eq!(
        LibraryLayoutMode::Narrow.visibility(true),
        LibraryPaneVisibility {
            sidebar: false,
            collection: false,
            content: true,
            back: true,
        }
    );
}

fn assert_adaptive_header_keeps_every_library_destination_and_sort_option_reachable() {
    let directory = tempfile::tempdir().unwrap();
    let header = AppHeader::new(
        AppearanceManager::new(AppearanceStore::at(
            directory.path().join("appearance.json"),
        )),
        noor_storage::NoteSort::UpdatedDesc,
    );
    let selected = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    header.connect_navigation_selected({
        let selected = selected.clone();
        move |section| selected.borrow_mut().push(section)
    });

    header.set_adaptive(true, false);
    assert!(header.navigation.is_visible());
    assert!(header.sort.is_visible());
    for section in LibrarySection::NAVIGATION {
        header.navigation_button(section).unwrap().emit_clicked();
    }
    assert_eq!(&*selected.borrow(), &LibrarySection::NAVIGATION);

    header.set_adaptive(true, true);
    assert!(header.navigation.is_visible());
    assert!(!header.sort.is_visible());
    assert!(header.compact_sort.is_visible());
    for (index, button) in header.compact_sort_buttons().iter().enumerate() {
        button.emit_clicked();
        assert_eq!(header.sort.selected(), index as u32);
    }
}

#[test]
fn real_shell_allocation_tracks_ratios_and_gives_narrow_preview_the_window_width() {
    gtk::init().unwrap();
    assert_adaptive_header_keeps_every_library_destination_and_sort_option_reachable();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _runtime_guard = runtime.enter();
    let directory = tempfile::tempdir().unwrap();
    let repository = runtime
        .block_on(SqliteNoteRepository::open_encrypted(
            &directory.path().join("notes.db"),
            &DatabaseKey::generate(),
        ))
        .unwrap();
    runtime
        .block_on(repository.save_note(&Note::new(Utc::now())))
        .unwrap();
    let assistance = runtime.block_on(WritingAssistanceRuntime::new(
        repository.clone(),
        WritingAssistanceStore::at(directory.path().join("writing.json")),
        Arc::new(InMemoryKeyStore::default()),
    ));
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_secs(30));
    install_global(AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    )));
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.ResponsiveTest")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();

    let real = MainWindow::new(
        &app,
        repository,
        autosave,
        Arc::new(FallbackWindowController),
        assistance,
    );
    real.window.set_default_size(1_480, 760);
    real.present();
    real.window.unmaximize();
    real.window.set_default_size(1_480, 760);
    settle();
    let resized = real.layout_snapshot();
    if real.window.is_maximized() {
        assert!(resized.window_width >= 1_536, "{resized:?}");
        assert!(resized.navigation_visible, "{resized:?}");
        assert!(!resized.header_compact, "{resized:?}");
    } else {
        // CSD shadows may occupy part of the requested native surface width.
        // Allocation must follow the actual content width, not the outer request.
        assert_eq!(resized.window_width, real.window.width(), "{resized:?}");
        assert!(resized.window_width >= 1_440, "{resized:?}");
        assert!(resized.navigation_visible, "{resized:?}");
        assert!(!resized.header_compact, "{resized:?}");
    }
    assert!(
        resized.document_width > resized.collection_width,
        "{resized:?}"
    );
    assert!(!resized.back_visible, "{resized:?}");
    assert!(!resized.preview_compact, "{resized:?}");

    let shell_widgets = descendants(real.window.clone().upcast());
    let sidebar = shell_widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-sidebar"))
        .expect("library sidebar")
        .clone();
    let editor = shell_widgets
        .iter()
        .find_map(|widget| widget.clone().downcast::<gtk::TextView>().ok())
        .filter(|editor| editor.has_css_class("nn-preview-editor"))
        .expect("integrated editor");

    assert!(sidebar.is_visible(), "{resized:?}");
    assert!(
        editor.left_margin() <= 16,
        "margin={}",
        editor.left_margin()
    );
    real.window.close();

    let repository = runtime
        .block_on(SqliteNoteRepository::open_encrypted(
            &directory.path().join("notes-narrow.db"),
            &DatabaseKey::generate(),
        ))
        .unwrap();
    runtime
        .block_on(repository.save_note(&Note::new(Utc::now())))
        .unwrap();
    let assistance = runtime.block_on(WritingAssistanceRuntime::new(
        repository.clone(),
        WritingAssistanceStore::at(directory.path().join("writing-narrow.json")),
        Arc::new(InMemoryKeyStore::default()),
    ));
    let narrow = MainWindow::new(
        &app,
        repository.clone(),
        AutosaveQueue::new(repository, Duration::from_secs(30)),
        Arc::new(FallbackWindowController),
        assistance,
    );
    narrow.window.set_default_size(620, 760);
    narrow.present();
    narrow.window.unmaximize();
    narrow.window.set_default_size(620, 760);
    settle();
    let document = narrow.layout_snapshot();
    if narrow.window.is_maximized() {
        assert!(document.window_width >= 1_536, "{document:?}");
        assert!(!document.back_visible, "{document:?}");
        assert!(document.navigation_visible, "{document:?}");
        assert!(!document.header_compact, "{document:?}");
        assert!(!document.preview_compact, "{document:?}");
    } else {
        assert_eq!(document.window_width, narrow.window.width(), "{document:?}");
        assert!((600..=640).contains(&document.window_width), "{document:?}");
        assert!(document.back_visible, "{document:?}");
        assert!(!document.navigation_visible, "{document:?}");
        assert!(document.document_width >= 600, "{document:?}");
        assert!(document.header_compact, "{document:?}");
        assert!(document.preview_compact, "{document:?}");
    }
    narrow.window.close();

    let window = gtk::Window::builder()
        .default_width(1_180)
        .default_height(480)
        .build();
    let panes = gtk::Paned::new(gtk::Orientation::Horizontal);
    let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let collection = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let preview = gtk::Box::new(gtk::Orientation::Vertical, 0);
    navigation.append(&sidebar);
    navigation.append(&collection);
    panes.set_start_child(Some(&navigation));
    panes.set_end_child(Some(&preview));
    panes.set_resize_start_child(false);
    panes.set_shrink_start_child(false);
    window.set_child(Some(&panes));
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    apply_library_layout(
        &panes,
        &navigation,
        &sidebar,
        &collection,
        &preview,
        LibraryLayoutMode::Wide,
        1_180,
        false,
    );
    panes.allocate(1_180, 480, -1, None);
    while gtk::glib::MainContext::default().iteration(false) {}

    assert_eq!(sidebar.width_request(), 160);
    assert_eq!(collection.width_request(), 280);
    assert_eq!(panes.position(), 440);
    assert!(preview.width() >= 700, "preview={}", preview.width());

    apply_library_layout(
        &panes,
        &navigation,
        &sidebar,
        &collection,
        &preview,
        LibraryLayoutMode::Narrow,
        620,
        true,
    );
    panes.allocate(620, 480, -1, None);
    while gtk::glib::MainContext::default().iteration(false) {}

    let position = panes.position();
    let preview_width = preview.width();
    assert!(!navigation.is_visible());
    assert!(preview.is_visible());
    assert!(
        preview_width > 500,
        "position={position} preview={preview_width}"
    );
    window.close();
}

fn settle() {
    for _ in 0..80 {
        while gtk::glib::MainContext::default().iteration(false) {}
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn descendants(root: gtk::Widget) -> Vec<gtk::Widget> {
    let mut widgets = Vec::new();
    let mut pending = vec![root];
    while let Some(widget) = pending.pop() {
        let mut child = widget.first_child();
        while let Some(next) = child {
            pending.push(next.clone());
            child = next.next_sibling();
        }
        widgets.push(widget);
    }
    widgets
}
