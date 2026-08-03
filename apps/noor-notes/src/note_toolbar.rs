use adw::prelude::*;

#[derive(Clone)]
pub struct NoteToolbar {
    pub widget: gtk::Box,
    pub pin: gtk::ToggleButton,
    pub all_workspaces: gtk::ToggleButton,
    pub opacity: gtk::Scale,
    pub archive: gtk::Button,
    pub trash: gtk::Button,
}

impl NoteToolbar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        widget.add_css_class("note-toolbar");

        let pin = gtk::ToggleButton::builder()
            .icon_name("view-pin-symbolic")
            .tooltip_text("Always on Top")
            .build();
        let all_workspaces = gtk::ToggleButton::builder()
            .icon_name("focus-windows-symbolic")
            .tooltip_text("Show on all workspaces")
            .build();
        let opacity = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.35, 1.0, 0.05);
        opacity.set_width_request(110);
        opacity.set_draw_value(false);
        opacity.set_tooltip_text(Some("Note opacity"));
        let archive = gtk::Button::builder()
            .icon_name("folder-symbolic")
            .tooltip_text("Archive")
            .build();
        let trash = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Move to Trash")
            .build();

        widget.append(&pin);
        widget.append(&all_workspaces);
        widget.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        widget.append(&gtk::Image::from_icon_name("weather-clear-symbolic"));
        widget.append(&opacity);
        widget.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        widget.append(&archive);
        widget.append(&trash);

        Self {
            widget,
            pin,
            all_workspaces,
            opacity,
            archive,
            trash,
        }
    }
}

impl Default for NoteToolbar {
    fn default() -> Self {
        Self::new()
    }
}
