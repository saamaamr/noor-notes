use noor_sync::{EndpointPolicy, SupabaseClient, SyncClientError};
use url::Url;

const BUILT_URL: Option<&str> = option_env!("NOOR_SUPABASE_URL");
const BUILT_KEY: Option<&str> = option_env!("NOOR_SUPABASE_PUBLISHABLE_KEY");

#[derive(Clone)]
pub struct CloudConfig {
    base_url: String,
    publishable_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CloudConfigError {
    #[error("Noor cloud is not configured in this build")]
    NotConfigured,
    #[error("the Noor cloud endpoint is invalid")]
    InvalidEndpoint,
    #[error("the Noor cloud endpoint must use HTTPS")]
    InsecureEndpoint,
    #[error("a privileged Supabase key cannot be embedded in Noor Notes")]
    PrivilegedKey,
}

impl CloudConfig {
    pub fn load() -> Result<Self, CloudConfigError> {
        let base_url = std::env::var("NOOR_SUPABASE_URL")
            .ok()
            .or_else(|| BUILT_URL.map(str::to_owned))
            .ok_or(CloudConfigError::NotConfigured)?;
        let publishable_key = std::env::var("NOOR_SUPABASE_PUBLISHABLE_KEY")
            .ok()
            .or_else(|| BUILT_KEY.map(str::to_owned))
            .ok_or(CloudConfigError::NotConfigured)?;
        Self::new(&base_url, &publishable_key)
    }

    pub fn new(base_url: &str, publishable_key: &str) -> Result<Self, CloudConfigError> {
        let base_url = base_url.trim();
        let publishable_key = publishable_key.trim();
        let parsed = Url::parse(base_url).map_err(|_| CloudConfigError::InvalidEndpoint)?;
        if parsed.scheme() != "https" {
            return Err(CloudConfigError::InsecureEndpoint);
        }
        if parsed.host().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(CloudConfigError::InvalidEndpoint);
        }
        if publishable_key.is_empty() {
            return Err(CloudConfigError::NotConfigured);
        }
        if publishable_key.starts_with("sb_secret_")
            || publishable_key
                .to_ascii_lowercase()
                .contains("service_role")
        {
            return Err(CloudConfigError::PrivilegedKey);
        }
        Ok(Self {
            base_url: base_url.to_owned(),
            publishable_key: publishable_key.to_owned(),
        })
    }

    pub fn client(&self) -> Result<SupabaseClient, SyncClientError> {
        SupabaseClient::new(
            &self.base_url,
            &self.publishable_key,
            EndpointPolicy::Production,
        )
    }
}
