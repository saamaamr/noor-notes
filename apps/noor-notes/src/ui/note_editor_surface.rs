//! Shared editor-surface boundary used by the library preview and sticky view.
//!
//! The concrete implementation currently lives in `note_preview` so existing
//! preview tests and styling remain stable while both hosts migrate to the
//! same surface. Keeping this alias as the public boundary prevents hosts
//! from depending on the implementation module.

pub type NoteEditorSurface = super::note_preview::NotePreview;
