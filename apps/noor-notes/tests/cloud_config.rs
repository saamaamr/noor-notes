use noor_notes::cloud_config::{CloudConfig, CloudConfigError};

#[test]
fn production_config_rejects_insecure_credentialed_and_service_role_values() {
    assert!(matches!(
        CloudConfig::new("http://example.com", "sb_publishable_test"),
        Err(CloudConfigError::InsecureEndpoint)
    ));
    assert!(matches!(
        CloudConfig::new(
            "https://user:password@example.supabase.co",
            "sb_publishable_test"
        ),
        Err(CloudConfigError::InvalidEndpoint)
    ));
    for privileged in ["service_role.secret-material", "sb_secret_server-only"] {
        assert!(matches!(
            CloudConfig::new("https://example.supabase.co", privileged),
            Err(CloudConfigError::PrivilegedKey)
        ));
    }
}

#[test]
fn valid_public_config_builds_a_production_client() {
    let config = CloudConfig::new("https://example.supabase.co", "sb_publishable_test").unwrap();
    assert!(config.client().is_ok());
}
