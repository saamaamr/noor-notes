use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::StorageError;

pub(crate) fn preserve_corrupt_database(path: &Path) -> Result<Option<PathBuf>, StorageError> {
    if !path.is_file() {
        return Ok(None);
    }

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "notes.db".into());
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S%6fZ");
    let backup = path.with_file_name(format!("{file_name}.corrupt-{timestamp}.bak"));
    std::fs::copy(path, &backup).map_err(|source| StorageError::BackupCorruptDatabase {
        path: path.to_path_buf(),
        backup: backup.clone(),
        source,
    })?;
    Ok(Some(backup))
}
