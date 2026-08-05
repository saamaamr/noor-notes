use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{StatusCode, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
const MAX_REVISIONS: usize = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointPolicy {
    Production,
    AllowLoopbackHttpForTests,
}

use crate::{AuthSession, RemoteRevision};

#[derive(Debug, thiserror::Error)]
pub enum SyncClientError {
    #[error("cloud authentication is required")]
    AuthRequired,
    #[error("cloud service rate limited the request; retry after {0:?}")]
    RateLimited(Duration),
    #[error("cloud response was malformed")]
    MalformedResponse,
    #[error("cloud request failed: {0}")]
    Transport(String),
    #[error("cloud service returned HTTP {0}")]
    Http(StatusCode),
    #[error("invalid Supabase URL")]
    InvalidUrl,
    #[error("cloud endpoint must use HTTPS")]
    InsecureUrl,
    #[error("cloud response exceeded the security limit")]
    ResponseTooLarge,
}

#[derive(Clone)]
pub struct SupabaseClient {
    http: reqwest::Client,
    base_url: Url,
    anon_key: String,
}

impl SupabaseClient {
    pub fn new(
        base_url: &str,
        anon_key: &str,
        policy: EndpointPolicy,
    ) -> Result<Self, SyncClientError> {
        let mut base_url = Url::parse(base_url).map_err(|_| SyncClientError::InvalidUrl)?;
        if !base_url.username().is_empty()
            || base_url.password().is_some()
            || base_url.host().is_none()
        {
            return Err(SyncClientError::InvalidUrl);
        }
        let loopback = base_url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        if base_url.scheme() != "https"
            && !(policy == EndpointPolicy::AllowLoopbackHttpForTests && loopback)
        {
            return Err(SyncClientError::InsecureUrl);
        }
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(redacted_transport)?;
        Ok(Self {
            http,
            base_url,
            anon_key: anon_key.into(),
        })
    }

    pub async fn sign_in(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthSession, SyncClientError> {
        #[derive(Serialize)]
        struct Credentials<'a> {
            email: &'a str,
            password: &'a str,
        }
        let url = self
            .base_url
            .join("auth/v1/token?grant_type=password")
            .map_err(|_| SyncClientError::InvalidUrl)?;
        let response = self
            .http
            .post(url)
            .header("apikey", &self.anon_key)
            .json(&Credentials { email, password })
            .send()
            .await
            .map_err(redacted_transport)?;
        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(SyncClientError::AuthRequired);
        }
        if !response.status().is_success() {
            return Err(SyncClientError::Http(response.status()));
        }
        bounded_json(response).await
    }

    pub async fn upload_revision(
        &self,
        access_token: &str,
        revision: &RemoteRevision,
    ) -> Result<(), SyncClientError> {
        let url = self
            .base_url
            .join("rest/v1/encrypted_note_revisions")
            .map_err(|_| SyncClientError::InvalidUrl)?;
        let response = self
            .http
            .post(url)
            .header("apikey", &self.anon_key)
            .bearer_auth(access_token)
            .header("Prefer", "resolution=ignore-duplicates,return=minimal")
            .json(revision)
            .send()
            .await
            .map_err(redacted_transport)?;
        match response.status() {
            status if status.is_success() || status == StatusCode::CONFLICT => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(SyncClientError::AuthRequired),
            StatusCode::TOO_MANY_REQUESTS => Err(SyncClientError::RateLimited(retry_after(
                response.headers(),
            ))),
            status => Err(SyncClientError::Http(status)),
        }
    }

    pub async fn upload_tombstone(
        &self,
        access_token: &str,
        revision: &RemoteRevision,
    ) -> Result<(), SyncClientError> {
        self.upload_revision(access_token, revision).await
    }

    pub async fn list_changes(
        &self,
        access_token: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<RemoteRevision>, SyncClientError> {
        let mut url = self
            .base_url
            .join("rest/v1/encrypted_note_revisions")
            .map_err(|_| SyncClientError::InvalidUrl)?;
        url.query_pairs_mut()
            .append_pair(
                "select",
                "note_id,revision,ciphertext,nonce,updated_at,deleted_at",
            )
            .append_pair("updated_at", &format!("gt.{}", since.to_rfc3339()))
            .append_pair("order", "updated_at.asc");
        let response = self
            .http
            .get(url)
            .header("apikey", &self.anon_key)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(redacted_transport)?;
        if response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
        {
            return Err(SyncClientError::AuthRequired);
        }
        if !response.status().is_success() {
            return Err(SyncClientError::Http(response.status()));
        }
        let revisions: Vec<RemoteRevision> = bounded_json(response).await?;
        if revisions.len() > MAX_REVISIONS {
            return Err(SyncClientError::ResponseTooLarge);
        }
        Ok(revisions)
    }
}

async fn bounded_json<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, SyncClientError> {
    if response
        .content_length()
        .is_some_and(|size| size > MAX_RESPONSE_BYTES)
    {
        return Err(SyncClientError::ResponseTooLarge);
    }
    let bytes = response.bytes().await.map_err(redacted_transport)?;
    if bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(SyncClientError::ResponseTooLarge);
    }
    serde_json::from_slice(&bytes).map_err(|_| SyncClientError::MalformedResponse)
}

fn retry_after(headers: &reqwest::header::HeaderMap) -> Duration {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(2))
}

fn redacted_transport(error: reqwest::Error) -> SyncClientError {
    SyncClientError::Transport(
        error
            .status()
            .map(|status| format!("HTTP {status}"))
            .unwrap_or_else(|| "connection unavailable".into()),
    )
}
