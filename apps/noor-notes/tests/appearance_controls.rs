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
    manager.set_mode(AppearanceMode::Light).unwrap();

    let control = AppearanceButton::new(manager.clone());
    assert_eq!(
        control.button.tooltip_text().as_deref(),
        Some("Light palette: Snow. Click for Warm Paper")
    );
    assert!(control.button.has_css_class("nn-icon-active"));
    control.button.emit_clicked();
    assert_eq!(manager.preferences().mode, AppearanceMode::WarmPaper);
    assert_eq!(
        control.button.tooltip_text().as_deref(),
        Some("Light palette: Warm Paper. Click for Cool Mist")
    );

    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.AppearanceTest")
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let settings = AppearanceSettings::new(&app, manager);
    assert_eq!(settings.choice_count(), 7);
    assert_eq!(settings.window.title().as_deref(), Some("Appearance"));
    assert!(settings.window.has_css_class("nn-settings-window"));
    for row in settings.choice_rows() {
        assert!(row.has_css_class("nn-settings-row"));
        assert!(row.activatable_widget().is_some());
        assert!(!row.title().is_empty());
        assert!(row.subtitle().is_some_and(|subtitle| !subtitle.is_empty()));
    }
}
