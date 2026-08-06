use chrono::{Duration, Utc};
use noor_domain::{Note, NoteState};
use noor_notes::library::{LibrarySection, LibraryState, SearchGeneration};
use noor_storage::NoteSort;

fn note(title: &str, body: &str, age_minutes: i64) -> Note {
    let now = Utc::now() - Duration::minutes(age_minutes);
    let mut note = Note::new(now);
    note.title = title.into();
    note.content = body.into();
    note.updated_at = now;
    note
}

#[test]
fn projects_all_navigation_sections_and_counts() {
    let mut pinned = note("Pinned", "alpha", 2);
    pinned.pinned = true;
    pinned.tags = vec!["work".into(), "urgent".into(), "third".into()];
    let mut favorite = note("Favorite", "beta", 1);
    favorite.favorite = true;
    let mut archived = note("Archive", "gamma", 3);
    archived.state = NoteState::Archived;
    let mut trashed = note("Trash", "delta", 4);
    trashed.state = NoteState::Trashed {
        deleted_at: Utc::now(),
    };

    let state = LibraryState::new(vec![pinned.clone(), favorite.clone(), archived, trashed]);

    assert_eq!(state.count(LibrarySection::AllNotes), 2);
    assert_eq!(state.count(LibrarySection::Pinned), 1);
    assert_eq!(state.count(LibrarySection::Favorites), 1);
    assert_eq!(state.count(LibrarySection::Tags), 1);
    assert_eq!(state.count(LibrarySection::Archived), 1);
    assert_eq!(state.count(LibrarySection::Trash), 1);
    assert_eq!(state.count(LibrarySection::Recent), 2);

    let cards = state.project(LibrarySection::Pinned, "", NoteSort::UpdatedDesc);
    assert_eq!(cards[0].id, pinned.id);
    assert_eq!(cards[0].tags, vec!["work", "urgent"]);
    assert!(cards[0].pinned);
    assert!(!cards[0].favorite);
}

#[test]
fn filters_title_content_and_tags_case_insensitively() {
    let mut alpha = note("Project Aurora", "Draft roadmap", 1);
    alpha.tags = vec!["Planning".into()];
    let beta = note("Meeting", "Discuss AURORA launch", 2);
    let state = LibraryState::new(vec![alpha, beta]);

    assert_eq!(
        state
            .project(LibrarySection::AllNotes, "aurora", NoteSort::UpdatedDesc)
            .len(),
        2
    );
    assert_eq!(
        state
            .project(LibrarySection::AllNotes, "planning", NoteSort::UpdatedDesc)
            .len(),
        1
    );
}

#[test]
fn sorting_is_stable_and_deterministic() {
    let mut beta = note("beta", "", 0);
    let mut alpha = note("Alpha", "", 0);
    beta.updated_at = alpha.updated_at;
    alpha.created_at = beta.created_at;
    let state = LibraryState::new(vec![beta, alpha]);

    let ascending = state.project(LibrarySection::AllNotes, "", NoteSort::TitleAsc);
    assert_eq!(ascending[0].title, "Alpha");
    assert_eq!(ascending[1].title, "beta");
    let descending = state.project(LibrarySection::AllNotes, "", NoteSort::TitleDesc);
    assert_eq!(descending[0].title, "beta");
}

#[test]
fn search_generation_rejects_stale_results() {
    let mut generation = SearchGeneration::default();
    let first = generation.begin();
    let second = generation.begin();
    assert!(!generation.is_current(first));
    assert!(generation.is_current(second));
}

#[test]
fn section_labels_and_icons_are_accessible() {
    for section in LibrarySection::NAVIGATION {
        assert!(!section.label().is_empty());
        assert!(section.icon_name().ends_with("-symbolic"));
    }
}
