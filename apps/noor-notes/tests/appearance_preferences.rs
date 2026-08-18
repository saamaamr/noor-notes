use std::os::unix::fs::PermissionsExt;

use noor_notes::appearance::{
    AppearanceMode, AppearancePreferences, AppearanceStore, DarkPalette, EffectiveTheme,
    LightPalette, SystemScheme,
};

#[test]
fn defaults_and_system_resolution_use_graphite_for_dark_desktops() {
    let preferences = AppearancePreferences::default();
    assert_eq!(preferences.mode, AppearanceMode::System);
    assert_eq!(preferences.preferred_light, LightPalette::Snow);
    assert_eq!(preferences.preferred_dark, DarkPalette::Graphite);
    assert_eq!(
        preferences.resolve(SystemScheme::Light),
        EffectiveTheme::Light
    );
    assert_eq!(
        preferences.resolve(SystemScheme::Dark),
        EffectiveTheme::Graphite
    );
}

#[test]
fn explicit_modes_resolve_without_consulting_the_desktop() {
    for (mode, effective) in [
        (AppearanceMode::Light, EffectiveTheme::Light),
        (AppearanceMode::WarmPaper, EffectiveTheme::WarmPaper),
        (AppearanceMode::CoolMist, EffectiveTheme::CoolMist),
        (AppearanceMode::Graphite, EffectiveTheme::Graphite),
        (AppearanceMode::Midnight, EffectiveTheme::Midnight),
        (AppearanceMode::Oled, EffectiveTheme::Oled),
    ] {
        let preferences = AppearancePreferences {
            mode,
            preferred_light: LightPalette::CoolMist,
            preferred_dark: DarkPalette::Midnight,
        };
        assert_eq!(preferences.resolve(SystemScheme::Light), effective);
        assert_eq!(preferences.resolve(SystemScheme::Dark), effective);
    }
}

#[test]
fn preferences_round_trip_atomically_with_stable_values_and_private_permissions() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("appearance.json");
    let store = AppearanceStore::at(path.clone());
    let preferences = AppearancePreferences {
        mode: AppearanceMode::CoolMist,
        preferred_light: LightPalette::CoolMist,
        preferred_dark: DarkPalette::Midnight,
    };

    store.save(&preferences).unwrap();

    assert_eq!(store.load(), preferences);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains(r#""mode":"cool-mist""#));
    assert!(raw.contains(r#""preferred_light":"cool-mist""#));
    assert!(raw.contains(r#""preferred_dark":"midnight""#));
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn system_light_uses_the_remembered_light_palette_and_legacy_files_default_to_snow() {
    let preferences = AppearancePreferences {
        mode: AppearanceMode::System,
        preferred_light: LightPalette::WarmPaper,
        preferred_dark: DarkPalette::Graphite,
    };
    assert_eq!(
        preferences.resolve(SystemScheme::Light),
        EffectiveTheme::WarmPaper
    );

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("appearance.json");
    std::fs::write(&path, r#"{"mode":"light","preferred_dark":"midnight"}"#).unwrap();
    let loaded = AppearanceStore::at(path).load();
    assert_eq!(loaded.preferred_light, LightPalette::Snow);
    assert_eq!(loaded.resolve(SystemScheme::Light), EffectiveTheme::Light);
}

#[test]
fn invalid_preferences_fail_closed_without_destroying_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("appearance.json");
    std::fs::write(&path, "{not valid").unwrap();
    let store = AppearanceStore::at(path.clone());

    assert_eq!(store.load(), AppearancePreferences::default());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{not valid");
}

#[test]
fn warm_paper_and_cool_mist_are_selectable_appearance_actions() {
    assert!(AppearanceMode::from_action_name("warm-paper").is_some());
    assert!(AppearanceMode::from_action_name("cool-mist").is_some());
}
