use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use noor_domain::Note;
use sha2::{Digest, Sha256};

use crate::{ImportError, ImportIssue, ImportPreview, ImportableNote};

pub fn scan_xpad(path: &Path) -> Result<ImportPreview, ImportError> {
    let mut info_files: Vec<PathBuf> = fs::read_dir(path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| {
            candidate
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("info-"))
        })
        .collect();
    info_files.sort();

    let mut preview = ImportPreview::default();
    for info_path in info_files {
        match parse_note(path, &info_path) {
            Ok(note) => preview.importable.push(note),
            Err(error) => preview.skipped.push(ImportIssue {
                path: info_path,
                message: error.to_string(),
            }),
        }
    }
    Ok(preview)
}

fn parse_note(root: &Path, info_path: &Path) -> Result<ImportableNote, ImportError> {
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
    let content_bytes = fs::read(&content_path)?;
    let content = String::from_utf8(content_bytes.clone())?;
    let modified = fs::metadata(info_path)
        .and_then(|metadata| metadata.modified())
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(|_| Utc::now());
    let mut note = Note::new(modified);
    note.content = content;
    note.geometry.width = parse_i32(&values, "width")?;
    note.geometry.height = parse_i32(&values, "height")?;
    note.geometry.x = optional_i32(&values, "x")?;
    note.geometry.y = optional_i32(&values, "y")?;
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
    hasher.update(info_path.file_name().unwrap().as_encoded_bytes());
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
