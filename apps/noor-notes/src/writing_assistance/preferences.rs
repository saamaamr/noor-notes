use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::net::IpAddr;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use noor_domain::WritingAssistanceOverrides;
use serde::{Deserialize, Serialize};
use url::{Host, Url};

const MAX_ENDPOINT_LEN: usize = 2_048;
const MAX_SHORT_VALUE_LEN: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfiguration {
    pub base_url: String,
    pub model: String,
    pub provider_validated: bool,
    pub validated_base_url: String,
    pub validated_model: String,
}

impl Default for ProviderConfiguration {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            model: String::new(),
            provider_validated: false,
            validated_base_url: String::new(),
            validated_model: String::new(),
        }
    }
}

impl ProviderConfiguration {
    pub fn is_validated(&self) -> bool {
        self.provider_validated
            && self.base_url == self.validated_base_url
            && self.model == self.validated_model
            && self.is_well_formed()
    }

    fn is_well_formed(&self) -> bool {
        validate_provider_endpoint(&self.base_url).is_ok() && valid_short_value(&self.model)
    }

    fn clear_validation(&mut self) {
        self.provider_validated = false;
        self.validated_base_url.clear();
        self.validated_model.clear();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WritingAssistancePreferences {
    pub spelling: bool,
    pub grammar: bool,
    pub offline_prediction: bool,
    pub cloud_enabled: bool,
    pub language: String,
    pub provider: ProviderConfiguration,
}

impl Default for WritingAssistancePreferences {
    fn default() -> Self {
        Self {
            spelling: true,
            grammar: true,
            offline_prediction: true,
            cloud_enabled: false,
            language: "auto".into(),
            provider: ProviderConfiguration::default(),
        }
    }
}

impl WritingAssistancePreferences {
    pub fn resolve(&self, overrides: &WritingAssistanceOverrides) -> ResolvedWritingAssistance {
        let cloud_requested = overrides.cloud.unwrap_or(self.cloud_enabled);
        ResolvedWritingAssistance {
            spelling: overrides.spelling.unwrap_or(self.spelling),
            grammar: overrides.grammar.unwrap_or(self.grammar),
            offline_prediction: overrides
                .offline_prediction
                .unwrap_or(self.offline_prediction),
            cloud: cloud_requested && self.provider.is_validated(),
        }
    }

    pub fn update_provider(&mut self, base_url: &str, model: &str) {
        let base_url = base_url.trim().to_owned();
        let model = model.trim().to_owned();
        if self.provider.base_url != base_url || self.provider.model != model {
            self.provider.base_url = base_url;
            self.provider.model = model;
            self.provider.clear_validation();
            self.cloud_enabled = false;
        }
    }

    pub fn mark_provider_validated(&mut self) {
        if self.provider.is_well_formed() {
            self.provider.provider_validated = true;
            self.provider.validated_base_url = self.provider.base_url.clone();
            self.provider.validated_model = self.provider.model.clone();
        }
    }

    fn is_safe(&self) -> bool {
        valid_short_value(&self.language)
            && (!self.cloud_enabled || self.provider.is_validated())
            && (self.provider == ProviderConfiguration::default()
                || (self.provider.is_well_formed()
                    && (!self.provider.provider_validated || self.provider.is_validated())))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedWritingAssistance {
    pub spelling: bool,
    pub grammar: bool,
    pub offline_prediction: bool,
    pub cloud: bool,
}

#[derive(Clone, Debug)]
pub struct WritingAssistanceStore {
    path: PathBuf,
}

impl WritingAssistanceStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn for_current_user() -> Self {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        Self::at(base.join("noor-notes/writing-assistance.json"))
    }

    pub fn load(&self) -> WritingAssistancePreferences {
        fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<WritingAssistancePreferences>(&bytes).ok())
            .filter(WritingAssistancePreferences::is_safe)
            .unwrap_or_default()
    }

    pub fn save(&self, preferences: &WritingAssistancePreferences) -> io::Result<()> {
        if !preferences.is_safe() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsafe writing assistance preferences",
            ));
        }
        let Some(parent) = self.path.parent() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "missing parent",
            ));
        };
        fs::create_dir_all(parent)?;
        let temporary = self.path.with_extension("json.tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)?;
        serde_json::to_writer(&mut file, preferences).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        OpenOptions::new().read(true).open(parent)?.sync_all()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PreferenceError {
    #[error("provider endpoint is invalid")]
    InvalidEndpoint,
    #[error("remote provider endpoints require HTTPS")]
    InsecureRemoteEndpoint,
}

pub fn validate_provider_endpoint(value: &str) -> Result<Url, PreferenceError> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_ENDPOINT_LEN {
        return Err(PreferenceError::InvalidEndpoint);
    }
    let url = Url::parse(value).map_err(|_| PreferenceError::InvalidEndpoint)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.host().is_none()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(PreferenceError::InvalidEndpoint);
    }
    match url.scheme() {
        "https" => Ok(url),
        "http" if is_loopback(&url) => Ok(url),
        "http" => Err(PreferenceError::InsecureRemoteEndpoint),
        _ => Err(PreferenceError::InvalidEndpoint),
    }
}

pub fn provider_requires_api_key(url: &Url) -> bool {
    !is_loopback(url)
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => IpAddr::V4(address).is_loopback(),
        Some(Host::Ipv6(address)) => IpAddr::V6(address).is_loopback(),
        None => false,
    }
}

fn valid_short_value(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && value.chars().count() <= MAX_SHORT_VALUE_LEN
}
