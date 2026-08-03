use std::path::PathBuf;

use noor_domain::Note;

#[derive(Clone, Debug)]
pub struct ImportableNote {
    pub source_key: String,
    pub note: Note,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct ImportPreview {
    pub importable: Vec<ImportableNote>,
    pub skipped: Vec<ImportIssue>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub imported: usize,
    pub already_imported: usize,
    pub skipped: Vec<ImportIssue>,
}
