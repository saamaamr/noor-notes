//! Core Noor Notes domain types.

mod note;
mod note_metadata;
mod rich_text;
mod style;

pub use note::{EditorPreferences, Note, NoteId, NoteState, Revision};
pub use note_metadata::NoteColor;
pub use rich_text::{Alignment, ListKind, RichBlock, RichDocument, RichSpan, TextMarks};
pub use style::{NoteStyle, WindowGeometry};
