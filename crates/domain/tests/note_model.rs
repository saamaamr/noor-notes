use chrono::{TimeZone, Utc};
use noor_domain::{Note, NoteState, NoteStyle, WindowGeometry};

#[test]
fn new_note_has_safe_defaults() {
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();

    let note = Note::new(now);

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
