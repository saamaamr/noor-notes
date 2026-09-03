use chrono::{DateTime, Utc};

use crate::EndpointPolicy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupObject {
    pub id: String,
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum BackupProviderError {
    #[error("invalid backup provider configuration")]
    InvalidConfiguration,
    #[error("backup provider endpoint must use HTTPS")]
    InsecureEndpoint,
    #[error("backup authorization is required")]
    AuthorizationRequired,
    #[error("backup object name is unsafe")]
    UnsafeName,
    #[error("backup object exceeds 128 MiB")]
    TooLarge,
    #[error("backup provider response was malformed")]
    MalformedResponse,
    #[error("backup provider response exceeded its security limit")]
    ResponseTooLarge,
    #[error("backup provider request failed")]
    Transport,
    #[error("backup provider returned HTTP {0}")]
    Http(reqwest::StatusCode),
}

#[allow(async_fn_in_trait)]
pub trait BackupProvider {
    async fn upload(
        &self,
        access_token: &str,
        name: &str,
        encrypted: &[u8],
    ) -> Result<BackupObject, BackupProviderError>;
    async fn list(&self, access_token: &str) -> Result<Vec<BackupObject>, BackupProviderError>;
    async fn download(
        &self,
        access_token: &str,
        object: &BackupObject,
    ) -> Result<Vec<u8>, BackupProviderError>;
    async fn delete(
        &self,
        access_token: &str,
        object: &BackupObject,
    ) -> Result<(), BackupProviderError>;
}

pub(crate) const MAX_BACKUP_OBJECT_BYTES: usize = 128 * 1024 * 1024;
pub(crate) const MAX_LIST_OBJECTS: usize = 500;
const MAX_METADATA_BYTES: u64 = 1024 * 1024;

pub(crate) fn validate_name(name: &str) -> Result<(), BackupProviderError> {
    if name.is_empty()
        || name.len() > 128
        || name == "."
        || name == ".."
        || name.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || matches!(character, ' ' | '.' | '_' | '-'))
        })
    {
        Err(BackupProviderError::UnsafeName)
    } else {
        Ok(())
    }
}

pub(crate) fn secure_base(
    value: &str,
    policy: EndpointPolicy,
) -> Result<reqwest::Url, BackupProviderError> {
    let mut url =
        reqwest::Url::parse(value).map_err(|_| BackupProviderError::InvalidConfiguration)?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    });
    if !url.username().is_empty()
        || url.password().is_some()
        || (url.scheme() != "https"
            && !(policy == EndpointPolicy::AllowLoopbackHttpForTests && loopback))
    {
        return Err(BackupProviderError::InsecureEndpoint);
    }
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

pub(crate) fn checked(status: reqwest::StatusCode) -> Result<(), BackupProviderError> {
    match status {
        value if value.is_success() => Ok(()),
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            Err(BackupProviderError::AuthorizationRequired)
        }
        value => Err(BackupProviderError::Http(value)),
    }
}

pub(crate) async fn bounded_bytes(
    response: reqwest::Response,
) -> Result<Vec<u8>, BackupProviderError> {
    checked(response.status())?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BACKUP_OBJECT_BYTES as u64)
    {
        return Err(BackupProviderError::TooLarge);
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| BackupProviderError::Transport)?;
    if bytes.len() > MAX_BACKUP_OBJECT_BYTES {
        return Err(BackupProviderError::TooLarge);
    }
    Ok(bytes.to_vec())
}

pub(crate) async fn bounded_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, BackupProviderError> {
    checked(response.status())?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_METADATA_BYTES)
    {
        return Err(BackupProviderError::ResponseTooLarge);
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| BackupProviderError::Transport)?;
    if bytes.len() as u64 > MAX_METADATA_BYTES {
        return Err(BackupProviderError::ResponseTooLarge);
    }
    serde_json::from_slice(&bytes).map_err(|_| BackupProviderError::MalformedResponse)
}
