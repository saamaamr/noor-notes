use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_notes::library::LibrarySection;
use noor_notes::search::search_notes;
use noor_notes::ui::library_window::library_result_summary;
use noor_storage::SqliteNoteRepository;

#[tokio::test]
async fn application_search_handles_latin_arabic_and_bangla() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    for content in ["Release PLAN", "বাংলা পরীক্ষা", "اختبار عربي"]
    {
        let mut note = Note::new(now);
        note.content = content.into();
        repo.save_note(&note).await.unwrap();
    }

    assert_eq!(search_notes(&repo, "plan").await.unwrap().len(), 1);
    assert_eq!(search_notes(&repo, "বাংলা").await.unwrap().len(), 1);
    assert_eq!(search_notes(&repo, "عربي").await.unwrap().len(), 1);
}

#[test]
fn library_result_feedback_is_quiet_and_grammatical() {
    assert_eq!(
        library_result_summary(LibrarySection::AllNotes, 0, true),
        "No results"
    );
    assert_eq!(
        library_result_summary(LibrarySection::AllNotes, 1, true),
        "1 result"
    );
    assert_eq!(
        library_result_summary(LibrarySection::AllNotes, 3, true),
        "3 results"
    );
    assert_eq!(
        library_result_summary(LibrarySection::Pinned, 2, false),
        "Pinned · 2 notes"
    );
}
