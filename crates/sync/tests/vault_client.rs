use chrono::{TimeZone, Utc};
use noor_crypto::{RecoveryKey, Vault};
use noor_sync::{EndpointPolicy, RemoteVault, SupabaseClient};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn remote_vault() -> RemoteVault {
    let (vault, wrapped_vault) = Vault::create(b"never-send-this-passphrase").unwrap();
    let recovery = RecoveryKey::generate();
    let recovery_wrapped_vault = vault.wrap_for_recovery(&recovery).unwrap();
    RemoteVault {
        wrapped_vault,
        recovery_wrapped_vault,
        updated_at: Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap(),
    }
}

#[tokio::test]
async fn encrypted_vault_upsert_and_fetch_are_authenticated_and_bounded() {
    let server = MockServer::start().await;
    let vault = remote_vault();
    Mock::given(method("POST"))
        .and(path("/rest/v1/encrypted_vaults"))
        .and(header("authorization", "Bearer access"))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .and(query_param(
            "select",
            "wrapped_vault,recovery_wrapped_vault,updated_at",
        ))
        .and(header("authorization", "Bearer access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![vault.clone()]))
        .expect(1)
        .mount(&server)
        .await;
    let client = SupabaseClient::new(
        &server.uri(),
        "anon",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();

    client.put_vault("access", &vault).await.unwrap();
    assert_eq!(client.get_vault("access").await.unwrap(), Some(vault));

    let requests = server.received_requests().await.unwrap();
    let upload = requests
        .iter()
        .find(|request| request.method.as_str() == "POST")
        .unwrap();
    let body = String::from_utf8(upload.body.clone()).unwrap();
    assert!(!body.contains("never-send-this-passphrase"));
    assert!(!body.contains("vault_key"));
}

#[tokio::test]
async fn missing_vault_is_none_and_auth_failure_is_not_treated_as_missing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .and(header("authorization", "Bearer empty"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<RemoteVault>::new()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/rest/v1/encrypted_vaults"))
        .and(header("authorization", "Bearer expired"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let client = SupabaseClient::new(
        &server.uri(),
        "anon",
        EndpointPolicy::AllowLoopbackHttpForTests,
    )
    .unwrap();

    assert_eq!(client.get_vault("empty").await.unwrap(), None);
    assert!(client.get_vault("expired").await.is_err());
}
