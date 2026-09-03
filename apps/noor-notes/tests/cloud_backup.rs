use std::sync::Arc;

use chrono::{TimeZone, Utc};
use noor_domain::Note;
use noor_notes::cloud_backup::{BackupConfiguration, CloudBackupController, CloudBackupError};
use noor_notes::cloud_sync::CloudSyncController;
use noor_notes::key_store::{InMemoryKeyStore, KeyStore, SecretKind};
use noor_storage::SqliteNoteRepository;
use noor_sync::{
    BackupArchive, BackupProviderKind, EndpointPolicy, GoogleDriveProvider, OneDriveProvider,
    ProviderOAuth, SupabaseClient,
};
use wiremock::matchers::{body_string_contains, header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Fixture {
    controller: CloudBackupController,
    sync: CloudSyncController,
    repository: SqliteNoteRepository,
    keys: Arc<InMemoryKeyStore>,
}

async fn fixture(cloud: &MockServer, providers: &MockServer, oauth: &MockServer) -> Fixture {
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
        .mount(cloud)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_vaults"))
        .respond_with(ResponseTemplate::new(201))
        .mount(cloud)
        .await;
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.keep().join("notes.db"))
        .await
        .unwrap();
    let keys = Arc::new(InMemoryKeyStore::default());
    let client = SupabaseClient::new(
        &cloud.uri(),
        "anon",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    let sync = CloudSyncController::new(repository.clone(), client, keys.clone());
    sync.attach_session(noor_sync::AuthSession {
        access_token: "cloud-access".into(),
        refresh_token: "cloud-refresh".into(),
        expires_in: 3600,
        user: noor_sync::AuthUser {
            id: "user-one".into(),
            email: "person@example.com".into(),
        },
    })
    .await
    .unwrap();
    let recovery = sync
        .begin_enrollment(b"a strong backup passphrase")
        .await
        .unwrap();
    sync.confirm_enrollment(&recovery).await.unwrap();

    let google = ProviderOAuth::for_test(
        BackupProviderKind::GoogleDrive,
        "google-public",
        &format!("{}/google/authorize", oauth.uri()),
        &format!("{}/google/token", oauth.uri()),
        None,
        "http://127.0.0.1:43818/backup/google",
    )
    .unwrap();
    let onedrive = ProviderOAuth::for_test(
        BackupProviderKind::OneDrive,
        "onedrive-public",
        &format!("{}/onedrive/authorize", oauth.uri()),
        &format!("{}/onedrive/token", oauth.uri()),
        None,
        "http://127.0.0.1:43819/backup/onedrive",
    )
    .unwrap();
    let controller = CloudBackupController::for_test(
        repository.clone(),
        sync.clone(),
        keys.clone(),
        BackupConfiguration::for_test(Some(google), Some(onedrive)),
        GoogleDriveProvider::for_test(&providers.uri()).unwrap(),
        OneDriveProvider::for_test(&providers.uri()).unwrap(),
    );
    Fixture {
        controller,
        sync,
        repository,
        keys,
    }
}

async fn mock_connection(oauth: &MockServer, provider: &str, code: &str, refresh: &str) {
    Mock::given(method("POST"))
        .and(path(format!("/{provider}/token")))
        .and(body_string_contains(format!("code={code}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": format!("{provider}-access"),
            "refresh_token": refresh,
            "expires_in": 3600
        })))
        .expect(1)
        .mount(oauth)
        .await;
}

