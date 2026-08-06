use noor_notes::library_preferences::LibraryPreferences;
use noor_storage::NoteSort;

#[test]
fn sort_preference_round_trips_and_invalid_values_fall_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences");
    let prefs = LibraryPreferences::at(path.clone());
    assert_eq!(prefs.load_sort(), NoteSort::UpdatedDesc);
    prefs.save_sort(NoteSort::TitleAsc).unwrap();
    assert_eq!(
        LibraryPreferences::at(path.clone()).load_sort(),
        NoteSort::TitleAsc
    );
    prefs.save_sort(NoteSort::TitleDesc).unwrap();
    assert_eq!(
        LibraryPreferences::at(path.clone()).load_sort(),
        NoteSort::TitleDesc
    );
    std::fs::write(path, "unknown").unwrap();
    assert_eq!(prefs.load_sort(), NoteSort::UpdatedDesc);
}
