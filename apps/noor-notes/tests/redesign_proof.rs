//! Real widgets and synthetic, temporary notes only. Set NOOR_REDESIGN_PROOF_DIR
//! to save window/popover renders; never captures the desktop or a user database.
use adw::prelude::*;
use chrono::Utc;
use noor_domain::{EditorMode, Note, SourceLanguage};
use noor_notes::{
    appearance::{AppearanceManager, AppearanceMode, AppearanceStore, install_global},
    autosave::AutosaveQueue,
    key_store::InMemoryKeyStore,
    ui::{
        appearance_settings::AppearanceSettings, library_window::MainWindow,
        note_preview::NotePreview, writing_assistance_settings::WritingAssistanceSettings,
    },
    writing_assistance::{WritingAssistanceRuntime, WritingAssistanceStore},
};
use noor_storage::{DatabaseKey, SqliteNoteRepository};
use noor_windowing::FallbackWindowController;
use std::{sync::Arc, time::Duration};

#[test]
fn current_real_surfaces_render_across_both_themes_and_editor_modes() {
    adw::init().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let _guard = runtime.enter();
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    install_global(manager.clone());
    let display = gtk::gdk::Display::default().unwrap();
    manager.install_styles(&display);
    noor_notes::icon_theme::ensure_required_icons(&display);
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.DesignProof")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let repository = runtime
        .block_on(SqliteNoteRepository::open_encrypted(
            &directory.path().join("notes.db"),
            &DatabaseKey::generate(),
        ))
        .unwrap();
    let mut note = Note::new(Utc::now());
    note.title = "A little room to think".into();
    note.content = "Collect ideas. Keep the useful ones.\n\nA calm workspace for everyday writing, with powerful tools when you need them.\n\nToday's focus\nMake something thoughtful. Leave space for the next idea.".into();
    note.favorite = true;
    runtime.block_on(repository.save_note(&note)).unwrap();
    let assistance_store = WritingAssistanceStore::at(directory.path().join("writing.json"));
    let assistance = runtime.block_on(WritingAssistanceRuntime::new(
        repository.clone(),
        assistance_store.clone(),
        Arc::new(InMemoryKeyStore::default()),
    ));
    let main = MainWindow::new(
        &app,
        repository.clone(),
        AutosaveQueue::new(repository, Duration::from_secs(30)),
        Arc::new(FallbackWindowController),
        assistance,
    );
    let new_note = gtk::gio::SimpleAction::new("new-note", None);
    new_note.connect_activate({
        let main = main.clone();
        move |_, _| main.create_note()
    });
    app.add_action(&new_note);
    main.present();
    let preview = NotePreview::new();
    let editor_window = gtk::Window::builder()
        .title("Noor Notes — editor proof")
        .default_width(960)
        .default_height(720)
        .child(&preview.widget)
        .build();
    manager.register_window(&editor_window);
    let appearance = AppearanceSettings::new(&app, manager.clone());
    let writing = WritingAssistanceSettings::new(
        &app,
        assistance_store,
        Arc::new(InMemoryKeyStore::default()),
    );
    for (mode, name) in [
        (AppearanceMode::Snow, "snow"),
        (AppearanceMode::Midnight, "midnight"),
    ] {
        manager.set_mode(mode).unwrap();
        settle();
        capture(
            &main.window,
            main.window.upcast_ref(),
            &format!("{name}-library"),
        );
        assert!(main.layout_snapshot().document_width > main.layout_snapshot().collection_width);
        editor_window.present();
        for (editor_mode, label, content) in [
            (EditorMode::Rich, "rich", note.content.as_str()),
            (
                EditorMode::Markdown,
                "markdown",
                "# A little room to think\n\n**Keep it simple.**\n\n- Write something useful\n- Review tomorrow\n\n```rust\nfn main() { println!(\"Hello\"); }\n```",
            ),
            (
                EditorMode::PlainText,
                "plain",
                "Meeting notes\n\nDecisions\nKeep the interface simple.\n\nNext step: review the working application.",
            ),
            (
                EditorMode::Code,
                "code",
                "// A small, focused program\nfn main() {\n    let message = \"Hello, Noor\";\n    println!(\"{message}\");\n}\n",
            ),
        ] {
            let mut current = note.clone();
            current.editor_mode = editor_mode.clone();
            current.source_language = if editor_mode == EditorMode::Markdown {
                SourceLanguage::Markdown
            } else {
                SourceLanguage::new("rust").unwrap()
            };
            current.content = content.into();
            preview.show_note(&current);
            preview.begin_editing();
            preview.set_available_width(960);
            settle();
            assert_eq!(preview.active_mode(), editor_mode);
            assert!(preview.editor().is_editable());
            assert!(
                !preview.toolbar().find.get_visible(),
                "Standalone-only Find must not leak into the integrated editor"
            );
            assert!(
                preview.toolbar().widget.width() < 650,
                "Toolbar must fit its controls, not fill the 960px pane"
            );
            capture(
                &editor_window,
                editor_window.upcast_ref(),
                &format!("{name}-{label}"),
            );
            if editor_mode == EditorMode::Rich {
                preview.toolbar().format.popup();
                settle();
                capture(
                    &editor_window,
                    preview.toolbar().formatting.widget.upcast_ref(),
                    &format!("{name}-formatting"),
                );
                preview.toolbar().formatting.widget.popdown();
            }
            preview.finish_editing();
        }
        editor_window.set_visible(false);
        appearance.present();
        settle();
        capture(
            &appearance.window,
            appearance.window.upcast_ref(),
            &format!("{name}-appearance"),
        );
        appearance.window.set_visible(false);
        writing.present();
        settle();
        assert!(writing.endpoint.height() <= 40);
        capture(
            &writing.window,
            writing.window.upcast_ref(),
            &format!("{name}-writing"),
        );
        writing.window.set_visible(false);
    }
    main.window.close();
    editor_window.close();
    appearance.window.close();
    writing.window.close();
}

fn capture(window: &impl IsA<gtk::Window>, widget: &gtk::Widget, name: &str) {
    let Ok(directory) = std::env::var("NOOR_REDESIGN_PROOF_DIR") else {
        return;
    };
    std::fs::create_dir_all(&directory).unwrap();
    let paintable = gtk::WidgetPaintable::new(Some(widget));
    widget.queue_draw();
    settle();
    let mut rendered = None;
    for _ in 0..10 {
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, widget.width() as f64, widget.height() as f64);
        rendered = snapshot.to_node();
        if rendered.is_some() {
            break;
        }
        widget.queue_draw();
        settle();
    }
    let node = rendered.unwrap_or_else(|| {
        panic!(
            "{name} must render: {}x{} visible={} mapped={}",
            widget.width(),
            widget.height(),
            widget.is_visible(),
            widget.is_mapped()
        )
    });
    window
        .as_ref()
        .renderer()
        .unwrap()
        .render_texture(&node, None)
        .save_to_png(format!("{directory}/{name}.png"))
        .unwrap();
}

fn settle() {
    for _ in 0..30 {
        while gtk::glib::MainContext::default().iteration(false) {}
        std::thread::sleep(Duration::from_millis(10));
    }
}
