use std::sync::Arc;

use chrono::{TimeZone, Utc};
use noor_crypto::{RecoveryKey, Vault};
use noor_notes::cloud_sync::{CloudSyncController, CloudSyncState};
use noor_notes::key_store::{InMemoryKeyStore, KeyStore, SecretKind};
use noor_storage::SqliteNoteRepository;
use noor_sync::{AuthSession, AuthUser, EndpointPolicy, RemoteVault, SupabaseClient, SyncStatus};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn session(access: &str, refresh: &str) -> AuthSession {
    AuthSession {
        access_token: access.into(),
        refresh_token: refresh.into(),
        expires_in: 3600,
        user: AuthUser {
            id: "018f2f91-8d87-7c4a-a9ee-9b90518f4123".into(),
            email: "person@example.com".into(),
        },
    }
}

fn remote_vault(passphrase: &[u8]) -> RemoteVault {
    let (vault, wrapped_vault) = Vault::create(passphrase).unwrap();
    let recovery = RecoveryKey::generate();
    RemoteVault {
        wrapped_vault,
        recovery_wrapped_vault: vault.wrap_for_recovery(&recovery).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap(),
    }
}

async fn controller(server: &MockServer, keys: Arc<InMemoryKeyStore>) -> CloudSyncController {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.keep().join("notes.db");
    let repository = SqliteNoteRepository::open(&path).await.unwrap();
    let client = SupabaseClient::new(
        &server.uri(),
        "anon",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    CloudSyncController::new(repository, client, keys)
}

#[tokio::test]
async fn enrollment_uploads_only_after_exact_recovery_confirmation() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<RemoteVault>::new()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;
    let keys = Arc::new(InMemoryKeyStore::default());
    let controller = controller(&server, keys.clone()).await;

    assert_eq!(
        controller
            .attach_session(session("access", "refresh"))
            .await
            .unwrap(),
        CloudSyncState::EnrollmentRequired
    );
    let recovery = controller
        .begin_enrollment(b"a strong sync passphrase")
        .await
        .unwrap();
    assert_eq!(
        controller.state().await,
        CloudSyncState::RecoveryConfirmation
    );
    assert!(controller.confirm_enrollment("wrong-key").await.is_err());
    assert_eq!(
        controller.state().await,
        CloudSyncState::RecoveryConfirmation
    );

    controller.confirm_enrollment(&recovery).await.unwrap();

    assert_eq!(controller.state().await, CloudSyncState::Ready);
    assert!(
        keys.get(
            SecretKind::SyncVault,
            "018f2f91-8d87-7c4a-a9ee-9b90518f4123"
        )
        .await
        .unwrap()
        .is_some()
    );
}

#[tokio::test]
async fn existing_remote_vault_stays_locked_until_correct_passphrase() {
    let server = MockServer::start().await;
    let vault = remote_vault(b"correct passphrase");
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![vault]))
        .mount(&server)
        .await;
    let keys = Arc::new(InMemoryKeyStore::default());
    let controller = controller(&server, keys).await;

    assert_eq!(
        controller
            .attach_session(session("access", "refresh"))
            .await
            .unwrap(),
        CloudSyncState::Locked
    );
    assert!(controller.unlock_with_passphrase(b"wrong").await.is_err());
    assert_eq!(controller.state().await, CloudSyncState::Locked);
    controller
        .unlock_with_passphrase(b"correct passphrase")
        .await
        .unwrap();
    assert_eq!(controller.state().await, CloudSyncState::Ready);
}

#[tokio::test]
async fn auth_required_cycle_refreshes_session_and_retries_once() {
    let server = MockServer::start().await;
    let vault = remote_vault(b"correct passphrase");
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![vault]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .and(header("authorization", "Bearer expired"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/token"))
        .and(query_param("grant_type", "refresh_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(session("fresh", "rotated")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .and(header("authorization", "Bearer fresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
        .expect(1)
        .mount(&server)
        .await;
    let keys = Arc::new(InMemoryKeyStore::default());
    let controller = controller(&server, keys.clone()).await;
    controller
        .attach_session(session("expired", "refresh"))
        .await
        .unwrap();
    controller
        .unlock_with_passphrase(b"correct passphrase")
        .await
        .unwrap();

    let cycle = controller.run_once("desktop").await.unwrap();

    assert_eq!(cycle.status, SyncStatus::Idle);
    assert_eq!(controller.state().await, CloudSyncState::Ready);
    let stored = keys
        .get(SecretKind::CloudSession, "active")
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&stored).contains("rotated"));
}

#[tokio::test]
async fn failed_cycle_can_be_retried_without_reunlocking_the_vault() {
    let server = MockServer::start().await;
    let vault = remote_vault(b"correct passphrase");
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![vault]))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let keys = Arc::new(InMemoryKeyStore::default());
    let controller = controller(&server, keys).await;
    controller
        .attach_session(session("access", "refresh"))
        .await
        .unwrap();
    controller
        .unlock_with_passphrase(b"correct passphrase")
        .await
        .unwrap();

    let first = controller.run_once("desktop").await.unwrap();
    assert_eq!(first.status, SyncStatus::Error);
    assert_eq!(controller.state().await, CloudSyncState::Error);

    server.reset().await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
        .mount(&server)
        .await;

    let retry = controller.run_once("desktop").await.unwrap();
    assert_eq!(retry.status, SyncStatus::Idle);
    assert_eq!(controller.state().await, CloudSyncState::Ready);
}
