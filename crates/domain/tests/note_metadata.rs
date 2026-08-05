use chrono::{TimeZone, Utc};
use noor_domain::{Note, NoteColor, NoteState};

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
    let restored: Note = serde_json::from_value(value).unwrap();
    assert_eq!(restored.color, NoteColor::Yellow);
    assert!(restored.tags.is_empty());
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
