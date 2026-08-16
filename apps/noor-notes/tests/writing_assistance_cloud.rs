use noor_notes::writing_assistance::{
    CloudAssistanceClient, CloudError, ProviderConfiguration, paragraph_scope, sentence_scope,
};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn configuration(server: &MockServer) -> ProviderConfiguration {
    ProviderConfiguration {
        base_url: server.uri(),
        model: "test-model".into(),
        provider_validated: true,
        validated_base_url: server.uri(),
        validated_model: "test-model".into(),
    }
}

fn response(content: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{ "message": { "content": content.to_string() } }]
    }))
}

fn user_text(request: &wiremock::Request) -> String {
    let body: Value = serde_json::from_slice(&request.body).unwrap();
    body["messages"][1]["content"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn grammar_sends_only_the_capped_current_paragraph() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(response(json!({ "issues": [] })))
        .mount(&server)
        .await;
    let client =
        CloudAssistanceClient::new(configuration(&server), Some(b"secret".to_vec())).unwrap();
    let note = format!("title-like first paragraph\n\n{}", "x".repeat(2_300));

    let issues = client
        .check_grammar(&note, note.chars().count(), Some("en"))
        .await
        .unwrap();

    assert!(issues.is_empty());
    let request = server.received_requests().await.unwrap().pop().unwrap();
    let sent = user_text(&request);
    assert!(!sent.contains("title-like first paragraph"));
    assert!(sent.chars().count() <= 2_000);
    assert_eq!(
        request
            .headers
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer secret"
    );
    assert!(!String::from_utf8(request.body).unwrap().contains("secret"));
}

#[tokio::test]
async fn prediction_uses_sentence_scope_and_sanitizes_suggestions() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(response(json!({
            "suggestions": [" helps", "HELPS", "works", "grows", "bad\nvalue"]
        })))
        .mount(&server)
        .await;
    let mut config = configuration(&server);
    config.base_url = format!("{}/api", server.uri());
    config.validated_base_url = config.base_url.clone();
    let client = CloudAssistanceClient::new(config, None).unwrap();
    let text = format!("unrelated. {} current sentence", "x".repeat(900));

    let suggestions = client.predict(&text, text.chars().count()).await.unwrap();

    assert_eq!(suggestions, vec!["helps", "works", "grows"]);
    let request = server.received_requests().await.unwrap().pop().unwrap();
    assert!(user_text(&request).chars().count() <= 800);
}

#[tokio::test]
async fn grammar_offsets_are_validated_and_shifted_to_the_document() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(response(json!({
            "issues": [
                {"start": 0, "end": 4, "category": "grammar", "message": "Fix", "replacements": ["This"]},
                {"start": 0, "end": 9999, "category": "bad", "message": "Bad", "replacements": ["x"]},
                {"start": 0, "end": 1, "category": "bad", "message": "Bad", "replacements": ["x\n"]}
            ]
        })))
        .mount(&server)
        .await;
    let client = CloudAssistanceClient::new(configuration(&server), None).unwrap();
    let text = "বাংলা paragraph\n\nThis are text.";
    let base = text.chars().position(|character| character == 'T').unwrap();

    let issues = client
        .check_grammar(text, base + 2, Some("en"))
        .await
        .unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].range, base..base + 4);
    assert_eq!(issues[0].replacements, vec!["This"]);
}

#[test]
fn unicode_scopes_return_character_offsets_and_caps() {
    let paragraph = format!("আগে\n\n{}", "আ".repeat(2_100));
    let scoped = paragraph_scope(&paragraph, paragraph.chars().count(), 2_000);
    assert_eq!(scoped.text.chars().count(), 2_000);
    assert_eq!(scoped.base, paragraph.chars().count() - 2_000);

    let sentence = format!("আগে। {}", "খ".repeat(900));
    let scoped = sentence_scope(&sentence, sentence.chars().count(), 800);
    assert_eq!(scoped.text.chars().count(), 800);
    assert_eq!(scoped.base, sentence.chars().count() - 800);
}

#[tokio::test]
async fn connection_and_provider_failures_are_non_content_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).set_body_string("private provider body"))
        .mount(&server)
        .await;
    let client = CloudAssistanceClient::new(configuration(&server), None).unwrap();

    let error = client.test_connection().await.unwrap_err();
    assert!(matches!(error, CloudError::RateLimited));
    assert!(!error.to_string().contains("private provider body"));
}

#[tokio::test]
async fn malformed_outer_or_inner_json_is_reported_without_payloads() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let client = CloudAssistanceClient::new(configuration(&server), None).unwrap();
    assert!(matches!(
        client.predict("text", 4).await,
        Err(CloudError::InvalidResponse)
    ));
}
