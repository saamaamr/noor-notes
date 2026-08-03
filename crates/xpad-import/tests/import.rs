use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use noor_storage::SqliteNoteRepository;
use noor_xpad_import::{import_xpad, scan_xpad};
use sha2::{Digest, Sha256};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/xpad")
}

fn hash_fixture_tree(path: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            let digest = Sha256::digest(fs::read(entry.path()).unwrap()).to_vec();
            (name, digest)
        })
        .collect()
}

#[tokio::test]
async fn import_is_read_only_and_idempotent() {
    let before = hash_fixture_tree(&fixture_path());
    let preview = scan_xpad(&fixture_path()).unwrap();
    assert_eq!(preview.importable.len(), 2);
    assert_eq!(preview.skipped.len(), 1);
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();

    let first = import_xpad(&preview, &repo).await.unwrap();
    let second = import_xpad(&preview, &repo).await.unwrap();

    assert_eq!(first.imported, 2);
    assert_eq!(second.imported, 0);
    assert_eq!(before, hash_fixture_tree(&fixture_path()));
    assert_eq!(repo.search_notes("বাংলা").await.unwrap().len(), 1);
}
