use noor_domain::{Note, NoteState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoteCounts {
    pub active: usize,
    pub archived: usize,
    pub trashed: usize,
}

impl NoteCounts {
    pub fn from_notes(notes: &[Note]) -> Self {
        let mut counts = Self::default();
        for note in notes {
            match note.state {
                NoteState::Active => counts.active += 1,
                NoteState::Archived => counts.archived += 1,
                NoteState::Trashed { .. } => counts.trashed += 1,
            }
        }
        counts
    }
}

pub fn content_preview(content: &str, max_characters: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return "Empty note".into();
    }
    let mut characters = normalized.chars();
    let preview: String = characters.by_ref().take(max_characters).collect();
    if characters.next().is_some() {
        format!("{}…", preview.trim_end())
    } else {
        preview
    }
}
