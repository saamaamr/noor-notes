use std::sync::Arc;

use noor_notes::account::AccountController;
use noor_notes::key_store::{InMemoryKeyStore, KeyStore, SecretKind};
use noor_sync::{EndpointPolicy, SupabaseClient};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const USER_ID: &str = "01990df4-6d31-7d63-a242-f58f237fd8dc";

#[tokio::test]
async fn sign_in_persists_one_restorable_cloud_session_without_the_password() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/token"))
        .and(query_param("grant_type", "password"))
        .and(body_json(json!({
            "email": "writer@example.com",
            "password": "correct horse battery staple"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "short-lived-access",
            "refresh_token": "stored-refresh",
            "expires_in": 3600,
            "user": {
                "id": USER_ID,
                "email": "writer@example.com"
            }
        })))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(
        &server.uri(),
        "sb_publishable_test",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    let keys = Arc::new(InMemoryKeyStore::default());
    let controller = AccountController::with_key_store(client, keys.clone());

    let session = controller
        .sign_in("writer@example.com", "correct horse battery staple")
        .await
        .unwrap();

    assert_eq!(session.user.id, USER_ID);
    let stored = keys
        .get(SecretKind::CloudSession, "active")
        .await
        .unwrap()
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&stored).unwrap();
    assert_eq!(value["user_id"], USER_ID);
    assert_eq!(value["email"], "writer@example.com");
    assert_eq!(value["refresh_token"], "stored-refresh");
    assert!(!String::from_utf8_lossy(&stored).contains("correct horse"));
}

#[tokio::test]
async fn restore_rotates_the_refresh_token_for_the_same_account() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/token"))
        .and(query_param("grant_type", "refresh_token"))
        .and(body_json(json!({ "refresh_token": "stored-refresh" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "fresh-access",
            "refresh_token": "rotated-refresh",
            "expires_in": 3600,
            "user": {
                "id": USER_ID,
                "email": "writer@example.com"
            }
        })))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(
        &server.uri(),
        "sb_publishable_test",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    let keys = Arc::new(InMemoryKeyStore::default());
    keys.put(
        SecretKind::CloudSession,
        "active",
        serde_json::to_string(&json!({
            "user_id": USER_ID,
            "email": "writer@example.com",
            "refresh_token": "stored-refresh"
        }))
        .unwrap()
        .as_bytes(),
    )
    .await
    .unwrap();
    let controller = AccountController::with_key_store(client, keys.clone());

    let session = controller.restore_session().await.unwrap().unwrap();

    assert_eq!(session.refresh_token, "rotated-refresh");
    let stored = keys
        .get(SecretKind::CloudSession, "active")
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&stored).contains("rotated-refresh"));
    assert!(!String::from_utf8_lossy(&stored).contains("stored-refresh"));
}

#[tokio::test]
async fn sign_out_clears_only_cloud_session_material_when_revoke_is_offline() {
    let server = MockServer::start().await;
    let client = SupabaseClient::new(
        &server.uri(),
        "sb_publishable_test",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    let keys = Arc::new(InMemoryKeyStore::default());
    keys.put(SecretKind::CloudSession, "active", b"stored-cloud-session")
        .await
        .unwrap();
    keys.put(
        SecretKind::DatabaseKey,
        "local-default",
        b"local-database-key",
    )
    .await
    .unwrap();
    let controller = AccountController::with_key_store(client, keys.clone());

    assert!(controller.sign_out(Some("expired-access")).await.is_err());

    assert!(
        keys.get(SecretKind::CloudSession, "active")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        keys.get(SecretKind::DatabaseKey, "local-default")
            .await
            .unwrap()
            .unwrap()
            .as_slice(),
        b"local-database-key"
    );
}

#[tokio::test]
async fn restore_rejects_account_substitution_and_clears_the_stale_session() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/token"))
        .and(query_param("grant_type", "refresh_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "wrong-access",
            "refresh_token": "wrong-refresh",
            "expires_in": 3600,
            "user": {
                "id": "01990df4-6d31-7d63-a242-f58f237fd999",
                "email": "attacker@example.com"
            }
        })))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(
        &server.uri(),
        "sb_publishable_test",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    let keys = Arc::new(InMemoryKeyStore::default());
    keys.put(
        SecretKind::CloudSession,
        "active",
        serde_json::to_string(&json!({
            "user_id": USER_ID,
            "email": "writer@example.com",
            "refresh_token": "stored-refresh"
        }))
        .unwrap()
        .as_bytes(),
    )
    .await
    .unwrap();
    let controller = AccountController::with_key_store(client, keys.clone());

    let error = match controller.restore_session().await {
        Err(error) => error,
        Ok(_) => panic!("account substitution must fail closed"),
    };

    assert_eq!(error.to_string(), "the cloud account changed unexpectedly");
    assert!(
        keys.get(SecretKind::CloudSession, "active")
            .await
            .unwrap()
            .is_none()
    );
}
