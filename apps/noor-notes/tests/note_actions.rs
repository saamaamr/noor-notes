use chrono::{TimeZone, Utc};
use noor_domain::{Note, NoteState, Revision};
use noor_notes::note_actions::{archive, trash};

#[test]
fn archive_preserves_content_and_advances_note_metadata() {
    let created = Utc.with_ymd_and_hms(2026, 8, 4, 10, 0, 0).unwrap();
    let changed = Utc.with_ymd_and_hms(2026, 8, 4, 11, 0, 0).unwrap();
    let mut note = Note::new(created);
    note.content = "Keep this note".into();

    archive(&mut note, changed);

    assert_eq!(note.state, NoteState::Archived);
    assert_eq!(note.content, "Keep this note");
    assert_eq!(note.revision, Revision::from_value(1));
    assert_eq!(note.updated_at, changed);
}

#[test]
fn trash_records_deletion_time_and_advances_note_metadata() {
    let created = Utc.with_ymd_and_hms(2026, 8, 4, 10, 0, 0).unwrap();
    let changed = Utc.with_ymd_and_hms(2026, 8, 4, 11, 30, 0).unwrap();
    let mut note = Note::new(created);
    note.content = "Do not erase content".into();

    trash(&mut note, changed);

    assert_eq!(
        note.state,
        NoteState::Trashed {
            deleted_at: changed
        }
    );
    assert_eq!(note.content, "Do not erase content");
    assert_eq!(note.revision, Revision::from_value(1));
    assert_eq!(note.updated_at, changed);
}
