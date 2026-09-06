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

    assert_sticky_body_uses_window_width_for_responsive_spacing(&gtk_app);
    assert_body_starts_below_header_with_responsive_insets(&gtk_app);
    assert_rapid_always_on_top_uses_only_the_latest_intent(&gtk_app);
}

fn assert_body_starts_below_header_with_responsive_insets(app: &gtk::Application) {
    use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore};
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(directory.path().join("theme.json")));
    manager.install_styles(&gtk::gdk::Display::default().unwrap());
    let mut note = Note::new(Utc::now());
    note.title = "Reading space".into();
    note.content = format!("Body starts here.\n\n{}", "long-token-".repeat(80));
    let sticky = StickyNoteWindow::new(app, note, Arc::new(FallbackWindowController));
    manager.register_window(&sticky.window);
    let widgets = descendants(sticky.window.clone().upcast());
    let surface = widgets
        .iter()
        .find(|w| w.has_css_class("nn-sticky-body"))
        .unwrap();
    let body = widgets
        .iter()
        .find(|w| w.has_css_class("nn-preview-body"))
        .unwrap();
    for theme in [AppearanceMode::Snow, AppearanceMode::Midnight] {
        manager.set_mode(theme).unwrap();
        let mut small_inset = 0.0;
        for width in [320, 560, 960, 320] {
            sticky.window.set_default_size(width, 480);
            sticky.present();
            for _ in 0..30 {
                settle();
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let bounds = body.compute_bounds(surface).unwrap();
            assert!(
                (10.0..=28.0).contains(&bounds.y()),
                "body must start just below header, not hidden editor chrome: {bounds:?}"
            );
            assert!(
                bounds.x() >= 10.0 && bounds.x() < surface.width() as f32 * 0.15,
                "comfortable left inset: {bounds:?}"
            );
            assert!(
                bounds.x() + bounds.width() <= surface.width() as f32 - 10.0,
                "long tokens must wrap inside body: {bounds:?}"
            );
            if width == 320 {
                if small_inset > 0.0 {
                    assert_eq!(bounds.y(), small_inset);
                }
                small_inset = bounds.y();
            } else {
                assert!(
                    bounds.y() > small_inset,
                    "larger window needs more breathing room"
                );
            }
            assert!(
                widgets
                    .iter()
                    .filter(|w| w.has_css_class("nn-preview-heading"))
                    .all(|w| !w.is_mapped())
            );
            assert!(
                widgets
                    .iter()
                    .filter_map(|w| w.downcast_ref::<gtk::TextView>())
                    .all(|v| !v.is_editable())
            );
            if let Ok(directory) = std::env::var("NOOR_STICKY_PROOF_DIR") {
                std::fs::create_dir_all(&directory).unwrap();
                let paintable = gtk::WidgetPaintable::new(Some(&sticky.window));
                sticky.window.queue_draw();
                let mut node = None;
                for _ in 0..30 {
                    settle();
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    let snapshot = gtk::Snapshot::new();
                    paintable.snapshot(
                        &snapshot,
                        sticky.window.width() as f64,
                        sticky.window.height() as f64,
                    );
                    node = snapshot.to_node();
                    if node.is_some() {
                        break;
                    }
                }
                sticky
                    .window
                    .renderer()
                    .unwrap()
                    .render_texture(node.expect("real sticky render"), None)
                    .save_to_png(format!("{directory}/{theme:?}-{width}.png"))
                    .unwrap();
            }
        }
    }
    sticky.window.close();
}

fn assert_sticky_body_uses_window_width_for_responsive_spacing(app: &gtk::Application) {
    let compact = sticky_body_at_width(app, 540);
    assert!(compact.has_css_class("compact"));
    assert!(!compact.has_css_class("narrow"));

    let narrow = sticky_body_at_width(app, 320);
    assert!(narrow.has_css_class("compact"));
    assert!(narrow.has_css_class("narrow"));
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

fn sticky_body_at_width(app: &gtk::Application, width: i32) -> gtk::Widget {
    let mut note = Note::new(Utc::now());
    note.geometry.width = width;
    let sticky = StickyNoteWindow::new(app, note, Arc::new(FallbackWindowController));
    let body = descendants(sticky.window.clone().upcast())
        .into_iter()
        .find(|widget| widget.has_css_class("nn-sticky-body"))
        .expect("sticky body");
    sticky.window.close();
    body
}
