use chrono::{Duration, Utc};
use noor_domain::Note;
use noor_notes::library::{LibrarySection, LibraryState};
use noor_storage::NoteSort;
use std::time::Instant;

#[test]
fn filters_five_thousand_notes_with_bounded_projection() {
    let now = Utc::now();
    let notes = (0..5_000)
        .map(|index| {
            let mut note = Note::new(now - Duration::seconds(index));
            note.title = format!("Research note {index:04}");
            note.content = if index % 10 == 0 {
                "Unicode বাংলা searchable needle".into()
            } else {
                "ordinary content".into()
            };
            note
        })
        .collect();
    let state = LibraryState::new(notes);

    let started = Instant::now();
    let result = state.project(LibrarySection::AllNotes, "বাংলা", NoteSort::UpdatedDesc);

    assert_eq!(result.len(), 500);
    assert_eq!(result[0].title, "Research note 0000");
    assert!(started.elapsed().as_secs_f32() < 2.0);
}
