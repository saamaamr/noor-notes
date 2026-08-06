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

        let cycle_manager = manager.clone();
        button.connect_clicked(move |_| {
            let _ = cycle_manager.cycle_dark_palette();
        });
        let live_button = button.clone();
        manager.subscribe(move |_, theme| {
            let (active, next) = match theme {
                EffectiveTheme::Light => ("Light", "Graphite"),
                EffectiveTheme::Graphite => ("Graphite", "Midnight"),
                EffectiveTheme::Midnight => ("Midnight", "OLED"),
                EffectiveTheme::Oled => ("OLED", "Graphite"),
            };
            let description = format!("Dark palette: {active}. Click for {next}");
            live_button.set_tooltip_text(Some(&description));
            live_button.update_property(&[gtk::accessible::Property::Label(&description)]);
        });
        Self { button }
    }
}