#[tokio::test]
async fn provider_tokens_are_isolated_and_disconnect_removes_only_one() {
    let cloud = MockServer::start().await;
    let providers = MockServer::start().await;
    let oauth = MockServer::start().await;
    mock_connection(&oauth, "google", "google-code", "google-refresh").await;
    mock_connection(&oauth, "onedrive", "onedrive-code", "onedrive-refresh").await;
    let fixture = fixture(&cloud, &providers, &oauth).await;

    fixture
        .controller
        .connect(BackupProviderKind::GoogleDrive, "google-code", "verifier")
        .await
        .unwrap();
    fixture
        .controller
        .connect(BackupProviderKind::OneDrive, "onedrive-code", "verifier")
        .await
        .unwrap();
    assert!(
        fixture
            .keys
            .get(SecretKind::GoogleDriveSession, "active")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        fixture
            .keys
            .get(SecretKind::OneDriveSession, "active")
            .await
            .unwrap()
            .is_some()
    );

    fixture
        .controller
        .disconnect(BackupProviderKind::GoogleDrive)
        .await
        .unwrap();
    assert!(
        fixture
            .keys
            .get(SecretKind::GoogleDriveSession, "active")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .keys
            .get(SecretKind::OneDriveSession, "active")
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn provider_failure_is_isolated_and_http_receives_no_plaintext() {
    let cloud = MockServer::start().await;
    let providers = MockServer::start().await;
    let oauth = MockServer::start().await;
    mock_connection(&oauth, "google", "google-code", "google-refresh").await;
    mock_connection(&oauth, "onedrive", "onedrive-code", "onedrive-refresh").await;
    let fixture = fixture(&cloud, &providers, &oauth).await;
    let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.title = "Never leave as plaintext".into();
    note.content = "super secret body".into();
    fixture.repository.save_note(&note).await.unwrap();
    fixture
        .controller
        .connect(BackupProviderKind::GoogleDrive, "google-code", "verifier")
        .await
        .unwrap();
    fixture
        .controller
        .connect(BackupProviderKind::OneDrive, "onedrive-code", "verifier")
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&providers)
        .await;
    Mock::given(method("PUT"))
        .and(path_regex(
            "/me/drive/special/approot:/Noor%20Notes/.*:/content",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "backup",
            "name": "backup.nnbackup",
            "lastModifiedDateTime": "2026-09-03T12:00:00Z",
            "size": 100
        })))
        .expect(2)
        .mount(&providers)
        .await;
    Mock::given(method("POST"))
        .and(path("/me/drive/special/approot/children"))
        .respond_with(ResponseTemplate::new(201))
        .expect(2)
        .mount(&providers)
        .await;

    let results = fixture.controller.backup_now("desktop-a").await;
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .any(|result| result.provider == BackupProviderKind::GoogleDrive && !result.uploaded)
    );
    assert!(
        results
            .iter()
            .any(|result| result.provider == BackupProviderKind::OneDrive && result.uploaded)
    );
    for request in providers.received_requests().await.unwrap() {
        assert!(
            !request
                .body
                .windows(note.title.len())
                .any(|bytes| bytes == note.title.as_bytes())
        );
        assert!(
            !request
                .body
                .windows(note.content.len())
                .any(|bytes| bytes == note.content.as_bytes())
        );
    }
}

#[tokio::test]
async fn restore_requires_authenticated_preview_token_and_uses_repository_merge() {
    let cloud = MockServer::start().await;
    let providers = MockServer::start().await;
    let oauth = MockServer::start().await;
    mock_connection(&oauth, "onedrive", "onedrive-code", "onedrive-refresh").await;
    let fixture = fixture(&cloud, &providers, &oauth).await;
    fixture
        .controller
        .connect(BackupProviderKind::OneDrive, "onedrive-code", "verifier")
        .await
        .unwrap();
    assert!(matches!(
        fixture.controller.restore("not-previewed").await,
        Err(CloudBackupError::RestoreNotConfirmed)
    ));

    let now = Utc.with_ymd_and_hms(2026, 9, 3, 13, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.title = "Restored safely".into();
    let vault = fixture.sync.unlocked_vault().await.unwrap();
    let encrypted = BackupArchive::create(&vault, now, "old-device", vec![note.clone()]).unwrap();
    let bytes = serde_json::to_vec(&encrypted).unwrap();
    let object = noor_sync::BackupObject {
        id: "restore-one".into(),
        name: "current.nnbackup".into(),
        modified_at: now,
        size: bytes.len() as u64,
    };
    Mock::given(method("GET"))
        .and(path("/me/drive/items/restore-one/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes))
        .expect(1)
        .mount(&providers)
        .await;

    let preview = fixture
        .controller
        .preview_restore(BackupProviderKind::OneDrive, object)
        .await
        .unwrap();
    assert_eq!(preview.archive.note_count, 1);
    let report = fixture.controller.restore(&preview.token).await.unwrap();
    assert_eq!(report.applied, 1);
    assert_eq!(
        fixture.repository.get_note(note.id).await.unwrap(),
        Some(note)
    );
    assert!(fixture.controller.restore(&preview.token).await.is_err());
}

#[tokio::test]
async fn expired_provider_access_is_refreshed_before_storage_request() {
    let cloud = MockServer::start().await;
    let providers = MockServer::start().await;
    let oauth = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/onedrive/token"))
        .and(body_string_contains("code=short-code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "expiring-access",
            "refresh_token": "rotating-refresh",
            "expires_in": 1
        })))
        .expect(1)
        .mount(&oauth)
        .await;
    let fixture = fixture(&cloud, &providers, &oauth).await;
    fixture
        .controller
        .connect(BackupProviderKind::OneDrive, "short-code", "verifier")
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
    Mock::given(method("POST"))
        .and(path("/onedrive/token"))
        .and(body_string_contains("refresh_token=rotating-refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "fresh-access",
            "refresh_token": "rotated-refresh",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&oauth)
        .await;
    Mock::given(method("GET"))
        .and(path("/me/drive/special/approot:/Noor%20Notes:/children"))
        .and(header("authorization", "Bearer fresh-access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": []})))
        .expect(1)
        .mount(&providers)
        .await;

    assert!(
        fixture
            .controller
            .list_backups(BackupProviderKind::OneDrive)
            .await
            .unwrap()
            .is_empty()
    );
    let stored = fixture
        .keys
        .get(SecretKind::OneDriveSession, "active")
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&stored).contains("rotated-refresh"));
}
