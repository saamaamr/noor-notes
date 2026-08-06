use adw::prelude::*;
use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore, EffectiveTheme};

#[test]
fn manager_updates_windows_cycles_palettes_and_persists_the_active_mode() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let store = AppearanceStore::at(directory.path().join("appearance.json"));
    let manager = AppearanceManager::new(store.clone());
    let first = gtk::Window::new();
    let second = gtk::Window::new();
    manager.register_window(&first);
    manager.register_window(&second);

    manager.set_mode(AppearanceMode::Midnight).unwrap();
    assert_eq!(manager.effective_theme(), EffectiveTheme::Midnight);
    for window in [&first, &second] {
        assert!(window.has_css_class("nn-theme-midnight"));
        assert_eq!(
            EffectiveTheme::ALL_CLASSES
                .iter()
                .filter(|class| window.has_css_class(class))
                .count(),
            1
        );
    }

    manager.set_mode(AppearanceMode::Oled).unwrap();
    assert!(first.has_css_class("nn-theme-oled"));
    assert!(!first.has_css_class("nn-theme-midnight"));
    assert_eq!(
        manager.cycle_dark_palette().unwrap(),
        EffectiveTheme::Graphite
    );
    assert_eq!(manager.active_label(), "Graphite");
    assert_eq!(
        manager.cycle_dark_palette().unwrap(),
        EffectiveTheme::Midnight
    );
    assert_eq!(manager.cycle_dark_palette().unwrap(), EffectiveTheme::Oled);
    assert_eq!(store.load().mode, AppearanceMode::Oled);
    let blocker = directory.path().join("not-a-directory");
    std::fs::write(&blocker, "blocked").unwrap();
    let failing = AppearanceManager::new(AppearanceStore::at(blocker.join("appearance.json")));
    let failure_window = gtk::Window::new();
    failing.register_window(&failure_window);
    assert!(failing.set_mode(AppearanceMode::Oled).is_err());
    assert_eq!(failing.effective_theme(), EffectiveTheme::Oled);
    assert!(failure_window.has_css_class("nn-theme-oled"));
}
