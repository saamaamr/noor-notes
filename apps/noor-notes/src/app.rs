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
use crate::writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore};

pub async fn run() -> anyhow::Result<gtk::glib::ExitCode> {
    let keys = Arc::new(Oo7KeyStore::new().await?);
    let repository = open_repository(&data_path(), keys.clone()).await?;
    let writing_runtime = WritingAssistanceRuntime::new(
        repository.clone(),
        WritingAssistanceStore::for_current_user(),
        keys,
    )
    .await;
    writing_runtime.rebuild_if_stale().await?;
    let autosave_runtime = writing_runtime.clone();
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_millis(400))
        .with_success_hook(move || {
            autosave_runtime.schedule_model_rebuild(Duration::from_secs(5));
        });
    let controller: Arc<dyn WindowController> = match detect_backend(&Environment::current()) {
        BackendKind::X11 => X11WindowController::connect()
            .map(|controller| Arc::new(controller) as Arc<dyn WindowController>)
            .unwrap_or_else(|_| Arc::new(FallbackWindowController)),
        BackendKind::GnomeWayland | BackendKind::Fallback => Arc::new(FallbackWindowController),
    };
    let app = crate::identity::application();
    app.connect_startup(|_| {
        if let Some(display) = gtk::gdk::Display::default() {
            crate::appearance::install_static_styles(
                &display,
                crate::appearance::EffectiveTheme::Snow,
            );
        }
    });
    let runtime = writing_runtime.clone();
    app.connect_activate(move |app| {
        let note = Note::new(Utc::now());
        let window = NoteWindow::new(
            app,
            note,
            autosave.clone(),
            repository.clone(),
            controller.clone(),
            runtime.clone(),
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
