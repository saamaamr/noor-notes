use adw::prelude::*;

use crate::appearance::{AppearanceManager, AppearanceMode};

#[derive(Clone)]
pub struct AppearanceSettings {
    pub window: adw::PreferencesWindow,
    choices: Vec<adw::ActionRow>,
}

impl AppearanceSettings {
    pub fn new(app: &adw::Application, manager: AppearanceManager) -> Self {
        let window = adw::PreferencesWindow::builder()
            .application(app)
            .title("Appearance")
            .default_width(560)
            .default_height(520)
            .search_enabled(false)
            .build();
        window.add_css_class("nn-appearance-settings");
        window.add_css_class("nn-settings-window");
        manager.register_window(&window);

        let page = adw::PreferencesPage::new();
        page.set_title("Appearance");
        page.set_icon_name(Some("applications-graphics-symbolic"));
        let group = adw::PreferencesGroup::new();
        group.add_css_class("nn-settings-group");
        group.set_title("Theme");
        group.set_description(Some(
            "System follows GNOME and remembers your preferred light and dark palettes.",
        ));

        let mut choices = Vec::new();
        let mut previous: Option<gtk::CheckButton> = None;
        let mut selectors = Vec::new();
        for (mode, title, subtitle, swatch) in [
            (
                AppearanceMode::System,
                "System",
                "Follow the desktop appearance",
                "nn-swatch-system",
            ),
            (
                AppearanceMode::Light,
                "Snow",
                "Clean neutral surfaces",
                "nn-swatch-light",
            ),
            (
                AppearanceMode::WarmPaper,
                "Warm Paper",
                "Soft ivory surfaces for comfortable reading",
                "nn-swatch-warm-paper",
            ),
            (
                AppearanceMode::CoolMist,
                "Cool Mist",
                "Calm blue-gray productivity surfaces",
                "nn-swatch-cool-mist",
            ),
            (
                AppearanceMode::Graphite,
                "Graphite",
                "Warm charcoal and restrained indigo",
                "nn-swatch-graphite",
            ),
            (
                AppearanceMode::Midnight,
                "Midnight",
                "Deep navy and calm sky blue",
                "nn-swatch-midnight",
            ),
            (
                AppearanceMode::Oled,
                "OLED",
                "Near-black surfaces and vivid violet-blue",
                "nn-swatch-oled",
            ),
        ] {
            let row = adw::ActionRow::builder()
                .title(title)
                .subtitle(subtitle)
                .activatable(true)
                .build();
            row.add_css_class("nn-settings-row");
            let preview = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            preview.set_size_request(42, 26);
            preview.add_css_class("nn-theme-swatch");
            preview.add_css_class(swatch);
            row.add_prefix(&preview);
            let check = gtk::CheckButton::new();
            if let Some(group) = previous.as_ref() {
                check.set_group(Some(group));
            }
            check.set_active(manager.preferences().mode == mode);
            row.add_suffix(&check);
            row.set_activatable_widget(Some(&check));
            let selection = manager.clone();
            check.connect_toggled(move |check| {
                if check.is_active() {
                    let _ = selection.set_mode(mode);
                }
            });
            previous = Some(check.clone());
            selectors.push((mode, check.clone()));
            group.add(&row);
            choices.push(row);
        }
        manager.subscribe(move |preferences, _| {
            for (mode, check) in &selectors {
                if *mode == preferences.mode && !check.is_active() {
                    check.set_active(true);
                }
            }
        });

        page.add(&group);
        window.add(&page);
        Self { window, choices }
    }

    pub fn choice_count(&self) -> usize {
        self.choices.len()
    }

    pub fn choice_rows(&self) -> Vec<adw::ActionRow> {
        self.choices.clone()
    }

    pub fn present(&self) {
        self.window.present();
    }
}
