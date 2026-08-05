use chrono::{TimeZone, Utc};
use noor_domain::{Note, NoteState, NoteStyle, WindowGeometry};

#[test]
fn new_note_has_safe_defaults() {
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();

    let note = Note::new(now);

    assert_eq!(note.title, "Untitled note");
    assert!(note.content.is_empty());
    assert_eq!(note.style.opacity, 1.0);
    assert!(!note.always_on_top);
    assert!(!note.all_workspaces);
    assert_eq!(note.state, NoteState::Active);
    assert_eq!(note.geometry, WindowGeometry::default());
    assert_eq!(note.created_at, now);
    assert_eq!(note.updated_at, now);
    assert_eq!(note.revision.value(), 0);
}

#[test]
fn legacy_note_json_derives_title_without_changing_content() {
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.content = "\n  First useful line  \nBody stays here".into();
    let mut value = serde_json::to_value(&note).unwrap();
    value.as_object_mut().unwrap().remove("title");

    let restored: Note = serde_json::from_value(value).unwrap();

    assert_eq!(restored.title, "First useful line");
    assert_eq!(restored.content, note.content);
    assert_eq!(restored.display_title(), "First useful line");
}

#[test]
fn blank_titles_display_as_untitled_and_derivation_is_unicode_safe() {
    assert_eq!(Note::derive_title(" \n\t"), "Untitled note");
    let long = "আ".repeat(90);
    assert_eq!(Note::derive_title(&long).chars().count(), 80);
}

#[test]
fn opacity_is_clamped_to_the_supported_range() {
    let mut style = NoteStyle::default();

    style.set_opacity(1.7);
    assert_eq!(style.opacity, 1.0);

    style.set_opacity(0.1);
    assert_eq!(style.opacity, 0.35);
}

#[test]
fn window_geometry_defaults_to_a_useful_sticky_note_size() {
    let geometry = WindowGeometry::default();

    assert_eq!(geometry.width, 360);
    assert_eq!(geometry.height, 320);
}
