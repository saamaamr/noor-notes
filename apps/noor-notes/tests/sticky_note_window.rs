use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use adw::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::sticky_note_window::StickyNoteWindow;
use noor_windowing::{
    FallbackWindowController, NativeWindowId, WindowCapabilities, WindowController, WindowError,
};
use tokio::sync::Notify;

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
    sticky.window.close();

    assert_rapid_always_on_top_uses_only_the_latest_intent(&gtk_app);
}

struct DelayedWindowController {
    calls: Mutex<Vec<bool>>,
    release_first: Notify,
}

#[async_trait]
impl WindowController for DelayedWindowController {
    async fn set_above(&self, _window: NativeWindowId, enabled: bool) -> Result<(), WindowError> {
        let first = {
            let mut calls = self.calls.lock().unwrap();
            calls.push(enabled);
            calls.len() == 1
        };
        if first {
            self.release_first.notified().await;
        }
        Ok(())
    }

    async fn set_all_workspaces(
        &self,
        _window: NativeWindowId,
        _enabled: bool,
    ) -> Result<(), WindowError> {
        Ok(())
    }

    async fn set_opacity(&self, _window: NativeWindowId, _value: f64) -> Result<(), WindowError> {
        Ok(())
    }

    fn capabilities(&self) -> WindowCapabilities {
        WindowCapabilities {
            always_on_top: true,
            ..WindowCapabilities::default()
        }
    }
}

fn assert_rapid_always_on_top_uses_only_the_latest_intent(app: &gtk::Application) {
    let controller = Arc::new(DelayedWindowController {
        calls: Mutex::new(Vec::new()),
        release_first: Notify::new(),
    });
    let sticky = StickyNoteWindow::new(app, Note::new(Utc::now()), controller.clone());
    let persisted = Rc::new(RefCell::new(Vec::new()));
    sticky.connect_always_on_top_changed({
        let persisted = persisted.clone();
        move |enabled| persisted.borrow_mut().push(enabled)
    });
    sticky.present();
    settle_until(|| sticky.window.surface().is_some());

    sticky.always_on_top.set_active(true);
    settle_until(|| controller.calls.lock().unwrap().len() == 1);
    sticky.always_on_top.set_active(false);
    settle();
    controller.release_first.notify_one();
    settle_until(|| controller.calls.lock().unwrap().len() == 2);
    settle_until(|| !persisted.borrow().is_empty());

    assert_eq!(&*controller.calls.lock().unwrap(), &[true, false]);
    assert_eq!(&*persisted.borrow(), &[false]);
    assert!(!sticky.always_on_top.is_active());
    sticky.window.close();
}

fn settle() {
    while gtk::glib::MainContext::default().iteration(false) {}
}

fn settle_until(mut ready: impl FnMut() -> bool) {
    for _ in 0..500 {
        settle();
        if ready() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    panic!("timed out waiting for GTK state");
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
