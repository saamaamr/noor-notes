use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::backup_provider::{
    MAX_BACKUP_OBJECT_BYTES, MAX_LIST_OBJECTS, bounded_bytes, checked, secure_base, validate_name,
};
use crate::{BackupObject, BackupProvider, BackupProviderError, EndpointPolicy};

#[derive(Clone)]
pub struct OneDriveProvider {
    http: reqwest::Client,
    graph_base: reqwest::Url,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveItem {
    id: String,
    name: String,
    last_modified_date_time: DateTime<Utc>,
    size: u64,
}

#[derive(Deserialize)]
struct DriveList {
    #[serde(default)]
    value: Vec<DriveItem>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

impl OneDriveProvider {
    pub fn new() -> Result<Self, BackupProviderError> {
        Self::with_endpoint(
            "https://graph.microsoft.com/v1.0/",
            EndpointPolicy::Production,
        )
    }

    #[doc(hidden)]
    pub fn for_test(base: &str) -> Result<Self, BackupProviderError> {
        Self::with_endpoint(base, EndpointPolicy::AllowLoopbackHttpForTests)
    }

    fn with_endpoint(base: &str, policy: EndpointPolicy) -> Result<Self, BackupProviderError> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|_| BackupProviderError::Transport)?;
        Ok(Self {
            http,
            graph_base: secure_base(base, policy)?,
        })
    }
}

impl BackupProvider for OneDriveProvider {
    async fn upload(
        &self,
        access_token: &str,
        name: &str,
        encrypted: &[u8],
    ) -> Result<BackupObject, BackupProviderError> {
        validate_name(name)?;
        if encrypted.len() > MAX_BACKUP_OBJECT_BYTES {
            return Err(BackupProviderError::TooLarge);
        }
        let path = format!("me/drive/special/approot:/Noor Notes/{name}:/content");
        let url = self
            .graph_base
            .join(&path)
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        let response = self
            .http
            .put(url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/octet-stream")
            .body(encrypted.to_vec())
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        checked(response.status())?;
        response
            .json::<DriveItem>()
            .await
            .map(Into::into)
            .map_err(|_| BackupProviderError::MalformedResponse)
    }

    async fn list(&self, access_token: &str) -> Result<Vec<BackupObject>, BackupProviderError> {
        let url = self
            .graph_base
            .join("me/drive/special/approot:/Noor Notes:/children")
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        checked(response.status())?;
        let list: DriveList = response
            .json()
            .await
            .map_err(|_| BackupProviderError::MalformedResponse)?;
        if list.next_link.is_some() || list.value.len() > MAX_LIST_OBJECTS {
            return Err(BackupProviderError::ResponseTooLarge);
        }
        Ok(list.value.into_iter().map(Into::into).collect())
    }

    async fn download(
        &self,
        access_token: &str,
        object: &BackupObject,
    ) -> Result<Vec<u8>, BackupProviderError> {
        if object.size > MAX_BACKUP_OBJECT_BYTES as u64 {
            return Err(BackupProviderError::TooLarge);
        }
        let url = self
            .graph_base
            .join(&format!("me/drive/items/{}/content", object.id))
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        bounded_bytes(response).await
    }

    async fn delete(
        &self,
        access_token: &str,
        object: &BackupObject,
    ) -> Result<(), BackupProviderError> {
        let url = self
            .graph_base
            .join(&format!("me/drive/items/{}", object.id))
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        let response = self
            .http
            .delete(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        checked(response.status())
    }
}

impl From<DriveItem> for BackupObject {
    fn from(value: DriveItem) -> Self {
        Self {
            id: value.id,
            name: value.name,
            modified_at: value.last_modified_date_time,
            size: value.size,
        }
    }
}
