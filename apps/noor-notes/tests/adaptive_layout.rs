use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::appearance::{AppearanceManager, AppearanceStore, install_global};
use noor_notes::autosave::AutosaveQueue;
use noor_notes::key_store::InMemoryKeyStore;
use noor_notes::ui::adaptive_layout::{
    LibraryLayoutMode, LibraryPaneVisibility, allocation_for_width, apply_library_layout,
};
use noor_notes::ui::library_window::MainWindow;
use noor_notes::writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore};
use noor_storage::{DatabaseKey, SqliteNoteRepository};
use noor_windowing::FallbackWindowController;

#[test]
fn wide_allocation_targets_ten_twenty_seventy_with_readability_guards() {
    let standard = allocation_for_width(LibraryLayoutMode::Wide, 1_180, false);
    assert_eq!(standard.sidebar, 160);
    assert_eq!(standard.collection, 280);
    assert_eq!(standard.navigation, 440);

    let large = allocation_for_width(LibraryLayoutMode::Wide, 1_920, false);
    assert_eq!(large.sidebar, 192);
    assert_eq!(large.collection, 360);
    assert_eq!(large.navigation, 552);
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
    assert_eq!(LibraryLayoutMode::for_width(1_180), LibraryLayoutMode::Wide);
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
        LibraryLayoutMode::Wide
    );
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

#[test]
fn real_shell_allocation_tracks_ratios_and_gives_narrow_preview_the_window_width() {
    gtk::init().unwrap();
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
    real.window.set_default_size(1_180, 760);
    real.present();
    settle();
    let wide = real.layout_snapshot();
    assert!((158..=162).contains(&wide.sidebar_width), "{wide:?}");
    assert!((278..=282).contains(&wide.collection_width), "{wide:?}");
    assert!(wide.document_width > wide.collection_width, "{wide:?}");
    assert!(!wide.back_visible, "{wide:?}");
    assert!(!wide.header_compact, "{wide:?}");
    assert!(!wide.preview_compact, "{wide:?}");
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
    settle();
    let document = narrow.layout_snapshot();
    assert!(document.back_visible, "{document:?}");
    assert!(!document.navigation_visible, "{document:?}");
    assert!(document.document_width >= 600, "{document:?}");
    assert!(document.header_compact, "{document:?}");
    assert!(document.preview_compact, "{document:?}");
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
