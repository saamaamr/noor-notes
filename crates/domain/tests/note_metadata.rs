use chrono::{TimeZone, Utc};
use noor_domain::{EditorMode, EditorPreferences, Note, NoteColor, NoteState, SourceLanguage};

#[test]
fn metadata_defaults_and_legacy_json_are_safe() {
    let now = Utc.with_ymd_and_hms(2026, 8, 5, 10, 0, 0).unwrap();
    let note = Note::new(now);
    assert_eq!(note.color, NoteColor::Yellow);
    assert!(note.tags.is_empty());
    let mut value = serde_json::to_value(&note).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("color");
    object.remove("tags");
    object.remove("pinned");
    object.remove("favorite");
    object.remove("editor_preferences");
    object.remove("editor_mode");
    object.remove("source_language");
    let restored: Note = serde_json::from_value(value).unwrap();
    assert_eq!(restored.color, NoteColor::Yellow);
    assert!(restored.tags.is_empty());
    assert!(!restored.pinned);
    assert!(!restored.favorite);
    assert_eq!(restored.editor_preferences, EditorPreferences::default());
    assert_eq!(restored.editor_mode, EditorMode::Rich);
    assert_eq!(restored.source_language, SourceLanguage::Markdown);
}

#[test]
fn source_modes_validate_languages_and_round_trip() {
    let now = Utc.with_ymd_and_hms(2026, 8, 5, 10, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.editor_mode = EditorMode::Code;
    note.source_language = SourceLanguage::new("rust").unwrap();
    note.editor_preferences.cursor_offset = 42;
    note.editor_preferences.scroll_offset = 120;
    note.editor_preferences.bookmarks = vec![2, 8, 13];
    assert!(SourceLanguage::new("../../unsafe").is_none());

    let restored: Note = serde_json::from_str(&serde_json::to_string(&note).unwrap()).unwrap();
    assert_eq!(restored.editor_mode, EditorMode::Code);
    assert_eq!(restored.source_language.as_str(), "rust");
    assert_eq!(restored.editor_preferences.cursor_offset, 42);
    assert_eq!(restored.editor_preferences.scroll_offset, 120);
    assert_eq!(restored.editor_preferences.bookmarks, vec![2, 8, 13]);
}

#[test]
fn editor_preferences_are_clamped_and_round_trip_with_note_metadata() {
    let now = Utc.with_ymd_and_hms(2026, 8, 5, 10, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.pinned = true;
    note.favorite = true;
    note.editor_preferences.word_wrap = false;
    note.editor_preferences.set_zoom_percent(425);

    assert_eq!(note.editor_preferences.zoom_percent, 300);

    let restored: Note = serde_json::from_str(&serde_json::to_string(&note).unwrap()).unwrap();
    assert!(restored.pinned);
    assert!(restored.favorite);
    assert!(!restored.editor_preferences.word_wrap);
    assert_eq!(restored.editor_preferences.zoom_percent, 300);
}

#[test]
fn tags_are_trimmed_and_deduplicated_case_insensitively() {
    let now = Utc.with_ymd_and_hms(2026, 8, 5, 10, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.set_tags(vec![
        " Work ".into(),
        "work".into(),
        "বাংলা".into(),
        "".into(),
    ]);
    assert_eq!(note.tags, vec!["Work", "বাংলা"]);
}

#[test]
fn duplicate_is_fresh_active_note_with_copied_content_and_style() {
    let created = Utc.with_ymd_and_hms(2026, 8, 5, 10, 0, 0).unwrap();
    let duplicated_at = Utc.with_ymd_and_hms(2026, 8, 5, 11, 0, 0).unwrap();
    let mut note = Note::new(created);
    note.title = "Plan".into();
    note.content = "Body".into();
    note.color = NoteColor::Blue;
    note.set_tags(vec!["work".into()]);
    note.state = NoteState::Archived;
    let copy = note.duplicate(duplicated_at);
    assert_ne!(copy.id, note.id);
    assert_eq!(copy.title, "Plan copy");
    assert_eq!(copy.content, note.content);
    assert_eq!(copy.color, NoteColor::Blue);
    assert_eq!(copy.tags, note.tags);
    assert_eq!(copy.state, NoteState::Active);
    assert_eq!(copy.created_at, duplicated_at);
    assert_eq!(copy.updated_at, duplicated_at);
    assert_eq!(copy.revision.value(), 0);
}
