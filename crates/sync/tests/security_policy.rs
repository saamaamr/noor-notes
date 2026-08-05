use chrono::{Duration, Utc};
use noor_sync::{
    EndpointPolicy, RemoteRevision, RevisionValidationError, SupabaseClient, SyncClientError,
};
use uuid::Uuid;

#[test]
fn production_rejects_insecure_or_credentialed_endpoints() {
    assert!(matches!(
        SupabaseClient::new("http://example.com", "anon", EndpointPolicy::Production),
        Err(SyncClientError::InsecureUrl)
    ));
    assert!(matches!(
        SupabaseClient::new(
            "https://user:pass@example.com",
            "anon",
            EndpointPolicy::Production
        ),
        Err(SyncClientError::InvalidUrl)
    ));
    assert!(
        SupabaseClient::new(
            "http://127.0.0.1:8080",
            "anon",
            EndpointPolicy::AllowLoopbackHttpForTests
        )
        .is_ok()
    );
    assert!(matches!(
        SupabaseClient::new(
            "http://example.com",
            "anon",
            EndpointPolicy::AllowLoopbackHttpForTests
        ),
        Err(SyncClientError::InsecureUrl)
    ));
}

#[test]
fn remote_revision_limits_are_checked_before_decode() {
    let now = Utc::now();
    let mut revision = RemoteRevision {
        note_id: Uuid::new_v4(),
        revision: 1,
        ciphertext: "AA==".into(),
        nonce: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
        updated_at: now,
        deleted_at: None,
    };
    assert!(revision.validate(now).is_ok());
    revision.nonce = "short".into();
    assert_eq!(
        revision.validate(now),
        Err(RevisionValidationError::InvalidNonce)
    );
    revision.nonce = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into();
    revision.updated_at = now + Duration::minutes(6);
    assert_eq!(
        revision.validate(now),
        Err(RevisionValidationError::InvalidTimestamp)
    );
}
