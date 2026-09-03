use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::{RngCore, rngs::OsRng};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::{EndpointPolicy, OAuthPkce};

const MAX_TOKEN_BYTES: u64 = 64 * 1024;
const GOOGLE_SCOPE: &str = "https://www.googleapis.com/auth/drive.appdata";
const ONEDRIVE_SCOPE: &str = "offline_access Files.ReadWrite.AppFolder";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackupProviderKind {
    GoogleDrive,
    OneDrive,
}

#[derive(Clone)]
pub struct ProviderOAuth {
    http: reqwest::Client,
    kind: BackupProviderKind,
    client_id: String,
    authorization_endpoint: Url,
    token_endpoint: Url,
    revoke_endpoint: Option<Url>,
    redirect_uri: Url,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSession {
    pub access_token: Zeroizing<String>,
    pub refresh_token: Zeroizing<String>,
    pub expires_in: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderOAuthError {
    #[error("invalid provider OAuth configuration")]
    InvalidConfiguration,
    #[error("provider OAuth endpoint must use HTTPS")]
    InsecureEndpoint,
    #[error("provider authorization is required")]
    AuthorizationRequired,
    #[error("provider OAuth response was malformed")]
    MalformedResponse,
    #[error("provider OAuth response exceeded the security limit")]
    ResponseTooLarge,
    #[error("provider OAuth request failed")]
    Transport,
    #[error("provider OAuth service returned HTTP {0}")]
    Http(StatusCode),
    #[error("this provider does not offer token revocation for public clients")]
    RevocationUnsupported,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: u64,
}

impl ProviderOAuth {
    pub fn google(client_id: impl Into<String>) -> Result<Self, ProviderOAuthError> {
        Self::new(
            BackupProviderKind::GoogleDrive,
            client_id.into(),
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            Some("https://oauth2.googleapis.com/revoke"),
            "http://127.0.0.1:43818/backup/google",
            EndpointPolicy::Production,
        )
    }

    pub fn onedrive(client_id: impl Into<String>) -> Result<Self, ProviderOAuthError> {
        Self::new(
            BackupProviderKind::OneDrive,
            client_id.into(),
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            None,
            "http://127.0.0.1:43819/backup/onedrive",
            EndpointPolicy::Production,
        )
    }

    #[doc(hidden)]
    pub fn for_test(
        kind: BackupProviderKind,
        client_id: impl Into<String>,
        authorization_endpoint: &str,
        token_endpoint: &str,
        revoke_endpoint: Option<&str>,
        redirect_uri: &str,
    ) -> Result<Self, ProviderOAuthError> {
        Self::new(
            kind,
            client_id.into(),
            authorization_endpoint,
            token_endpoint,
            revoke_endpoint,
            redirect_uri,
            EndpointPolicy::AllowLoopbackHttpForTests,
        )
    }

    fn new(
        kind: BackupProviderKind,
        client_id: String,
        authorization_endpoint: &str,
        token_endpoint: &str,
        revoke_endpoint: Option<&str>,
        redirect_uri: &str,
        policy: EndpointPolicy,
    ) -> Result<Self, ProviderOAuthError> {
        if client_id.trim().is_empty() || client_id.len() > 512 {
            return Err(ProviderOAuthError::InvalidConfiguration);
        }
        let authorization_endpoint = secure_url(authorization_endpoint, policy)?;
        let token_endpoint = secure_url(token_endpoint, policy)?;
        let revoke_endpoint = revoke_endpoint
            .map(|value| secure_url(value, policy))
            .transpose()?;
        let redirect_uri =
            Url::parse(redirect_uri).map_err(|_| ProviderOAuthError::InvalidConfiguration)?;
        if redirect_uri.scheme() != "http"
            || redirect_uri.host_str() != Some("127.0.0.1")
            || redirect_uri.port().is_none()
        {
            return Err(ProviderOAuthError::InvalidConfiguration);
        }
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| ProviderOAuthError::Transport)?;
        Ok(Self {
            http,
            kind,
            client_id,
            authorization_endpoint,
            token_endpoint,
            revoke_endpoint,
            redirect_uri,
        })
    }

    pub fn kind(&self) -> BackupProviderKind {
        self.kind
    }

    pub fn redirect_uri(&self) -> &Url {
        &self.redirect_uri
    }

    pub fn authorization(&self) -> OAuthPkce {
        let mut verifier_bytes = [0_u8; 32];
        let mut state_bytes = [0_u8; 24];
        OsRng.fill_bytes(&mut verifier_bytes);
        OsRng.fill_bytes(&mut state_bytes);
        let verifier = Zeroizing::new(URL_SAFE_NO_PAD.encode(verifier_bytes));
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let state = URL_SAFE_NO_PAD.encode(state_bytes);
        let mut authorization_url = self.authorization_endpoint.clone();
        authorization_url
            .query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", self.redirect_uri.as_str())
            .append_pair("response_type", "code")
            .append_pair("scope", self.scope())
            .append_pair("state", &state)
            .append_pair("code_challenge", &challenge)
            .append_pair("code_challenge_method", "S256");
        if self.kind == BackupProviderKind::GoogleDrive {
            authorization_url
                .query_pairs_mut()
                .append_pair("access_type", "offline")
                .append_pair("prompt", "consent");
        }
        OAuthPkce {
            authorization_url,
            verifier,
            state,
        }
    }

    pub async fn exchange(
        &self,
        code: &str,
        verifier: &str,
    ) -> Result<ProviderSession, ProviderOAuthError> {
        let values = [
            ("client_id", self.client_id.as_str()),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];
        self.token(&values, None).await
    }

    pub async fn refresh(
        &self,
        refresh_token: &str,
    ) -> Result<ProviderSession, ProviderOAuthError> {
        let values = [
            ("client_id", self.client_id.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];
        self.token(&values, Some(refresh_token)).await
    }

    pub async fn revoke(&self, token: &str) -> Result<(), ProviderOAuthError> {
        let endpoint = self
            .revoke_endpoint
            .clone()
            .ok_or(ProviderOAuthError::RevocationUnsupported)?;
        let response = self
            .http
            .post(endpoint)
            .form(&[("token", token)])
            .send()
            .await
            .map_err(|_| ProviderOAuthError::Transport)?;
        checked(response.status())
    }

    async fn token(
        &self,
        values: &[(&str, &str)],
        existing_refresh: Option<&str>,
    ) -> Result<ProviderSession, ProviderOAuthError> {
        let response = self
            .http
            .post(self.token_endpoint.clone())
            .form(values)
            .send()
            .await
            .map_err(|_| ProviderOAuthError::Transport)?;
        checked(response.status())?;
        if response
            .content_length()
            .is_some_and(|size| size > MAX_TOKEN_BYTES)
        {
            return Err(ProviderOAuthError::ResponseTooLarge);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|_| ProviderOAuthError::Transport)?;
        if bytes.len() as u64 > MAX_TOKEN_BYTES {
            return Err(ProviderOAuthError::ResponseTooLarge);
        }
        let token: TokenResponse =
            serde_json::from_slice(&bytes).map_err(|_| ProviderOAuthError::MalformedResponse)?;
        if token.access_token.is_empty() || token.expires_in == 0 {
            return Err(ProviderOAuthError::MalformedResponse);
        }
        let refresh_token = token
            .refresh_token
            .or_else(|| existing_refresh.map(str::to_owned))
            .filter(|value| !value.is_empty())
            .ok_or(ProviderOAuthError::MalformedResponse)?;
        Ok(ProviderSession {
            access_token: Zeroizing::new(token.access_token),
            refresh_token: Zeroizing::new(refresh_token),
            expires_in: token.expires_in,
        })
    }

    fn scope(&self) -> &'static str {
        match self.kind {
            BackupProviderKind::GoogleDrive => GOOGLE_SCOPE,
            BackupProviderKind::OneDrive => ONEDRIVE_SCOPE,
        }
    }
}

fn secure_url(value: &str, policy: EndpointPolicy) -> Result<Url, ProviderOAuthError> {
    let url = Url::parse(value).map_err(|_| ProviderOAuthError::InvalidConfiguration)?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    });
    if url.username().is_empty()
        && url.password().is_none()
        && (url.scheme() == "https"
            || (policy == EndpointPolicy::AllowLoopbackHttpForTests && loopback))
    {
        Ok(url)
    } else {
        Err(ProviderOAuthError::InsecureEndpoint)
    }
}

fn checked(status: StatusCode) -> Result<(), ProviderOAuthError> {
    match status {
        value if value.is_success() => Ok(()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            Err(ProviderOAuthError::AuthorizationRequired)
        }
        value => Err(ProviderOAuthError::Http(value)),
    }
}
