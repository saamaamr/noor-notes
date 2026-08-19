use std::sync::Arc;

use adw::prelude::*;
use noor_domain::Note;
use noor_windowing::{NativeWindowId, WindowController};

use crate::identity;
use crate::ui::note_editor_surface::NoteEditorSurface;

#[derive(Clone)]
pub struct StickyNoteWindow {
    pub window: adw::ApplicationWindow,
}

impl StickyNoteWindow {
    pub fn new(app: &gtk::Application, note: Note, controller: Arc<dyn WindowController>) -> Self {
        let surface = NoteEditorSurface::new();
        surface.show_note(&note);
        surface.set_sticky_read_only();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(identity::display_name())
            .default_width(note.geometry.width.clamp(260, 560))
            .default_height(note.geometry.height.clamp(220, 720))
            .build();
        window.add_css_class("nn-sticky-note-window");

        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        header.add_css_class("nn-app-header");
        let title = adw::WindowTitle::new(note.display_title(), "Read-only note");
        header.set_title_widget(Some(&title));

        let always_on_top = gtk::ToggleButton::with_label("Always on top");
        always_on_top.set_tooltip_text(Some("Keep this sticky note above other windows"));
        always_on_top.update_property(&[gtk::accessible::Property::Label(
            "Keep sticky note always on top",
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
        {
            let window = window.clone();
            let controller = controller.clone();
            always_on_top.connect_toggled(move |button| {
                let Some(id) = native_window_id(&window) else {
                    return;
                };
                let controller = controller.clone();
                let enabled = button.is_active();
                gtk::glib::MainContext::default().spawn_local(async move {
                    let _ = controller.set_above(id, enabled).await;
                });
            });
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn close(&self) {
        self.window.close();
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
