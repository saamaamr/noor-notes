use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::backup_provider::{
    MAX_BACKUP_OBJECT_BYTES, MAX_LIST_OBJECTS, bounded_bytes, bounded_json, checked, secure_base,
    validate_name,
};
use crate::{BackupObject, BackupProvider, BackupProviderError, EndpointPolicy};

#[derive(Clone)]
pub struct GoogleDriveProvider {
    http: reqwest::Client,
    api_base: reqwest::Url,
    upload_base: reqwest::Url,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleFile {
    id: String,
    name: String,
    modified_time: DateTime<Utc>,
    #[serde(deserialize_with = "string_u64")]
    size: u64,
}

#[derive(Deserialize)]
struct GoogleList {
    #[serde(default)]
    files: Vec<GoogleFile>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Serialize)]
struct Metadata<'a> {
    name: &'a str,
    parents: [&'static str; 1],
    #[serde(rename = "appProperties")]
    app_properties: std::collections::HashMap<&'static str, &'static str>,
}

impl GoogleDriveProvider {
    pub fn new() -> Result<Self, BackupProviderError> {
        Self::with_endpoints(
            "https://www.googleapis.com/drive/v3/",
            "https://www.googleapis.com/upload/drive/v3/",
            EndpointPolicy::Production,
        )
    }

    #[doc(hidden)]
    pub fn for_test(base: &str) -> Result<Self, BackupProviderError> {
        Self::with_endpoints(base, base, EndpointPolicy::AllowLoopbackHttpForTests)
    }

    fn with_endpoints(
        api: &str,
        upload: &str,
        policy: EndpointPolicy,
    ) -> Result<Self, BackupProviderError> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|_| BackupProviderError::Transport)?;
        Ok(Self {
            http,
            api_base: secure_base(api, policy)?,
            upload_base: secure_base(upload, policy)?,
        })
    }

    async fn upload_media(
        &self,
        access_token: &str,
        id: &str,
        encrypted: &[u8],
    ) -> Result<GoogleFile, BackupProviderError> {
        let mut url = self
            .upload_base
            .join(&format!("files/{id}"))
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        url.query_pairs_mut()
            .append_pair("uploadType", "media")
            .append_pair("fields", "id,name,modifiedTime,size");
        let response = self
            .http
            .patch(url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/octet-stream")
            .body(encrypted.to_vec())
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        parse_file(response).await
    }

    async fn create_file(
        &self,
        access_token: &str,
        name: &str,
    ) -> Result<GoogleFile, BackupProviderError> {
        let mut url = self
            .api_base
            .join("files")
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        url.query_pairs_mut()
            .append_pair("fields", "id,name,modifiedTime,size");
        let mut app_properties = std::collections::HashMap::new();
        app_properties.insert("noor-notes-backup", "1");
        let response = self
            .http
            .post(url)
            .bearer_auth(access_token)
            .json(&Metadata {
                name,
                parents: ["appDataFolder"],
                app_properties,
            })
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        parse_file(response).await
    }
}

impl BackupProvider for GoogleDriveProvider {
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
        let temporary_name = format!("{name}.upload");
        let temporary = self.create_file(access_token, &temporary_name).await?;
        self.upload_media(access_token, &temporary.id, encrypted)
            .await?;
        let existing = self
            .list(access_token)
            .await?
            .into_iter()
            .find(|object| object.name == name);
        let current = if let Some(existing) = existing {
            self.upload_media(access_token, &existing.id, encrypted)
                .await?
        } else {
            let current = self.create_file(access_token, name).await?;
            self.upload_media(access_token, &current.id, encrypted)
                .await?
        };
        let temporary = BackupObject::from(temporary);
        self.delete(access_token, &temporary).await?;
        Ok(current.into())
    }

    async fn list(&self, access_token: &str) -> Result<Vec<BackupObject>, BackupProviderError> {
        let mut url = self
            .api_base
            .join("files")
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        url.query_pairs_mut()
            .append_pair("spaces", "appDataFolder")
            .append_pair(
                "q",
                "appProperties has { key='noor-notes-backup' and value='1' }",
            )
            .append_pair("fields", "nextPageToken,files(id,name,modifiedTime,size)")
            .append_pair("pageSize", "500");
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| BackupProviderError::Transport)?;
        let list: GoogleList = bounded_json(response).await?;
        if list.next_page_token.is_some() || list.files.len() > MAX_LIST_OBJECTS {
            return Err(BackupProviderError::ResponseTooLarge);
        }
        Ok(list.files.into_iter().map(Into::into).collect())
    }

    async fn download(
        &self,
        access_token: &str,
        object: &BackupObject,
    ) -> Result<Vec<u8>, BackupProviderError> {
        if object.size > MAX_BACKUP_OBJECT_BYTES as u64 {
            return Err(BackupProviderError::TooLarge);
        }
        let mut url = self
            .api_base
            .join(&format!("files/{}", object.id))
            .map_err(|_| BackupProviderError::InvalidConfiguration)?;
        url.query_pairs_mut().append_pair("alt", "media");
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
            .api_base
            .join(&format!("files/{}", object.id))
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

impl From<GoogleFile> for BackupObject {
    fn from(value: GoogleFile) -> Self {
        Self {
            id: value.id,
            name: value.name,
            modified_at: value.modified_time,
            size: value.size,
        }
    }
}

async fn parse_file(response: reqwest::Response) -> Result<GoogleFile, BackupProviderError> {
    bounded_json(response).await
}

fn string_u64<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    use serde::Deserialize;
    String::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}
