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
            "Choose a calm daytime or comfortable nighttime theme.",
        ));

        let mut choices = Vec::new();
        let mut previous: Option<gtk::CheckButton> = None;
        let mut selectors = Vec::new();
        for (mode, title, subtitle, swatch) in [
            (
                AppearanceMode::Snow,
                "Snow",
                "Clean daytime theme",
                "nn-swatch-snow",
            ),
            (
                AppearanceMode::Midnight,
                "Midnight",
                "Comfortable dark theme",
                "nn-swatch-midnight",
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
            preview.set_valign(gtk::Align::Center);
            preview.add_css_class("nn-theme-swatch");
            preview.add_css_class(swatch);
            row.add_prefix(&preview);
            let check = gtk::CheckButton::new();
            if let Some(group) = previous.as_ref() {
                check.set_group(Some(group));
            }
            let selected_mode = match manager.effective_theme() {
                crate::appearance::EffectiveTheme::Snow => AppearanceMode::Snow,
                crate::appearance::EffectiveTheme::Midnight => AppearanceMode::Midnight,
            };
            check.set_active(selected_mode == mode);
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
        manager.subscribe(move |_, theme| {
            let selected_mode = match theme {
                crate::appearance::EffectiveTheme::Snow => AppearanceMode::Snow,
                crate::appearance::EffectiveTheme::Midnight => AppearanceMode::Midnight,
            };
            for (mode, check) in &selectors {
                if *mode == selected_mode && !check.is_active() {
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
