use std::sync::Arc;

use chrono::{TimeZone, Utc};
use noor_crypto::Vault;
use noor_domain::Note;
use noor_storage::SqliteNoteRepository;
use noor_sync::{SupabaseClient, SyncStatus, SyncWorker};
use tempfile::tempdir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn failed_upload_stays_queued_then_acknowledges_after_reconnect() {
    let dir = tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let mut note = Note::new(Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap());
    note.content = "offline edit".into();
    repo.save_note(&note).await.unwrap();
    let (vault, _) = Vault::create(b"sync passphrase").unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(&server.uri(), "anon").unwrap();
    let worker = SyncWorker::new(repo.clone(), client, Arc::new(vault), "access");

    assert_eq!(worker.run_once().await, SyncStatus::Offline);
    assert_eq!(repo.pending_changes(10).await.unwrap().len(), 1);
    assert_eq!(worker.run_once().await, SyncStatus::Idle);
    assert!(repo.pending_changes(10).await.unwrap().is_empty());
    assert_eq!(worker.run_once().await, SyncStatus::Idle);
}
