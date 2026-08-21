use adw::prelude::*;
use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore};
use noor_notes::ui::appearance_button::AppearanceButton;
use noor_notes::ui::appearance_settings::AppearanceSettings;

#[test]
fn header_toggle_and_settings_expose_only_two_themes() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    manager.set_mode(AppearanceMode::Snow).unwrap();

    let control = AppearanceButton::new(manager.clone());
    assert_eq!(
        control.button.tooltip_text().as_deref(),
        Some("Switch to Midnight")
    );
    assert!(control.button.has_css_class("nn-icon-active"));
    control.button.emit_clicked();
    assert_eq!(manager.preferences().mode, AppearanceMode::Midnight);
    assert_eq!(
        control.button.tooltip_text().as_deref(),
        Some("Switch to Snow")
    );

    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.AppearanceTest")
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let settings = AppearanceSettings::new(&app, manager);
    assert_eq!(settings.choice_count(), 2);
    assert_eq!(settings.window.title().as_deref(), Some("Appearance"));
    assert!(settings.window.has_css_class("nn-settings-window"));
    let titles: Vec<_> = settings
        .choice_rows()
        .iter()
        .map(|row| row.title())
        .collect();
    assert_eq!(titles, ["Snow", "Midnight"]);
    for row in settings.choice_rows() {
        assert!(row.has_css_class("nn-settings-row"));
        assert!(row.activatable_widget().is_some());
        assert!(!row.title().is_empty());
        assert!(row.subtitle().is_some_and(|subtitle| !subtitle.is_empty()));
    }
}
