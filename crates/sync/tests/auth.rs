use noor_sync::{AuthSession, EndpointPolicy, SupabaseClient};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const USER_ID: &str = "01990df4-6d31-7d63-a242-f58f237fd8dc";

fn session_json(access_token: &str, refresh_token: &str) -> serde_json::Value {
    json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_in": 3600,
        "user": {
            "id": USER_ID,
            "email": "writer@example.com"
        }
    })
}

async fn client() -> (MockServer, SupabaseClient) {
    let server = MockServer::start().await;
    let client = SupabaseClient::new(
        &server.uri(),
        "sb_publishable_test",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();
    (server, client)
}

#[tokio::test]
async fn signup_reports_email_confirmation_without_inventing_a_session() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/signup"))
        .and(header("apikey", "sb_publishable_test"))
        .and(body_json(json!({
            "email": "writer@example.com",
            "password": "correct horse battery staple"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": USER_ID,
            "email": "writer@example.com"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let outcome = client
        .sign_up("writer@example.com", "correct horse battery staple")
        .await
        .unwrap();

    assert!(outcome.confirmation_required);
    assert!(outcome.session.is_none());
    assert_eq!(outcome.user.id, USER_ID);
    assert_eq!(outcome.user.email, "writer@example.com");
}

#[tokio::test]
async fn signup_returns_a_real_session_when_confirmation_is_disabled() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/signup"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(session_json("new-access", "new-refresh")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let outcome = client
        .sign_up("writer@example.com", "correct horse battery staple")
        .await
        .unwrap();

    assert!(!outcome.confirmation_required);
    assert_eq!(outcome.session.unwrap().access_token, "new-access");
}

#[tokio::test]
async fn google_oauth_uses_s256_pkce_without_drive_permission() {
    let (_, client) = client().await;
    let oauth = client
        .google_oauth_pkce("http://127.0.0.1:43817/auth/callback")
        .unwrap();
    let query = oauth
        .authorization_url
        .query_pairs()
        .into_owned()
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(query.get("provider").map(String::as_str), Some("google"));
    let redirect = reqwest::Url::parse(&query["redirect_to"]).unwrap();
    assert_eq!(redirect.scheme(), "http");
    assert_eq!(redirect.host_str(), Some("127.0.0.1"));
    assert_eq!(redirect.port(), Some(43_817));
    assert_eq!(redirect.path(), "/auth/callback");
    assert_eq!(
        redirect
            .query_pairs()
            .find_map(|(key, value)| (key == "nn_state").then(|| value.into_owned())),
        Some(oauth.state.clone())
    );
    assert_eq!(
        query.get("code_challenge_method").map(String::as_str),
        Some("s256")
    );
    assert!(query["code_challenge"].len() >= 43);
    assert!(oauth.verifier.len() >= 43);
    assert!(oauth.state.len() >= 32);
    assert!(!oauth.authorization_url.as_str().contains("drive"));
}

#[tokio::test]
async fn oauth_code_exchange_sends_the_original_verifier() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/token"))
        .and(query_param("grant_type", "pkce"))
        .and(body_json(json!({
            "auth_code": "one-time-code",
            "code_verifier": "original-verifier"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(session_json("oauth-access", "oauth-refresh")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let session = client
        .exchange_oauth_code("one-time-code", "original-verifier")
        .await
        .unwrap();

    assert_eq!(session.user.email, "writer@example.com");
    assert_eq!(session.refresh_token, "oauth-refresh");
}

#[tokio::test]
async fn refresh_user_and_logout_use_only_the_supabase_session() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/token"))
        .and(query_param("grant_type", "refresh_token"))
        .and(body_json(json!({ "refresh_token": "stored-refresh" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(session_json("fresh-access", "fresh-refresh")),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .and(header("authorization", "Bearer fresh-access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": USER_ID,
            "email": "writer@example.com"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/auth/v1/logout"))
        .and(header("authorization", "Bearer fresh-access"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let session: AuthSession = client.refresh_session("stored-refresh").await.unwrap();
    let user = client.user(&session.access_token).await.unwrap();
    client.sign_out(&session.access_token).await.unwrap();

    assert_eq!(session.refresh_token, "fresh-refresh");
    assert_eq!(user.id, USER_ID);
}
