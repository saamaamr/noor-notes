use adw::prelude::*;

use crate::appearance::{AppearanceManager, EffectiveTheme};

#[derive(Clone)]
pub struct AppearanceButton {
    pub button: gtk::Button,
}

impl AppearanceButton {
    pub fn new(manager: AppearanceManager) -> Self {
        let button = gtk::Button::builder()
            .icon_name("weather-clear-night-symbolic")
            .focusable(true)
            .build();
        button.add_css_class("flat");
        button.add_css_class("nn-icon-active");
        button.set_accessible_role(gtk::AccessibleRole::Button);

        let toggle_manager = manager.clone();
        button.connect_clicked(move |_| {
            let _ = toggle_manager.toggle_theme();
        });
        let live_button = button.clone();
        manager.subscribe(move |_, theme| {
            let (icon, tooltip) = match theme {
                EffectiveTheme::Snow => ("weather-clear-symbolic", "Switch to Midnight"),
                EffectiveTheme::Midnight => ("weather-clear-night-symbolic", "Switch to Snow"),
            };
            live_button.set_icon_name(icon);
            live_button.set_tooltip_text(Some(tooltip));
            live_button.update_property(&[gtk::accessible::Property::Label(tooltip)]);
        });
        Self { button }
    }
}
