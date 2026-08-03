use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_storage::SqliteNoteRepository;
use noor_windowing::{
    BackendKind, Environment, FallbackWindowController, WindowController, X11WindowController,
    detect_backend,
};

use crate::actions::add_action;
use crate::autosave::AutosaveQueue;
use crate::import_dialog::ImportFlow;
use crate::main_window::MainWindow;
use crate::note_window::NoteWindow;

pub async fn run() -> anyhow::Result<gtk::glib::ExitCode> {
    let repository = SqliteNoteRepository::open(&data_path()).await?;
    let autosave = AutosaveQueue::new(repository.clone(), Duration::from_millis(400));
    let controller = window_controller();
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes")
        .build();
    app.connect_startup(|_| load_css());
    let main_window: Rc<RefCell<Option<MainWindow>>> = Rc::new(RefCell::new(None));

    {
        let app = app.clone();
        let autosave = autosave.clone();
        let controller = controller.clone();
        add_action(&app.clone(), "new-note", move |_, _| {
            NoteWindow::new(
                &app,
                Note::new(Utc::now()),
                autosave.clone(),
                controller.clone(),
            )
            .present();
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
            let dialog = adw::AlertDialog::new(Some("Import Xpad notes?"), Some(&body));
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("import", "Import");
            dialog.set_response_appearance("import", adw::ResponseAppearance::Suggested);
            let repository = repository.clone();
            gtk::glib::MainContext::default().spawn_local(async move {
                if dialog.choose_future(Some(&window.window)).await == "import" {
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
        let app = app.clone();
        add_action(&app.clone(), "quit", move |_, _| app.quit());
    }

    app.set_accels_for_action("app.new-note", &["<Primary>n"]);
    app.set_accels_for_action("app.search", &["<Primary>f"]);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
    {
        let main_window = main_window.clone();
        let repository = repository.clone();
        let autosave = autosave.clone();
        let controller = controller.clone();
        app.connect_activate(move |app| {
            if main_window.borrow().is_none() {
                main_window.replace(Some(MainWindow::new(
                    app,
                    repository.clone(),
                    autosave.clone(),
                    controller.clone(),
                )));
            }
            if let Some(window) = main_window.borrow().as_ref() {
                window.present();
            }
        });
    }
    Ok(app.run())
}

fn window_controller() -> Arc<dyn WindowController> {
    match detect_backend(&Environment::current()) {
        BackendKind::X11 => X11WindowController::connect()
            .map(|controller| Arc::new(controller) as Arc<dyn WindowController>)
            .unwrap_or_else(|_| Arc::new(FallbackWindowController)),
        BackendKind::GnomeWayland | BackendKind::Fallback => Arc::new(FallbackWindowController),
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

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("../resources/style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
