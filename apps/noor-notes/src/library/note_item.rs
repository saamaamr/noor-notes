use chrono::{DateTime, Utc};
use noor_domain::{Note, NoteColor, NoteId};

use crate::library_view::content_preview;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteListItem {
    pub id: NoteId,
    pub title: String,
    pub preview: String,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub favorite: bool,
    pub color: NoteColor,
}

impl From<&Note> for NoteListItem {
    fn from(note: &Note) -> Self {
        Self {
            id: note.id,
            title: note.display_title().to_owned(),
            preview: content_preview(&note.content, 180),
            updated_at: note.updated_at,
            tags: note.tags.iter().take(2).cloned().collect(),
            pinned: note.pinned,
            favorite: note.favorite,
            color: note.color,
        }
    }
}
