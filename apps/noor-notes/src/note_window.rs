use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use noor_domain::Note;
use noor_windowing::{GnomeWindowController, NativeWindowId, WindowController};

use crate::autosave::{AutosaveQueue, NoteDraft};
use crate::note_toolbar::NoteToolbar;

pub struct NoteWindow {
    pub window: adw::ApplicationWindow,
}

impl NoteWindow {
    pub fn new(
        app: &adw::Application,
        note: Note,
        autosave: AutosaveQueue,
        controller: Arc<dyn WindowController>,
    ) -> Self {
        let note = Rc::new(RefCell::new(note));
        let current = note.borrow().clone();
        let window_title = GnomeWindowController::window_title(&current.id.value().to_string());
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(&window_title)
            .default_width(current.geometry.width)
            .default_height(current.geometry.height)
            .build();
        window.add_css_class("noor-note");
        window.set_opacity(current.style.opacity);

        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let header = adw::HeaderBar::new();
        header.add_css_class("flat");
        let toolbar = NoteToolbar::new();
        header.pack_end(&toolbar.widget);
        layout.append(&header);

        let buffer = gtk::TextBuffer::new(None);
        buffer.set_text(&current.content);
        let editor = gtk::TextView::builder()
            .buffer(&buffer)
            .wrap_mode(gtk::WrapMode::WordChar)
            .left_margin(22)
            .right_margin(22)
            .top_margin(18)
            .bottom_margin(22)
            .accepts_tab(true)
            .build();
        editor.add_css_class("note-editor");
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&editor)
            .build();
        layout.append(&scroller);
        window.set_content(Some(&layout));

        toolbar.pin.set_active(current.always_on_top);
        toolbar.all_workspaces.set_active(current.all_workspaces);
        toolbar.opacity.set_value(current.style.opacity);
        let capabilities = controller.capabilities();
        toolbar.pin.set_sensitive(capabilities.always_on_top);
        toolbar
            .all_workspaces
            .set_sensitive(capabilities.all_workspaces);
        if !capabilities.always_on_top {
            toolbar
                .pin
                .set_tooltip_text(Some("Always on Top is unavailable on this Wayland desktop"));
        }

        {
            let note = note.clone();
            let autosave = autosave.clone();
            buffer.connect_changed(move |buffer| {
                let mut note = note.borrow_mut();
                note.content = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), true)
                    .to_string();
                autosave.schedule(NoteDraft::from(note.clone()));
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let controller = controller.clone();
            toolbar.pin.connect_toggled(move |button| {
                let enabled = button.is_active();
                note.borrow_mut().always_on_top = enabled;
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                if let Some(id) = native_window_id(&window) {
                    let controller = controller.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = controller.set_above(id, enabled).await;
                    });
                }
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let controller = controller.clone();
            toolbar.all_workspaces.connect_toggled(move |button| {
                let enabled = button.is_active();
                note.borrow_mut().all_workspaces = enabled;
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                if let Some(id) = native_window_id(&window) {
                    let controller = controller.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = controller.set_all_workspaces(id, enabled).await;
                    });
                }
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            let window = window.clone();
            let controller = controller.clone();
            toolbar.opacity.connect_value_changed(move |scale| {
                let value = scale.value();
                window.set_opacity(value);
                note.borrow_mut().style.set_opacity(value);
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
                if let Some(id) = native_window_id(&window) {
                    let controller = controller.clone();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let _ = controller.set_opacity(id, value).await;
                    });
                }
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            window.connect_notify_local(Some("width"), move |window, _| {
                note.borrow_mut().geometry.width = window.width();
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }
        {
            let note = note.clone();
            let autosave = autosave.clone();
            window.connect_notify_local(Some("height"), move |window, _| {
                note.borrow_mut().geometry.height = window.height();
                autosave.schedule(NoteDraft::from(note.borrow().clone()));
            });
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
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
