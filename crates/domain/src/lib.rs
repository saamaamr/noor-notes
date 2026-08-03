//! Core Noor Notes domain types.

mod note;
mod rich_text;
mod style;

pub use note::{Note, NoteId, NoteState, Revision};
pub use rich_text::{Alignment, ListKind, RichBlock, RichDocument, RichSpan, TextMarks};
pub use style::{NoteStyle, WindowGeometry};
