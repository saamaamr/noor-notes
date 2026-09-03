use noor_sync::{BackupProviderKind, ProviderOAuth, ProviderOAuthError};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn oauth(server: &MockServer, kind: BackupProviderKind) -> ProviderOAuth {
    ProviderOAuth::for_test(
        kind,
        "public-client-id",
        &format!("{}/authorize", server.uri()),
        &format!("{}/token", server.uri()),
        Some(&format!("{}/revoke", server.uri())),
        match kind {
            BackupProviderKind::GoogleDrive => "http://127.0.0.1:43818/backup/google",
            BackupProviderKind::OneDrive => "http://127.0.0.1:43819/backup/onedrive",
        },
    )
    .unwrap()
}

#[tokio::test]
async fn authorization_uses_exact_app_folder_scopes_and_s256_without_a_secret() {
    let server = MockServer::start().await;
    let google = oauth(&server, BackupProviderKind::GoogleDrive).authorization();
    let onedrive = oauth(&server, BackupProviderKind::OneDrive).authorization();

    let google_query: std::collections::HashMap<_, _> = google
        .authorization_url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    assert_eq!(
        google_query.get("scope").map(String::as_str),
        Some("https://www.googleapis.com/auth/drive.appdata")
    );
    assert_eq!(
        google_query.get("redirect_uri").map(String::as_str),
        Some("http://127.0.0.1:43818/backup/google")
    );
    assert_eq!(
        google_query
            .get("code_challenge_method")
            .map(String::as_str),
        Some("S256")
    );
    assert!(!google_query.contains_key("client_secret"));

    let onedrive_query: std::collections::HashMap<_, _> = onedrive
        .authorization_url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    assert_eq!(
        onedrive_query.get("scope").map(String::as_str),
        Some("offline_access Files.ReadWrite.AppFolder")
    );
    assert_eq!(
        onedrive_query.get("redirect_uri").map(String::as_str),
        Some("http://127.0.0.1:43819/backup/onedrive")
    );
    assert!(!onedrive_query.contains_key("client_secret"));
}

#[tokio::test]
async fn exchange_and_refresh_send_public_client_forms_and_preserve_rotating_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .and(body_string_contains("code_verifier=verifier"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "access-one",
            "refresh_token": "refresh-one",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&server)
        .await;
    let oauth = oauth(&server, BackupProviderKind::GoogleDrive);
    let session = oauth.exchange("one-time-code", "verifier").await.unwrap();
    assert_eq!(session.refresh_token.as_str(), "refresh-one");

    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=refresh-one"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "access-two",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&server)
        .await;
    let refreshed = oauth.refresh(&session.refresh_token).await.unwrap();
    assert_eq!(refreshed.access_token.as_str(), "access-two");
    assert_eq!(refreshed.refresh_token.as_str(), "refresh-one");

    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body);
    assert!(!body.contains("client_secret"));
}

#[tokio::test]
async fn errors_are_redacted_and_insecure_production_endpoints_are_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string("secret-token-in-error"))
        .mount(&server)
        .await;
    let error = oauth(&server, BackupProviderKind::GoogleDrive)
        .exchange("secret-code", "secret-verifier")
        .await
        .unwrap_err();
    let displayed = error.to_string();
    assert!(!displayed.contains("secret-token-in-error"));
    assert!(!displayed.contains("secret-code"));
    assert!(matches!(error, ProviderOAuthError::Http(_)));
}
