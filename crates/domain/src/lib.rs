//! Core Noor Notes domain types.

mod note;
mod style;

pub use note::{Note, NoteId, NoteState, Revision};
pub use style::{NoteStyle, WindowGeometry};
