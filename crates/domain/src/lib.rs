//! Core Noor Notes domain types.

mod note;
mod note_metadata;
mod rich_text;
mod style;

pub use note::{
    EditorMode, EditorPreferences, Note, NoteId, NoteState, Revision, SourceLanguage,
    WritingAssistanceOverrides,
};
pub use note_metadata::NoteColor;
pub use rich_text::{Alignment, ListKind, RichBlock, RichDocument, RichSpan, TextMarks};
pub use style::{NoteStyle, WindowGeometry};
