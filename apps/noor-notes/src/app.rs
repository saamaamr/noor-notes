use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_windowing::{
    BackendKind, Environment, FallbackWindowController, WindowController, X11WindowController,
    detect_backend,
};

use crate::autosave::AutosaveQueue;
use crate::key_store::Oo7KeyStore;
use crate::note_window::NoteWindow;
use crate::security_bootstrap::open_repository;

pub async fn run() -> anyhow::Result<gtk::glib::ExitCode> {
    let keys = Arc::new(Oo7KeyStore::new().await?);
    let repository = open_repository(&data_path(), keys).await?;
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_millis(400));
    let controller: Arc<dyn WindowController> = match detect_backend(&Environment::current()) {
        BackendKind::X11 => X11WindowController::connect()
            .map(|controller| Arc::new(controller) as Arc<dyn WindowController>)
            .unwrap_or_else(|_| Arc::new(FallbackWindowController)),
        BackendKind::GnomeWayland | BackendKind::Fallback => Arc::new(FallbackWindowController),
    };
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes")
        .build();
    app.connect_startup(|_| load_css());
    app.connect_activate(move |app| {
        let note = Note::new(Utc::now());
        let window = NoteWindow::new(
            app,
            note,
            autosave.clone(),
            repository.clone(),
            controller.clone(),
        );
        window.present();
    });
    Ok(app.run())
}

fn data_path() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".local/share"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("noor-notes/notes.db")
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("../resources/design-system.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
