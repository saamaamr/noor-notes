use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use noor_domain::Note;
use sha2::{Digest, Sha256};

use crate::{ImportError, ImportIssue, ImportPreview, ImportableNote};

const MAX_INFO_FILES: usize = 10_000;
const MAX_INFO_BYTES: u64 = 64 * 1024;
const MAX_CONTENT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;

pub fn scan_xpad(path: &Path) -> Result<ImportPreview, ImportError> {
    let root = path.canonicalize()?;
    let mut info_files: Vec<PathBuf> = fs::read_dir(path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| {
            candidate
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("info-"))
        })
        .collect();
    if info_files.len() > MAX_INFO_FILES {
        return Err(ImportError::LimitExceeded("too many note files"));
    }
    info_files.sort();

    let mut preview = ImportPreview::default();
    let mut total_bytes = 0_u64;
    for info_path in info_files {
        match parse_note(&root, &info_path, &mut total_bytes) {
            Ok(note) => preview.importable.push(note),
            Err(error) => preview.skipped.push(ImportIssue {
                path: info_path,
                message: error.to_string(),
            }),
        }
    }
    Ok(preview)
}

fn parse_note(
    root: &Path,
    info_path: &Path,
    total_bytes: &mut u64,
) -> Result<ImportableNote, ImportError> {
    let info_metadata = fs::symlink_metadata(info_path)?;
    if !info_metadata.file_type().is_file() || info_metadata.file_type().is_symlink() {
        return Err(ImportError::UnsafeFileType(info_path.to_path_buf()));
    }
    if info_metadata.len() > MAX_INFO_BYTES {
        return Err(ImportError::LimitExceeded("note metadata is too large"));
    }
    let canonical_info = info_path.canonicalize()?;
    if !canonical_info.starts_with(root) {
        return Err(ImportError::UnsafeInfoPath(info_path.to_path_buf()));
    }
    let info_bytes = fs::read(info_path)?;
    let info = String::from_utf8(info_bytes.clone())?;
    let values: HashMap<&str, &str> = info
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .map(|(key, value)| (key, value.trim()))
        .collect();
    let content_name = required(&values, "content")?;
    if Path::new(content_name)
        .file_name()
        .and_then(|name| name.to_str())
        != Some(content_name)
    {
        return Err(ImportError::UnsafeContentPath(content_name.into()));
    }
    let content_path = root.join(content_name);
    let content_metadata = fs::symlink_metadata(&content_path)?;
    if !content_metadata.file_type().is_file() || content_metadata.file_type().is_symlink() {
        return Err(ImportError::UnsafeFileType(content_path));
    }
    if content_metadata.len() > MAX_CONTENT_BYTES {
        return Err(ImportError::LimitExceeded("note content is too large"));
    }
    let canonical_content = content_path.canonicalize()?;
    if !canonical_content.starts_with(root) {
        return Err(ImportError::UnsafeContentPath(content_name.into()));
    }
    *total_bytes = total_bytes
        .checked_add(info_metadata.len() + content_metadata.len())
        .ok_or(ImportError::LimitExceeded("import size overflow"))?;
    if *total_bytes > MAX_TOTAL_BYTES {
        return Err(ImportError::LimitExceeded("total import is too large"));
    }
    let content_bytes = fs::read(&content_path)?;
    let content = String::from_utf8(content_bytes.clone())?;
    let modified = fs::metadata(info_path)
        .and_then(|metadata| metadata.modified())
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(|_| Utc::now());
    let mut note = Note::new(modified);
    note.title = Note::derive_title(&content);
    note.content = content;
    note.geometry.width = parse_i32(&values, "width")?;
    note.geometry.height = parse_i32(&values, "height")?;
    note.geometry.x = optional_i32(&values, "x")?;
    note.geometry.y = optional_i32(&values, "y")?;
    if !(100..=8192).contains(&note.geometry.width)
        || !(100..=8192).contains(&note.geometry.height)
        || note
            .geometry
            .x
            .is_some_and(|value| !(-32768..=32767).contains(&value))
        || note
            .geometry
            .y
            .is_some_and(|value| !(-32768..=32767).contains(&value))
    {
        return Err(ImportError::InvalidGeometry);
    }
    note.always_on_top = values.get("sticky").is_some_and(|value| *value == "1");
    if let Some(background) = values.get("back") {
        note.style.background = (*background).into();
    }
    if let Some(foreground) = values.get("text") {
        note.style.foreground = (*foreground).into();
    }
    if let Some(font) = values.get("fontname") {
        note.style.font = (*font).into();
    }

    let mut hasher = Sha256::new();
    hasher.update(root.to_string_lossy().as_bytes());
    let info_name = info_path
        .file_name()
        .ok_or_else(|| ImportError::UnsafeInfoPath(info_path.to_path_buf()))?;
    hasher.update(info_name.as_encoded_bytes());
    hasher.update(&info_bytes);
    hasher.update(&content_bytes);
    let source_key = format!("xpad:{:x}", hasher.finalize());
    Ok(ImportableNote { source_key, note })
}

fn required<'a>(values: &'a HashMap<&str, &str>, key: &str) -> Result<&'a str, ImportError> {
    values
        .get(key)
        .copied()
        .ok_or_else(|| ImportError::MissingField(key.into()))
}

fn parse_i32(values: &HashMap<&str, &str>, key: &str) -> Result<i32, ImportError> {
    required(values, key)?
        .parse()
        .map_err(|_| ImportError::InvalidInteger(key.into()))
}

fn optional_i32(values: &HashMap<&str, &str>, key: &str) -> Result<Option<i32>, ImportError> {
    values
        .get(key)
        .map(|value| {
            value
                .parse()
                .map_err(|_| ImportError::InvalidInteger(key.into()))
        })
        .transpose()
}
