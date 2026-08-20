use std::sync::Arc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::sticky_note_window::StickyNoteWindow;
use noor_windowing::FallbackWindowController;

const TOOLBAR: &str = include_str!("../src/ui/editor_toolbar.rs");
const WINDOW: &str = include_str!("../src/sticky_note_window.rs");
const WINDOWING: &str = include_str!("../../../crates/windowing/src/controller.rs");

#[test]
fn sticky_window_has_explicit_always_on_top_and_read_only_controls() {
    assert!(TOOLBAR.contains("Always on Top"));
    assert!(TOOLBAR.contains("Read-only"));
    assert!(WINDOW.contains("GnomeWindowController::window_title"));
    assert!(WINDOWING.contains("set_always_on_top") || WINDOWING.contains("always_on_top"));
}

#[test]
fn sticky_window_has_one_title_and_a_body_only_document_surface() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.StickyPresentationTest")
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let mut note = Note::new(Utc::now());
    note.title = "One window title".into();
    note.content = "Body content only".into();

    let gtk_app: gtk::Application = app.clone().upcast();
    let sticky = StickyNoteWindow::new(&gtk_app, note, Arc::new(FallbackWindowController));
    assert!(sticky.window.has_css_class("nn-sticky-note-window"));
    assert_eq!(
        descendants(sticky.window.clone().upcast())
            .into_iter()
            .filter(|widget| widget.has_css_class("nn-display-title"))
            .count(),
        0
    );
    assert_eq!(
        descendants(sticky.window.clone().upcast())
            .into_iter()
            .filter(|widget| widget.has_css_class("nn-sticky-body"))
            .count(),
        1
    );
    assert_eq!(
        sticky.always_on_top.tooltip_text().as_deref(),
        Some("Always on top is unavailable on this desktop")
    );
    assert!(!sticky.always_on_top.is_sensitive());
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
