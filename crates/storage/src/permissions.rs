use std::path::Path;

use crate::StorageError;

pub(crate) fn prepare_database_path(path: &Path) -> Result<(), StorageError> {
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(StorageError::UnsafeDatabasePath(path.to_path_buf()));
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| StorageError::PrepareDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
        secure(parent, 0o700)?;
    }
    Ok(())
}

pub(crate) fn secure_data_tree(path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        secure(parent, 0o700)?;
    }
    for candidate in [
        path.to_path_buf(),
        path.with_extension("db-wal"),
        path.with_extension("db-shm"),
    ] {
        if candidate.exists() {
            secure(&candidate, 0o600)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn secure(path: &Path, mode: u32) -> Result<(), StorageError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|source| {
        StorageError::SecureFile {
            path: path.to_path_buf(),
            source,
        }
    })
}

#[cfg(not(unix))]
fn secure(_path: &Path, _mode: u32) -> Result<(), StorageError> {
    Ok(())
}
