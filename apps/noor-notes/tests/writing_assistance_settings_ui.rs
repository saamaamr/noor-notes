use std::sync::Arc;

use adw::prelude::*;
use noor_domain::{EditorMode, WritingAssistanceOverrides};
use noor_notes::key_store::InMemoryKeyStore;
use noor_notes::ui::note_writing_assistance::NoteWritingAssistancePopover;
use noor_notes::ui::writing_assistance_settings::WritingAssistanceSettings;
use noor_notes::writing_assistance::{WritingAssistancePreferences, WritingAssistanceStore};

#[test]
fn settings_and_note_controls_are_safe_accessible_and_explicit() {
    gtk::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.SettingsTest")
        .build();
    let directory = tempfile::tempdir().unwrap();
    let store = WritingAssistanceStore::at(directory.path().join("writing.json"));
    let settings =
        WritingAssistanceSettings::new(&app, store, Arc::new(InMemoryKeyStore::default()));

    assert!(settings.spelling.is_active());
    assert!(settings.grammar.is_active());
    assert!(settings.offline_prediction.is_active());
    assert!(!settings.cloud.is_active());
    assert!(!settings.cloud.is_sensitive());
    assert!(settings.privacy_text().contains("current paragraph"));
    assert!(settings.privacy_text().contains("nearby sentence"));
    assert!(settings.spelling.is_focusable());

    let note = NoteWritingAssistancePopover::new(
        &WritingAssistancePreferences::default(),
        &WritingAssistanceOverrides::default(),
        EditorMode::Code,
    );
    assert!(note.text().contains("Checks comments and strings only"));
    assert!(!note.override_global.is_active());
    note.override_global.set_active(true);
    assert!(note.spelling.is_sensitive());
    let values = note.overrides();
    assert_eq!(values.spelling, Some(true));
    assert_eq!(values.grammar, Some(true));
    assert_eq!(values.offline_prediction, Some(true));
    assert_eq!(values.cloud, Some(false));
}
