use noor_domain::NoteId;
use noor_notes::services::{EditorSessionRecord, RecentItems, SessionStore};

#[test]
fn session_restore_deduplicates_and_discards_missing_notes() {
    let directory = tempfile::tempdir().unwrap();
    let store = SessionStore::at(directory.path().join("session.json"));
    let first = NoteId::new();
    let second = NoteId::new();
    let missing = NoteId::new();
    store
        .save(&EditorSessionRecord {
            open_notes: vec![first, second, first, missing],
            active_note: Some(missing),
        })
        .unwrap();
    let restored = store.load_valid(&[first, second]).unwrap();
    assert_eq!(restored.open_notes, vec![first, second]);
    assert_eq!(restored.active_note, Some(first));
}

#[test]
fn invalid_session_file_fails_closed_without_destroying_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.json");
    std::fs::write(&path, "{broken").unwrap();
    let store = SessionStore::at(path.clone());
    assert!(store.load_valid(&[]).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{broken");
}

#[test]
fn recent_items_are_unique_and_most_recent_first() {
    let first = NoteId::new();
    let second = NoteId::new();
    let mut recent = RecentItems::with_limit(3);
    recent.touch(first);
    recent.touch(second);
    recent.touch(first);
    assert_eq!(recent.items(), &[first, second]);
}
