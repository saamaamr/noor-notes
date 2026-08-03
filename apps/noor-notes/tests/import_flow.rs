use std::path::{Path, PathBuf};

use noor_notes::import_dialog::ImportFlow;
use noor_storage::SqliteNoteRepository;

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/xpad")
}

#[tokio::test]
async fn preview_reports_errors_and_requires_confirmation_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let flow = ImportFlow::from_path(&fixture_path()).unwrap();

    assert_eq!(flow.preview().importable.len(), 2);
    assert_eq!(flow.preview().skipped.len(), 1);
    assert!(repo.search_notes("").await.unwrap().is_empty());

    let report = flow.confirm(&repo).await.unwrap();
    assert_eq!(report.imported, 2);
    assert_eq!(repo.search_notes("").await.unwrap().len(), 2);
}
