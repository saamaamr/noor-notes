use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use noor_windowing::{
    BackendKind, Environment, FallbackWindowController, GnomeWindowController, WindowController,
    X11WindowController, detect_backend,
};

use crate::actions::add_action;
use crate::appearance::{AppearanceManager, AppearanceStore, global, install_global};
use crate::autosave::AutosaveQueue;
use crate::import_dialog::ImportFlow;
use crate::key_store::Oo7KeyStore;
use crate::main_window::MainWindow;
use crate::security_bootstrap::open_repository;
use crate::shortcuts::shortcuts_window;
use crate::ui::appearance_settings::AppearanceSettings;
use crate::ui::dialog_primitives;
use crate::ui::writing_assistance_settings::WritingAssistanceSettings;
use crate::writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore};

pub async fn run() -> anyhow::Result<gtk::glib::ExitCode> {
    let keys = Arc::new(Oo7KeyStore::new().await?);
    let repository = open_repository(&data_path(), keys.clone()).await?;
    let writing_runtime = WritingAssistanceRuntime::new(
        repository.clone(),
        WritingAssistanceStore::for_current_user(),
        keys.clone(),
    )
    .await;
    writing_runtime.rebuild_if_stale().await?;
    let autosave_runtime = writing_runtime.clone();
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_millis(400))
        .with_success_hook(move || {
            autosave_runtime.schedule_model_rebuild(Duration::from_secs(5));
        });
    let controller = window_controller().await;
    let app = crate::identity::application();
    let appearance = AppearanceManager::new(AppearanceStore::for_current_user());
    appearance.install_action(&app);
    install_global(appearance.clone());
    let startup_appearance = appearance.clone();
    app.connect_startup(move |_| {
        if let Some(display) = gtk::gdk::Display::default() {
            startup_appearance.install_styles(&display);
        }
    });
    let main_window: Rc<RefCell<Option<MainWindow>>> = Rc::new(RefCell::new(None));

    {
        let main_window = main_window.clone();
        add_action(&app.clone(), "new-note", move |_, _| {
            if let Some(window) = main_window.borrow().as_ref() {
                window.create_note();
            }
        });
    }
    {
        let app = app.clone();
        let settings: Rc<RefCell<Option<WritingAssistanceSettings>>> = Rc::new(RefCell::new(None));
        let keys: Arc<dyn crate::key_store::KeyStore> = keys.clone();
        add_action(&app.clone(), "writing-assistance-settings", move |_, _| {
            if settings.borrow().is_none() {
                settings.replace(Some(WritingAssistanceSettings::new(
                    &app,
                    WritingAssistanceStore::for_current_user(),
                    keys.clone(),
                )));
            }
            if let Some(settings) = settings.borrow().as_ref() {
                settings.present();
            }
        });
    }
    {
        let main_window = main_window.clone();
        add_action(&app, "show-notes", move |_, _| {
            if let Some(window) = main_window.borrow().as_ref() {
                window.present();
            }
        });
    }
    {
        let main_window = main_window.clone();
        add_action(&app, "refresh-notes", move |_, _| {
            if let Some(window) = main_window.borrow().as_ref() {
                window.refresh();
            }
        });
    }
    {
        let main_window = main_window.clone();
        add_action(&app, "search", move |_, _| {
            if let Some(window) = main_window.borrow().as_ref() {
                window.present();
                window.focus_search();
            }
        });
    }
    {
        let repository = repository.clone();
        let main_window = main_window.clone();
        add_action(&app, "import-xpad", move |_, _| {
            let Some(window) = main_window.borrow().as_ref().cloned() else {
                return;
            };
            let source = std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_default()
                .join(".config/xpad");
            let flow = match ImportFlow::from_path(&source) {
                Ok(flow) => flow,
                Err(error) => {
                    window.set_status(&format!("Could not inspect Xpad notes: {error}"));
                    return;
                }
            };
            let body = format!(
                "{} notes are ready to import. {} files will be skipped and reported. Xpad files will not be changed.",
                flow.preview().importable.len(),
                flow.preview().skipped.len()
            );
            let repository = repository.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                if dialog_primitives::confirm_action(
                    &window.window,
                    "Import Xpad notes?",
                    &body,
                    "Import",
                )
                .await
                {
                    match flow.confirm(&repository).await {
                        Ok(report) => {
                            window.set_status(&format!(
                                "Imported {} notes; {} already imported; {} skipped",
                                report.imported,
                                report.already_imported,
                                report.skipped.len()
                            ));
                            window.refresh();
                        }
                        Err(error) => window.set_status(&format!("Import failed: {error}")),
                    }
                }
            });
        });
    }
    {
        let main_window = main_window.clone();
        add_action(&app, "sync-now", move |_, _| {
            if let Some(window) = main_window.borrow().as_ref() {
                window.set_status("Cloud sync is not configured yet · Local notes are safe");
            }
        });
    }
    {
        let main_window = main_window.clone();
        add_action(&app, "shortcuts", move |_, _| {
            if let Some(window) = main_window.borrow().as_ref() {
                let dialog = shortcuts_window();
                dialog.set_transient_for(Some(&window.window));
                dialog.present();
            }
        });
    }
    {
        let app = app.clone();
        let settings: Rc<RefCell<Option<AppearanceSettings>>> = Rc::new(RefCell::new(None));
        add_action(&app.clone(), "appearance-settings", move |_, _| {
            if settings.borrow().is_none() {
                settings.replace(Some(AppearanceSettings::new(&app, global())));
            }
            if let Some(settings) = settings.borrow().as_ref() {
                settings.present();
            }
        });
    }
    {
        let app = app.clone();
        add_action(&app.clone(), "quit", move |_, _| app.quit());
    }

    app.set_accels_for_action("app.new-note", &["<Primary>n"]);
    app.set_accels_for_action("app.search", &["<Primary>f"]);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
    app.set_accels_for_action("app.shortcuts", &["<Primary>question"]);
    {
        let main_window = main_window.clone();
        let repository = repository.clone();
        let autosave = autosave.clone();
        let controller = controller.clone();
        let writing_runtime = writing_runtime.clone();
        app.connect_activate(move |app| {
            if main_window.borrow().is_none() {
                main_window.replace(Some(MainWindow::new(
                    app,
                    repository.clone(),
                    autosave.clone(),
                    controller.clone(),
                    writing_runtime.clone(),
                )));
            }
            if let Some(window) = main_window.borrow().as_ref() {
                window.present();
            }
        });
    }
    Ok(app.run())
}

async fn window_controller() -> Arc<dyn WindowController> {
    match detect_backend(&Environment::current()) {
        BackendKind::X11 => X11WindowController::connect()
            .map(|controller| Arc::new(controller) as Arc<dyn WindowController>)
            .unwrap_or_else(|_| Arc::new(FallbackWindowController)),
        BackendKind::GnomeWayland => GnomeWindowController::connect()
            .await
            .map(|controller| Arc::new(controller) as Arc<dyn WindowController>)
            .unwrap_or_else(|_| Arc::new(FallbackWindowController)),
        BackendKind::Fallback => Arc::new(FallbackWindowController),
    }
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
