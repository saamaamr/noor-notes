use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use super::{
    AssistanceIssue, IssueSource, ProviderConfiguration, provider_requires_api_key,
    validate_provider_endpoint,
};

const GRAMMAR_SCOPE_LIMIT: usize = 2_000;
const PREDICTION_SCOPE_LIMIT: usize = 800;
const MAX_REPLACEMENT_LEN: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextScope {
    pub text: String,
    pub base: usize,
}

#[derive(Clone)]
pub struct CloudAssistanceClient {
    client: Client,
    endpoint: Url,
    model: String,
    api_key: Option<Arc<Zeroizing<Vec<u8>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CloudError {
    #[error("provider configuration is invalid")]
    InvalidConfiguration,
    #[error("the remote provider requires an API key")]
    MissingApiKey,
    #[error("the provider request timed out")]
    Timeout,
    #[error("the provider is temporarily unavailable")]
    Unavailable,
    #[error("the provider rate limit was reached")]
    RateLimited,
    #[error("the provider rejected the request")]
    RequestRejected,
    #[error("the provider returned an invalid response")]
    InvalidResponse,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    temperature: f32,
    messages: [ChatMessage<'a>; 2],
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct GrammarPayload {
    issues: Vec<CloudIssue>,
}

#[derive(Deserialize)]
struct CloudIssue {
    start: usize,
    end: usize,
    category: String,
    message: String,
    replacements: Vec<String>,
}

#[derive(Deserialize)]
struct PredictionPayload {
    suggestions: Vec<String>,
}

impl CloudAssistanceClient {
    pub fn new(
        configuration: ProviderConfiguration,
        api_key: Option<Vec<u8>>,
    ) -> Result<Self, CloudError> {
        let base = validate_provider_endpoint(&configuration.base_url)
            .map_err(|_| CloudError::InvalidConfiguration)?;
        if configuration.model.trim().is_empty() || configuration.model.chars().count() > 128 {
            return Err(CloudError::InvalidConfiguration);
        }
        if provider_requires_api_key(&base) && api_key.as_ref().is_none_or(Vec::is_empty) {
            return Err(CloudError::MissingApiKey);
        }
        let endpoint = chat_endpoint(base)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| CloudError::InvalidConfiguration)?;
        Ok(Self {
            client,
            endpoint,
            model: configuration.model.trim().to_owned(),
            api_key: api_key.map(|key| Arc::new(Zeroizing::new(key))),
        })
    }

    pub async fn test_connection(&self) -> Result<(), CloudError> {
        self.send(
            "Return an empty JSON object. Do not echo input.",
            "connection test",
        )
        .await
        .map(|_| ())
    }

    pub async fn check_grammar(
        &self,
        text: &str,
        cursor: usize,
        language: Option<&str>,
    ) -> Result<Vec<AssistanceIssue>, CloudError> {
        let scope = paragraph_scope(text, cursor, GRAMMAR_SCOPE_LIMIT);
        if scope.text.is_empty() {
            return Ok(Vec::new());
        }
        let language = language.unwrap_or("auto");
        let system = format!(
            "Check grammar in {language}. Return only JSON {{\"issues\":[{{\"start\":0,\"end\":1,\"category\":\"grammar\",\"message\":\"...\",\"replacements\":[\"...\"]}}]}}. Offsets are Unicode character indexes."
        );
        let content = self.send(&system, &scope.text).await?;
        let payload: GrammarPayload =
            serde_json::from_str(&content).map_err(|_| CloudError::InvalidResponse)?;
        Ok(sanitize_issues(
            payload.issues,
            scope.base,
            scope.text.chars().count(),
        ))
    }

    pub async fn predict(&self, text: &str, cursor: usize) -> Result<Vec<String>, CloudError> {
        let scope = sentence_scope(text, cursor, PREDICTION_SCOPE_LIMIT);
        if scope.text.is_empty() {
            return Ok(Vec::new());
        }
        let content = self
            .send(
                "Suggest likely next words. Return only JSON {\"suggestions\":[\"word\"]}, with at most three short suggestions.",
                &scope.text,
            )
            .await?;
        let payload: PredictionPayload =
            serde_json::from_str(&content).map_err(|_| CloudError::InvalidResponse)?;
        Ok(sanitize_suggestions(payload.suggestions))
    }

    async fn send(&self, system: &str, user: &str) -> Result<String, CloudError> {
        let body = ChatRequest {
            model: &self.model,
            temperature: 0.0,
            messages: [
                ChatMessage {
                    role: "system",
                    content: system,
                },
                ChatMessage {
                    role: "user",
                    content: user,
                },
            ],
        };
        let mut request = self.client.post(self.endpoint.clone()).json(&body);
        if let Some(key) = &self.api_key {
            let key = std::str::from_utf8(key.as_slice())
                .map_err(|_| CloudError::InvalidConfiguration)?;
            request = request.bearer_auth(key);
        }
        let response = request.send().await.map_err(map_transport_error)?;
        match response.status() {
            StatusCode::TOO_MANY_REQUESTS => return Err(CloudError::RateLimited),
            status if status.is_server_error() => return Err(CloudError::Unavailable),
            status if !status.is_success() => return Err(CloudError::RequestRejected),
            _ => {}
        }
        let response: ChatResponse = response
            .json()
            .await
            .map_err(|_| CloudError::InvalidResponse)?;
        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or(CloudError::InvalidResponse)
    }
}

pub fn paragraph_scope(text: &str, cursor: usize, maximum: usize) -> TextScope {
    let characters = text.chars().collect::<Vec<_>>();
    let cursor = cursor.min(characters.len());
    let mut start = 0;
    for index in 0..cursor.saturating_sub(1) {
        if characters[index] == '\n' && characters[index + 1] == '\n' {
            start = index + 2;
        }
    }
    let mut end = characters.len();
    for index in cursor..characters.len().saturating_sub(1) {
        if characters[index] == '\n' && characters[index + 1] == '\n' {
            end = index;
            break;
        }
    }
    bounded_scope(&characters, start, end, cursor, maximum)
}

pub fn sentence_scope(text: &str, cursor: usize, maximum: usize) -> TextScope {
    let characters = text.chars().collect::<Vec<_>>();
    let cursor = cursor.min(characters.len());
    let is_boundary = |character: char| matches!(character, '.' | '!' | '?' | '।' | '\n');
    let start = characters[..cursor]
        .iter()
        .rposition(|character| is_boundary(*character))
        .map_or(0, |index| index + 1);
    let end = characters[cursor..]
        .iter()
        .position(|character| is_boundary(*character))
        .map_or(characters.len(), |index| cursor + index + 1);
    bounded_scope(&characters, start, end, cursor, maximum)
}

fn bounded_scope(
    characters: &[char],
    start: usize,
    end: usize,
    cursor: usize,
    maximum: usize,
) -> TextScope {
    if maximum == 0 || start >= end {
        return TextScope {
            text: String::new(),
            base: cursor.min(characters.len()),
        };
    }
    let cursor = cursor.clamp(start, end);
    let mut bounded_start = start;
    let mut bounded_end = end;
    if end - start > maximum {
        bounded_start = cursor.saturating_sub(maximum).max(start);
        bounded_end = (bounded_start + maximum).min(end);
        if bounded_end - bounded_start < maximum {
            bounded_start = bounded_end.saturating_sub(maximum).max(start);
        }
    }
    TextScope {
        text: characters[bounded_start..bounded_end].iter().collect(),
        base: bounded_start,
    }
}

fn chat_endpoint(mut base: Url) -> Result<Url, CloudError> {
    let path = base.path().trim_end_matches('/');
    let normalized = if path.ends_with("/v1/chat/completions") {
        path.to_owned()
    } else if path.ends_with("/v1") {
        format!("{path}/chat/completions")
    } else if path.is_empty() {
        "/v1/chat/completions".to_owned()
    } else {
        format!("{path}/v1/chat/completions")
    };
    base.set_path(&normalized);
    base.set_query(None);
    base.set_fragment(None);
    Ok(base)
}

fn map_transport_error(error: reqwest::Error) -> CloudError {
    if error.is_timeout() {
        CloudError::Timeout
    } else {
        CloudError::Unavailable
    }
}

fn sanitize_issues(issues: Vec<CloudIssue>, base: usize, scope_len: usize) -> Vec<AssistanceIssue> {
    let mut seen = BTreeSet::new();
    issues
        .into_iter()
        .filter(|issue| {
            issue.start < issue.end
                && issue.end <= scope_len
                && valid_text(&issue.category, 128)
                && valid_text(&issue.message, 512)
        })
        .filter_map(|issue| {
            let had_replacements = !issue.replacements.is_empty();
            let replacements = issue
                .replacements
                .into_iter()
                .filter_map(|replacement| clean_replacement(&replacement))
                .take(5)
                .collect::<Vec<_>>();
            if had_replacements && replacements.is_empty() {
                return None;
            }
            let key = (
                issue.start,
                issue.end,
                issue.category.to_lowercase(),
                issue.message.clone(),
            );
            seen.insert(key).then_some(AssistanceIssue {
                range: base + issue.start..base + issue.end,
                category: issue.category,
                message: issue.message,
                replacements,
                source: IssueSource::CloudGrammar,
            })
        })
        .collect()
}

fn sanitize_suggestions(suggestions: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    suggestions
        .into_iter()
        .filter_map(|suggestion| clean_replacement(&suggestion))
        .filter(|suggestion| seen.insert(suggestion.to_lowercase()))
        .take(3)
        .collect()
}

fn clean_replacement(value: &str) -> Option<String> {
    if value.chars().any(char::is_control) {
        return None;
    }
    let value = value.trim();
    valid_text(value, MAX_REPLACEMENT_LEN).then(|| value.to_owned())
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.chars().count() <= maximum && !value.chars().any(char::is_control)
}
