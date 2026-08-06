use adw::prelude::*;
use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore};
use noor_notes::ui::appearance_button::AppearanceButton;
use noor_notes::ui::appearance_settings::AppearanceSettings;

#[test]
fn header_button_and_settings_share_all_appearance_choices() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    manager.set_mode(AppearanceMode::Graphite).unwrap();

    let control = AppearanceButton::new(manager.clone());
    assert_eq!(
        control.button.tooltip_text().as_deref(),
        Some("Dark palette: Graphite. Click for Midnight")
    );
    assert!(control.button.has_css_class("nn-icon-active"));
    control.button.emit_clicked();
    assert_eq!(manager.preferences().mode, AppearanceMode::Midnight);
    assert_eq!(
        control.button.tooltip_text().as_deref(),
        Some("Dark palette: Midnight. Click for OLED")
    );

    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.AppearanceTest")
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let settings = AppearanceSettings::new(&app, manager);
    assert_eq!(settings.choice_count(), 5);
    assert_eq!(settings.window.title().as_deref(), Some("Appearance"));
}
