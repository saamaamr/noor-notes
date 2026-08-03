use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{Duration, TimeZone, Utc};
use noor_crypto::Vault;
use noor_domain::{Note, Revision};
use noor_storage::SqliteNoteRepository;
use noor_sync::{RemoteRevision, SupabaseClient, SyncWorker};

#[tokio::test]
async fn authenticated_newer_remote_revision_applies_without_upload_echo() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let mut local = Note::new(now);
    local.content = "old".into();
    repo.save_note(&local).await.unwrap();
    let local_change = repo.pending_changes(1).await.unwrap().remove(0);
    repo.ack_change(local_change.id).await.unwrap();
    let (vault, _) = Vault::create(b"download key").unwrap();
    let mut remote_note = local.clone();
    remote_note.content = "new from desktop".into();
    remote_note.revision = Revision::from_value(1);
    remote_note.updated_at = now + Duration::minutes(1);
    let envelope = vault
        .encrypt_note(
            remote_note.id,
            remote_note.revision,
            &serde_json::to_vec(&remote_note).unwrap(),
        )
        .unwrap();
    let remote = RemoteRevision {
        note_id: remote_note.id.value(),
        revision: 1,
        ciphertext: STANDARD.encode(envelope.ciphertext),
        nonce: STANDARD.encode(envelope.nonce),
        updated_at: remote_note.updated_at,
        deleted_at: None,
    };
    let client = SupabaseClient::new("https://example.supabase.co", "anon").unwrap();
    let worker = SyncWorker::new(repo.clone(), client, Arc::new(vault), "access");

    worker
        .apply_remote_revision(remote, "desktop", now)
        .await
        .unwrap();

    assert_eq!(
        repo.get_note(local.id).await.unwrap().unwrap().content,
        "new from desktop"
    );
    assert!(repo.pending_changes(10).await.unwrap().is_empty());
}
