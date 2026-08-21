use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use noor_domain::{Note, NoteId};
use noor_windowing::{NativeWindowId, WindowController};

type ClosedHandler = Rc<dyn Fn()>;
type AlwaysOnTopHandler = Rc<dyn Fn(bool)>;

use crate::ui::note_editor_surface::NoteEditorSurface;

#[derive(Clone)]
pub struct StickyNoteWindow {
    pub window: adw::ApplicationWindow,
    pub always_on_top: gtk::ToggleButton,
    note_id: NoteId,
    closed: Rc<RefCell<Option<ClosedHandler>>>,
    always_on_top_changed: Rc<RefCell<Option<AlwaysOnTopHandler>>>,
    close_notified: Rc<Cell<bool>>,
}

impl StickyNoteWindow {
    pub fn new(app: &gtk::Application, note: Note, controller: Arc<dyn WindowController>) -> Self {
        let surface = NoteEditorSurface::new();
        surface.show_note(&note);
        surface.set_sticky_read_only();
        surface.widget.add_css_class("nn-sticky-body");

        let window_title =
            noor_windowing::GnomeWindowController::window_title(&note.id.value().to_string());
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(&window_title)
            .default_width(note.geometry.width.clamp(260, 560))
            .default_height(note.geometry.height.clamp(220, 720))
            .build();
        window.add_css_class("nn-sticky-note-window");

        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        header.add_css_class("nn-sticky-header");
        let title = adw::WindowTitle::new(note.display_title(), "");
        header.set_title_widget(Some(&title));

        let always_on_top = gtk::ToggleButton::builder()
            .icon_name("view-pin-symbolic")
            .build();
        always_on_top.add_css_class("flat");
        always_on_top.add_css_class("nn-sticky-pin");
        always_on_top.add_css_class("nn-icon-button");
        always_on_top.add_css_class("nn-h-32");
        always_on_top.add_css_class("nn-radius-6");
        always_on_top.add_css_class("nn-focus-ring");
        always_on_top.set_tooltip_text(Some("Keep this note always on top"));
        always_on_top.update_property(&[gtk::accessible::Property::Label(
            "Keep this note always on top",
        )]);
        header.pack_end(&always_on_top);
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&surface.widget));
        window.set_content(Some(&toolbar));

        let capabilities = controller.capabilities();
        always_on_top.set_active(note.always_on_top);
        always_on_top.set_sensitive(capabilities.always_on_top);
        if !capabilities.always_on_top {
            always_on_top.set_tooltip_text(Some("Always on top is unavailable on this desktop"));
        }
        let restoring_above = Rc::new(Cell::new(false));
        let desired_above = Rc::new(Cell::new(note.always_on_top));
        let confirmed_above = Rc::new(Cell::new(note.always_on_top));
        let above_request_running = Rc::new(Cell::new(false));
        let always_on_top_changed = Rc::new(RefCell::new(None::<AlwaysOnTopHandler>));
        {
            let window = window.clone();
            let controller = controller.clone();
            let restoring_above = restoring_above.clone();
            let desired_above = desired_above.clone();
            let confirmed_above = confirmed_above.clone();
            let above_request_running = above_request_running.clone();
            let always_on_top_changed = always_on_top_changed.clone();
            always_on_top.connect_toggled(move |button| {
                if restoring_above.get() {
                    return;
                }
                let Some(id) = native_window_id(&window) else {
                    restoring_above.set(true);
                    button.set_active(confirmed_above.get());
                    restoring_above.set(false);
                    button.set_tooltip_text(Some(
                        "Always on top becomes available after the window is shown",
                    ));
                    return;
                };
                desired_above.set(button.is_active());
                if above_request_running.replace(true) {
                    return;
                }
                let controller = controller.clone();
                let button = button.clone();
                let restoring_above = restoring_above.clone();
                let desired_above = desired_above.clone();
                let confirmed_above = confirmed_above.clone();
                let above_request_running = above_request_running.clone();
                let always_on_top_changed = always_on_top_changed.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    loop {
                        let requested = desired_above.get();
                        match controller.set_above(id.clone(), requested).await {
                            Ok(()) => {
                                confirmed_above.set(requested);
                                if desired_above.get() != requested {
                                    continue;
                                }
                                button.set_tooltip_text(Some("Keep this note always on top"));
                                if let Some(handler) = always_on_top_changed.borrow().as_ref() {
                                    handler(requested);
                                }
                            }
                            Err(_) if desired_above.get() != requested => continue,
                            Err(_) => {
                                let confirmed = confirmed_above.get();
                                desired_above.set(confirmed);
                                restoring_above.set(true);
                                button.set_active(confirmed);
                                restoring_above.set(false);
                                button.set_tooltip_text(Some("Could not change always on top"));
                            }
                        }
                        above_request_running.set(false);
                        break;
                    }
                });
            });
        }

        let closed = Rc::new(RefCell::new(None::<Rc<dyn Fn()>>));
        let close_notified = Rc::new(Cell::new(false));
        {
            let closed = closed.clone();
            let close_notified = close_notified.clone();
            window.connect_close_request(move |_| {
                if !close_notified.replace(true) {
                    if let Some(handler) = closed.borrow().as_ref() {
                        handler();
                    }
                }
                gtk::glib::Propagation::Proceed
            });
        }
        Self {
            window,
            always_on_top,
            note_id: note.id,
            closed,
            always_on_top_changed,
            close_notified,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn close(&self) {
        self.window.close();
    }

    pub fn connect_closed<F: Fn() + 'static>(&self, callback: F) {
        self.closed.replace(Some(Rc::new(callback)));
    }

    pub fn disconnect_closed(&self) {
        self.closed.take();
    }

    pub fn connect_always_on_top_changed<F: Fn(bool) + 'static>(&self, callback: F) {
        self.always_on_top_changed.replace(Some(Rc::new(callback)));
    }

    pub const fn note_id(&self) -> NoteId {
        self.note_id
    }

    pub fn close_was_notified(&self) -> bool {
        self.close_notified.get()
    }
}

fn native_window_id(window: &adw::ApplicationWindow) -> Option<NativeWindowId> {
    let surface = window.surface()?;
    if let Ok(surface) = surface.downcast::<gdk4_x11::X11Surface>() {
        return Some(NativeWindowId::X11(surface.xid() as u32));
    }
    window
        .title()
        .map(|title| NativeWindowId::Wayland(title.to_string()))
}
