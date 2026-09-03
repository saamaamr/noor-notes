use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{Duration, TimeZone, Utc};
use noor_crypto::Vault;
use noor_domain::Note;
use noor_storage::SqliteNoteRepository;
use noor_sync::{
    EndpointPolicy, RemoteRevision, SupabaseClient, SyncCursor, SyncStatus, SyncWorker,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn encrypted_revision(vault: &Vault, note: &Note) -> RemoteRevision {
    let envelope = vault
        .encrypt_note(note.id, note.revision, &serde_json::to_vec(note).unwrap())
        .unwrap();
    RemoteRevision {
        note_id: note.id.value(),
        revision: note.revision.value(),
        ciphertext: STANDARD.encode(envelope.ciphertext),
        nonce: STANDARD.encode(envelope.nonce),
        updated_at: note.updated_at,
        deleted_at: None,
    }
}

async fn worker(
    repository: SqliteNoteRepository,
    vault: Vault,
    revisions: Vec<RemoteRevision>,
) -> (SyncWorker, MockServer) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(revisions))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(
        &server.uri(),
        "anon",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    (
        SyncWorker::new(repository, client, Arc::new(vault), "access"),
        server,
    )
}

#[tokio::test]
async fn cycle_downloads_authenticated_revision_and_advances_cursor_after_commit() {
    let dir = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
    let (vault, _) = Vault::create(b"sync passphrase").unwrap();
    let mut remote = Note::new(now);
    remote.content = "downloaded ciphertext".into();
    remote.updated_at = now + Duration::minutes(1);
    let revision = encrypted_revision(&vault, &remote);
    let expected_cursor = SyncCursor::from_revision(&revision);
    let (worker, _server) = worker(repository.clone(), vault, vec![revision]).await;

    let cycle = worker
        .run_cycle(SyncCursor::default(), "desktop", now + Duration::minutes(2))
        .await;

    assert_eq!(cycle.status, SyncStatus::Idle);
    assert_eq!(cycle.downloaded, 1);
    assert_eq!(cycle.cursor, expected_cursor);
    assert_eq!(
        repository
            .get_note(remote.id)
            .await
            .unwrap()
            .unwrap()
            .content,
        "downloaded ciphertext"
    );
    assert!(repository.pending_changes(10).await.unwrap().is_empty());
}

#[tokio::test]
async fn failed_remote_authentication_leaves_cursor_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
    let (encrypting_vault, _) = Vault::create(b"first vault").unwrap();
    let (worker_vault, _) = Vault::create(b"different vault").unwrap();
    let remote = Note::new(now);
    let revision = encrypted_revision(&encrypting_vault, &remote);
    let (worker, _server) = worker(repository, worker_vault, vec![revision]).await;
    let original = SyncCursor::default();

    let cycle = worker.run_cycle(original, "desktop", now).await;

    assert_eq!(cycle.status, SyncStatus::Error);
    assert_eq!(cycle.cursor, original);
    assert_eq!(cycle.downloaded, 0);
}

#[tokio::test]
async fn replay_at_saved_cursor_does_not_duplicate_a_conflict_copy() {
    let dir = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
    let mut local = Note::new(now);
    local.content = "local edit".into();
    repository.save_note(&local).await.unwrap();
    let pending = repository.pending_changes(1).await.unwrap().remove(0);
    repository.ack_change(pending.id).await.unwrap();
    let (vault, _) = Vault::create(b"sync passphrase").unwrap();
    let mut remote = local.clone();
    remote.content = "remote edit".into();
    remote.updated_at = now + Duration::seconds(1);
    let revision = encrypted_revision(&vault, &remote);
    let (worker, _server) = worker(repository.clone(), vault, vec![revision]).await;

    let first = worker
        .run_cycle(SyncCursor::default(), "laptop", now + Duration::minutes(1))
        .await;
    assert_eq!(first.downloaded, 1);
    assert_eq!(repository.search_notes("").await.unwrap().len(), 2);

    let second = worker
        .run_cycle(first.cursor, "laptop", now + Duration::minutes(1))
        .await;
    assert_eq!(second.downloaded, 0);
    assert_eq!(repository.search_notes("").await.unwrap().len(), 2);
}
