use std::cell::RefCell;
use std::rc::Rc;

use chrono::Utc;
use gtk::prelude::*;
use noor_domain::{Note, NoteId};
use noor_storage::{SqliteNoteRepository, StorageError};

use crate::autosave::AutosaveQueue;
use crate::note_actions;
use crate::ui::dialog_primitives;

pub async fn confirm_move_to_trash(parent: &impl IsA<gtk::Widget>) -> bool {
    dialog_primitives::confirm_destructive(
        parent,
        "Move this note to Trash?",
        "The note will remain recoverable from the Trash section.",
        "Move to Trash",
    )
    .await
}

pub async fn trash_open_note(
    note: &Rc<RefCell<Note>>,
    autosave: &AutosaveQueue,
    repository: &SqliteNoteRepository,
) -> Result<(), StorageError> {
    let id = note.borrow().id;
    autosave.flush(id).await?;
    let mut changed = note.borrow().clone();
    note_actions::trash(&mut changed, Utc::now());
    repository.save_note(&changed).await?;
    note.replace(changed);
    Ok(())
}

pub async fn trash_saved_note(
    repository: &SqliteNoteRepository,
    id: NoteId,
) -> Result<(), StorageError> {
    repository.trash(id, Utc::now()).await
}
