use noor_notes::key_store::{InMemoryKeyStore, KeyStore, KeyStoreError, SecretKind};

#[tokio::test]
async fn secrets_are_namespaced_and_deletable() {
    let store = InMemoryKeyStore::default();
    store
        .put(SecretKind::DatabaseKey, "same", b"db-key")
        .await
        .unwrap();
    store
        .put(SecretKind::RefreshToken, "same", b"token")
        .await
        .unwrap();
    store
        .put(SecretKind::WritingAssistanceApiKey, "same", b"provider-key")
        .await
        .unwrap();
    assert_eq!(
        store
            .get(SecretKind::DatabaseKey, "same")
            .await
            .unwrap()
            .unwrap()
            .as_slice(),
        b"db-key"
    );
    assert_eq!(
        store
            .get(SecretKind::RefreshToken, "same")
            .await
            .unwrap()
            .unwrap()
            .as_slice(),
        b"token"
    );
    assert_eq!(
        store
            .get(SecretKind::WritingAssistanceApiKey, "same")
            .await
            .unwrap()
            .unwrap()
            .as_slice(),
        b"provider-key"
    );
    store
        .delete(SecretKind::RefreshToken, "same")
        .await
        .unwrap();
    assert!(
        store
            .get(SecretKind::RefreshToken, "same")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .get(SecretKind::DatabaseKey, "same")
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn duplicate_secret_matches_fail_closed_and_errors_are_redacted() {
    let store = InMemoryKeyStore::default();
    store.inject_duplicate_for_test(
        SecretKind::WrappedVault,
        "person@example.com",
        b"top-secret",
    );
    let error = store
        .get(SecretKind::WrappedVault, "person@example.com")
        .await
        .unwrap_err();
    assert!(matches!(error, KeyStoreError::Ambiguous));
    let rendered = error.to_string();
    assert!(!rendered.contains("top-secret"));
    assert!(!rendered.contains("person@example.com"));
}
