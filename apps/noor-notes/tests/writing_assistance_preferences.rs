use std::os::unix::fs::PermissionsExt;

use noor_domain::WritingAssistanceOverrides;
use noor_notes::writing_assistance::{
    ProviderConfiguration, WritingAssistancePreferences, WritingAssistanceStore,
    provider_requires_api_key, validate_provider_endpoint,
};

#[test]
fn local_features_are_on_and_cloud_is_off_by_default() {
    let value = WritingAssistancePreferences::default();

    assert!(value.spelling);
    assert!(value.grammar);
    assert!(value.offline_prediction);
    assert!(!value.cloud_enabled);
    assert_eq!(value.language, "auto");
}

#[test]
fn per_note_values_override_only_selected_global_values() {
    let global = WritingAssistancePreferences::default();
    let overrides = WritingAssistanceOverrides {
        grammar: Some(false),
        cloud: Some(true),
        ..Default::default()
    };

    let effective = global.resolve(&overrides);

    assert!(effective.spelling);
    assert!(!effective.grammar);
    assert!(effective.offline_prediction);
    assert!(!effective.cloud);
}

#[test]
fn endpoint_policy_rejects_remote_http_and_accepts_loopback_http() {
    assert!(validate_provider_endpoint("http://example.com").is_err());
    assert!(validate_provider_endpoint("http://localhost:11434").is_ok());
    assert!(validate_provider_endpoint("http://127.0.0.1:8080/v1").is_ok());
    assert!(validate_provider_endpoint("http://[::1]:8080").is_ok());
    assert!(validate_provider_endpoint("https://api.example.com").is_ok());
    assert!(validate_provider_endpoint("https://user:secret@example.com").is_err());
    assert!(provider_requires_api_key(
        &validate_provider_endpoint("https://api.example.com").unwrap()
    ));
    assert!(!provider_requires_api_key(
        &validate_provider_endpoint("http://localhost:11434").unwrap()
    ));
}

#[test]
fn provider_edits_revoke_connection_validation_and_cloud() {
    let mut preferences = WritingAssistancePreferences::default();
    preferences.update_provider("https://api.example.com", "model-a");
    preferences.mark_provider_validated();
    preferences.cloud_enabled = true;
    assert!(preferences.provider.is_validated());
    assert!(preferences.resolve(&Default::default()).cloud);

    preferences.update_provider("https://api.example.com", "model-b");

    assert!(!preferences.provider.is_validated());
    assert!(!preferences.cloud_enabled);
    assert!(!preferences.resolve(&Default::default()).cloud);
}

#[test]
fn preferences_round_trip_atomically_with_private_permissions() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("writing-assistance.json");
    let store = WritingAssistanceStore::at(path.clone());
    let preferences = WritingAssistancePreferences {
        spelling: false,
        grammar: true,
        offline_prediction: false,
        cloud_enabled: false,
        language: "en_US".into(),
        provider: ProviderConfiguration::default(),
    };

    store.save(&preferences).unwrap();

    assert_eq!(store.load(), preferences);
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn malformed_or_unsafe_preferences_fail_closed_without_destroying_the_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("writing-assistance.json");
    std::fs::write(&path, "{not valid").unwrap();
    let store = WritingAssistanceStore::at(path.clone());

    assert_eq!(store.load(), WritingAssistancePreferences::default());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{not valid");

    std::fs::write(
        &path,
        r#"{"spelling":true,"grammar":true,"offline_prediction":true,"cloud_enabled":true,"language":"auto","provider":{"base_url":"http://example.com","model":"x","provider_validated":true,"validated_base_url":"http://example.com","validated_model":"x"}}"#,
    )
    .unwrap();

    assert_eq!(store.load(), WritingAssistancePreferences::default());
}
