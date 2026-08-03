use std::time::Duration;

use chrono::{TimeZone, Utc};
use noor_sync::{RemoteRevision, SupabaseClient, SyncClientError};
use uuid::Uuid;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn revision() -> RemoteRevision {
    RemoteRevision {
        note_id: Uuid::parse_str("018f2f91-8d87-7c4a-a9ee-9b90518f4123").unwrap(),
        revision: 4,
        ciphertext: "Y2lwaGVydGV4dA==".into(),
        nonce: "bm9uY2U=".into(),
        updated_at: Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap(),
        deleted_at: None,
    }
}

#[tokio::test]
async fn upload_is_authenticated_and_duplicate_is_idempotent() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .and(header("authorization", "Bearer access-token"))
        .respond_with(ResponseTemplate::new(409))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(&server.uri(), "anon-key").unwrap();

    client
        .upload_revision("access-token", &revision())
        .await
        .unwrap();
}

#[tokio::test]
async fn expired_token_and_rate_limit_are_actionable_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .and(header("authorization", "Bearer expired"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_note_revisions"))
        .and(header("authorization", "Bearer limited"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "7"))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(&server.uri(), "anon-key").unwrap();

    assert!(matches!(
        client.upload_revision("expired", &revision()).await,
        Err(SyncClientError::AuthRequired)
    ));
    assert!(matches!(
        client.upload_revision("limited", &revision()).await,
        Err(SyncClientError::RateLimited(delay)) if delay == Duration::from_secs(7)
    ));
}
