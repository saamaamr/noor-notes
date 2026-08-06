use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{Note, NoteId};
use noor_storage::{SqliteNoteRepository, StorageError};

use crate::autosave::AutosaveQueue;
use crate::note_actions;

pub async fn confirm_move_to_trash(parent: &impl IsA<gtk::Widget>) -> bool {
    let dialog = adw::AlertDialog::new(
        Some("Move this note to Trash?"),
        Some("The note will remain recoverable from the Trash section."),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("trash", "Move to Trash");
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("trash", adw::ResponseAppearance::Destructive);
    dialog.choose_future(Some(parent)).await == "trash"
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
