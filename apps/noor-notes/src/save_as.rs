use std::path::{Path, PathBuf};

use adw::prelude::*;
use noor_domain::Note;
use thiserror::Error;

use crate::export::{ExportDocument, ExportError, ExportFormat, render_export};
use crate::safe_export::{ensure_export_extension, sanitize_export_name, set_owner_only};

#[derive(Debug, Error)]
pub enum SaveAsError {
    #[error("the selected destination is not a local file")]
    NonLocalDestination,
    #[error("the filename must end with .{0}; choose Save As again with that extension")]
    WrongExtension(&'static str),
    #[error(transparent)]
    Render(#[from] ExportError),
    #[error("the export worker stopped unexpectedly")]
    WorkerStopped,
    #[error("could not write the exported file: {0}")]
    Write(String),
    #[error("the file was written, but its private permissions could not be applied: {0}")]
    Permissions(String),
}

pub fn validate_export_path(path: &Path, format: ExportFormat) -> Result<PathBuf, SaveAsError> {
    let enforced_path = ensure_export_extension(path, format.extension());
    if enforced_path != path {
        return Err(SaveAsError::WrongExtension(format.extension().as_str()));
    }
    Ok(path.to_path_buf())
}

/// Opens one format-specific save dialog, renders away from the GTK thread,
/// and writes an unencrypted owner-only copy without mutating the note.
pub async fn save_note_as(
    parent: &gtk::Window,
    note: Note,
    format: ExportFormat,
) -> Result<Option<PathBuf>, SaveAsError> {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(&format!(
        "{} (.{} file)",
        format.label(),
        format.extension().as_str()
    )));
    filter.add_mime_type(format.mime_type());
    filter.add_suffix(format.extension().as_str());
    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::builder()
        .title(format!("Save unencrypted {} copy", format.label()))
        .accept_label("Save Copy")
        .initial_name(sanitize_export_name(
            note.display_title(),
            format.extension(),
        ))
        .filters(&filters)
        .default_filter(&filter)
        .modal(true)
        .build();

    let selected = match dialog.save_future(Some(parent)).await {
        Ok(file) => file,
        Err(error) if error.matches::<gtk::gio::IOErrorEnum>(gtk::gio::IOErrorEnum::Cancelled) => {
            return Ok(None);
        }
        Err(error) => return Err(SaveAsError::Write(error.to_string())),
    };
    let path = selected.path().ok_or(SaveAsError::NonLocalDestination)?;
    let path = validate_export_path(&path, format)?;
    let destination = gtk::gio::File::for_path(&path);

    let document = ExportDocument::from_note(&note);
    let bytes = gtk::gio::spawn_blocking(move || render_export(&document, format))
        .await
        .map_err(|_| SaveAsError::WorkerStopped)??;
    destination
        .replace_contents_future(
            bytes,
            None,
            false,
            gtk::gio::FileCreateFlags::REPLACE_DESTINATION,
        )
        .await
        .map_err(|(_, error)| SaveAsError::Write(error.to_string()))?;

    let permission_path = path.clone();
    gtk::gio::spawn_blocking(move || set_owner_only(&permission_path))
        .await
        .map_err(|_| SaveAsError::WorkerStopped)?
        .map_err(|error| SaveAsError::Permissions(error.to_string()))?;
    Ok(Some(path))
}
