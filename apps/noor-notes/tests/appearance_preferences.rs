use std::os::unix::fs::PermissionsExt;

use noor_notes::appearance::{
    AppearanceMode, AppearancePreferences, AppearanceStore, EffectiveTheme, SystemScheme,
};

#[test]
fn defaults_follow_the_system_with_two_effective_themes() {
    let preferences = AppearancePreferences::default();
    assert_eq!(preferences.mode, AppearanceMode::System);
    assert_eq!(
        preferences.resolve(SystemScheme::Light),
        EffectiveTheme::Snow
    );
    assert_eq!(
        preferences.resolve(SystemScheme::Dark),
        EffectiveTheme::Midnight
    );
}

#[test]
fn explicit_modes_resolve_without_consulting_the_desktop() {
    for (mode, effective) in [
        (AppearanceMode::Snow, EffectiveTheme::Snow),
        (AppearanceMode::Midnight, EffectiveTheme::Midnight),
    ] {
        let preferences = AppearancePreferences { mode };
        assert_eq!(preferences.resolve(SystemScheme::Light), effective);
        assert_eq!(preferences.resolve(SystemScheme::Dark), effective);
    }
}

#[test]
fn historical_modes_load_as_two_canonical_themes() {
    for (stored, mode, light, dark) in [
        (
            "light",
            AppearanceMode::Snow,
            EffectiveTheme::Snow,
            EffectiveTheme::Snow,
        ),
        (
            "warm-paper",
            AppearanceMode::Snow,
            EffectiveTheme::Snow,
            EffectiveTheme::Snow,
        ),
        (
            "cool-mist",
            AppearanceMode::Snow,
            EffectiveTheme::Snow,
            EffectiveTheme::Snow,
        ),
        (
            "graphite",
            AppearanceMode::Midnight,
            EffectiveTheme::Midnight,
            EffectiveTheme::Midnight,
        ),
        (
            "midnight",
            AppearanceMode::Midnight,
            EffectiveTheme::Midnight,
            EffectiveTheme::Midnight,
        ),
        (
            "oled",
            AppearanceMode::Midnight,
            EffectiveTheme::Midnight,
            EffectiveTheme::Midnight,
        ),
        (
            "system",
            AppearanceMode::System,
            EffectiveTheme::Snow,
            EffectiveTheme::Midnight,
        ),
    ] {
        let value = format!(
            r#"{{"mode":"{stored}","preferred_light":"warm-paper","preferred_dark":"oled"}}"#
        );
        let preferences: AppearancePreferences = serde_json::from_str(&value).unwrap();
        assert_eq!(preferences.mode, mode);
        assert_eq!(preferences.resolve(SystemScheme::Light), light);
        assert_eq!(preferences.resolve(SystemScheme::Dark), dark);
    }
}

#[test]
fn preferences_round_trip_with_canonical_values_and_private_permissions() {
    for (mode, expected) in [
        (AppearanceMode::Snow, "{\"mode\":\"snow\"}\n"),
        (AppearanceMode::Midnight, "{\"mode\":\"midnight\"}\n"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("appearance.json");
        let store = AppearanceStore::at(path.clone());
        let preferences = AppearancePreferences { mode };

        store.save(&preferences).unwrap();

        assert_eq!(store.load(), preferences);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
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
fn appearance_actions_accept_canonical_and_historical_names() {
    for value in ["snow", "light", "warm-paper", "cool-mist"] {
        assert_eq!(
            AppearanceMode::from_action_name(value),
            Some(AppearanceMode::Snow)
        );
    }
    for value in ["midnight", "graphite", "oled"] {
        assert_eq!(
            AppearanceMode::from_action_name(value),
            Some(AppearanceMode::Midnight)
        );
    }
    assert_eq!(
        AppearanceMode::from_action_name("system"),
        Some(AppearanceMode::System)
    );
    assert_eq!(AppearanceMode::from_action_name("unknown"), None);
}
