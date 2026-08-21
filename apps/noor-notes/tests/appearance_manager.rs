use adw::prelude::*;
use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore, EffectiveTheme};

#[test]
fn manager_updates_every_window_toggles_and_persists_two_themes() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let store = AppearanceStore::at(directory.path().join("appearance.json"));
    let manager = AppearanceManager::new(store.clone());
    let windows = [
        gtk::Window::new(),
        gtk::Window::new(),
        gtk::Window::new(),
        gtk::Window::new(),
    ];
    for window in &windows {
        manager.register_window(window);
    }

    manager.set_mode(AppearanceMode::Midnight).unwrap();
    assert_eq!(manager.effective_theme(), EffectiveTheme::Midnight);
    assert_eq!(manager.active_label(), "Midnight");
    for window in &windows {
        assert!(window.has_css_class("nn-theme-midnight"));
        assert_eq!(
            EffectiveTheme::ALL_CLASSES
                .iter()
                .filter(|class| window.has_css_class(class))
                .count(),
            1
        );
    }

    assert_eq!(manager.toggle_theme().unwrap(), EffectiveTheme::Snow);
    assert_eq!(manager.active_label(), "Snow");
    assert_eq!(store.load().mode, AppearanceMode::Snow);
    for window in &windows {
        assert!(window.has_css_class("nn-theme-snow"));
        assert!(!window.has_css_class("nn-theme-midnight"));
    }

    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.AppearanceTest")
        .build();
    manager.install_action(&app);
    assert_eq!(
        adw::StyleManager::default().color_scheme(),
        adw::ColorScheme::ForceLight
    );

    let blocker = directory.path().join("not-a-directory");
    std::fs::write(&blocker, "blocked").unwrap();
    let failing = AppearanceManager::new(AppearanceStore::at(blocker.join("appearance.json")));
    let failure_window = gtk::Window::new();
    failing.register_window(&failure_window);
    assert!(failing.set_mode(AppearanceMode::Midnight).is_err());
    assert_eq!(failing.effective_theme(), EffectiveTheme::Midnight);
    assert!(failure_window.has_css_class("nn-theme-midnight"));
}
