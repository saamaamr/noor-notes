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
            let _ = cycle_manager.cycle_palette();
        });
        let live_button = button.clone();
        manager.subscribe(move |_, theme| {
            let (family, active, next, icon) = match theme {
                EffectiveTheme::Light => (
                    "Light palette",
                    "Snow",
                    "Warm Paper",
                    "weather-clear-symbolic",
                ),
                EffectiveTheme::WarmPaper => (
                    "Light palette",
                    "Warm Paper",
                    "Cool Mist",
                    "weather-clear-symbolic",
                ),
                EffectiveTheme::CoolMist => (
                    "Light palette",
                    "Cool Mist",
                    "Snow",
                    "weather-clear-symbolic",
                ),
                EffectiveTheme::Graphite => (
                    "Dark palette",
                    "Graphite",
                    "Midnight",
                    "weather-clear-night-symbolic",
                ),
                EffectiveTheme::Midnight => (
                    "Dark palette",
                    "Midnight",
                    "OLED",
                    "weather-clear-night-symbolic",
                ),
                EffectiveTheme::Oled => (
                    "Dark palette",
                    "OLED",
                    "Graphite",
                    "weather-clear-night-symbolic",
                ),
            };
            let description = format!("{family}: {active}. Click for {next}");
            live_button.set_icon_name(icon);
            live_button.set_tooltip_text(Some(&description));
            live_button.update_property(&[gtk::accessible::Property::Label(&description)]);
        });
        Self { button }
    }
}
