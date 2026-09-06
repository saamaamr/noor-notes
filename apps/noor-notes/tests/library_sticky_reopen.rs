use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::{
    appearance::{AppearanceManager, AppearanceStore, install_global},
    autosave::AutosaveQueue,
    key_store::InMemoryKeyStore,
    ui::library_window::{MainWindow, StickySession},
    writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore},
};
use noor_storage::SqliteNoteRepository;
use noor_windowing::FallbackWindowController;
use std::{sync::Arc, time::Duration};

#[test]
fn reopened_main_can_close_surviving_sticky_and_follow_its_close_state() {
    adw::init().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let _guard = runtime.enter();
    let directory = tempfile::tempdir().unwrap();
    install_global(AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    )));
    let repository = runtime
        .block_on(SqliteNoteRepository::open(
            &directory.path().join("notes.db"),
        ))
        .unwrap();
    let note = Note::new(Utc::now());
    runtime.block_on(repository.save_note(&note)).unwrap();
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_millis(10));
    let writing = runtime.block_on(WritingAssistanceRuntime::new(
        repository.clone(),
        WritingAssistanceStore::at(directory.path().join("writing.json")),
        Arc::new(InMemoryKeyStore::default()),
    ));
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.StickyReopenTest")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let sticky_session = StickySession::default();
    let create = || {
        MainWindow::new_with_sticky_session(
            &app,
            repository.clone(),
            autosave.clone(),
            Arc::new(FallbackWindowController),
            writing.clone(),
            sticky_session.clone(),
        )
    };
    let first = create();
    first.present();
    wait_until(|| {
        descendants(first.window.clone().upcast())
            .iter()
            .filter_map(|w| w.downcast_ref::<gtk::Label>())
            .any(|label| label.has_css_class("nn-preview-title") && label.text() == "Untitled note")
    });
    read_only(&first).emit_clicked();
    wait_until(|| stickies(&app).len() == 1);
    gtk::glib::MainContext::default()
        .block_on(autosave.flush(note.id))
        .unwrap();
    first.window.close();
    wait_until(|| app.windows().len() == 1);
    let second = create();
    second.present();
    wait_until(|| read_only(&second).label().as_deref() == Some("Exit read-only"));
    read_only(&second).emit_clicked();
    wait_until(|| stickies(&app).is_empty());
    wait_until(|| read_only(&second).label().as_deref() == Some("Read-only"));
    read_only(&second).emit_clicked();
    wait_until(|| stickies(&app).len() == 1);
    gtk::glib::MainContext::default()
        .block_on(autosave.flush(note.id))
        .unwrap();
    second.window.close();
    let third = create();
    third.present();
    wait_until(|| read_only(&third).label().as_deref() == Some("Exit read-only"));
    stickies(&app)[0].close();
    wait_until(|| read_only(&third).label().as_deref() == Some("Read-only"));
    assert!(stickies(&app).is_empty());
    third.window.close();
    gtk::glib::MainContext::default()
        .block_on(autosave.flush(note.id))
        .unwrap();
    assert!(
        !gtk::glib::MainContext::default()
            .block_on(repository.get_note(note.id))
            .unwrap()
            .unwrap()
            .editor_preferences
            .view_only
    );
}

fn read_only(main: &MainWindow) -> gtk::Button {
    descendants(main.window.clone().upcast())
        .into_iter()
        .find(|w| w.has_css_class("nn-preview-read-only-button"))
        .unwrap()
        .downcast()
        .unwrap()
}

fn stickies(app: &adw::Application) -> Vec<gtk::Window> {
    app.windows()
        .into_iter()
        .filter(|w| w.has_css_class("nn-sticky-note-window"))
        .collect()
}

fn descendants(root: gtk::Widget) -> Vec<gtk::Widget> {
    let mut widgets = vec![root.clone()];
    let mut child = root.first_child();
    while let Some(current) = child {
        widgets.extend(descendants(current.clone()));
        child = current.next_sibling();
    }
    widgets
}

#[track_caller]
fn wait_until(mut ready: impl FnMut() -> bool) {
    for _ in 0..300 {
        while gtk::glib::MainContext::default().iteration(false) {}
        if ready() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(ready(), "expected real main/sticky state transition");
}
