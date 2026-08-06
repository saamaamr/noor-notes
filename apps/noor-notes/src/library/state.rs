use noor_domain::{Note, NoteState};
use noor_storage::NoteSort;

use super::{LibrarySection, NoteListItem};

#[derive(Clone, Debug, Default)]
pub struct LibraryState {
    notes: Vec<Note>,
}

impl LibraryState {
    pub fn new(notes: Vec<Note>) -> Self {
        Self { notes }
    }

    pub fn replace(&mut self, notes: Vec<Note>) {
        self.notes = notes;
    }

    pub fn count(&self, section: LibrarySection) -> usize {
        self.notes
            .iter()
            .filter(|note| section_matches(note, section))
            .count()
    }

    pub fn project(
        &self,
        section: LibrarySection,
        query: &str,
        sort: NoteSort,
    ) -> Vec<NoteListItem> {
        let query = query.trim().to_lowercase();
        let mut notes: Vec<&Note> = self
            .notes
            .iter()
            .filter(|note| section_matches(note, section))
            .filter(|note| query.is_empty() || search_matches(note, &query))
            .collect();
        match sort {
            NoteSort::UpdatedDesc => notes.sort_by(|left, right| {
                right
                    .updated_at
                    .cmp(&left.updated_at)
                    .then_with(|| left.id.value().cmp(&right.id.value()))
            }),
            NoteSort::TitleAsc => notes.sort_by(|left, right| {
                left.display_title()
                    .to_lowercase()
                    .cmp(&right.display_title().to_lowercase())
                    .then_with(|| left.id.value().cmp(&right.id.value()))
            }),
            NoteSort::TitleDesc => notes.sort_by(|left, right| {
                right
                    .display_title()
                    .to_lowercase()
                    .cmp(&left.display_title().to_lowercase())
                    .then_with(|| left.id.value().cmp(&right.id.value()))
            }),
            NoteSort::CreatedDesc => notes.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.value().cmp(&right.id.value()))
            }),
        }
        notes.into_iter().map(NoteListItem::from).collect()
    }
}

fn section_matches(note: &Note, section: LibrarySection) -> bool {
    match section {
        LibrarySection::AllNotes | LibrarySection::Recent => {
            matches!(note.state, NoteState::Active)
        }
        LibrarySection::Pinned => matches!(note.state, NoteState::Active) && note.pinned,
        LibrarySection::Favorites => matches!(note.state, NoteState::Active) && note.favorite,
        LibrarySection::Tags => matches!(note.state, NoteState::Active) && !note.tags.is_empty(),
        LibrarySection::Archived => matches!(note.state, NoteState::Archived),
        LibrarySection::Trash => matches!(note.state, NoteState::Trashed { .. }),
    }
}

fn search_matches(note: &Note, query: &str) -> bool {
    note.title.to_lowercase().contains(query)
        || note.content.to_lowercase().contains(query)
        || note
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(query))
}
