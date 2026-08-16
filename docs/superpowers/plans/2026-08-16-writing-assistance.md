# Writing Assistance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add default-on offline spelling, English grammar, and predictive suggestions to every editable Noor Notes mode, with independent global/per-note switches and an explicitly enabled, privacy-scoped OpenAI-compatible provider.

**Architecture:** Standardize every mode on GtkSourceView, then attach one `WritingAssistanceController` per note window. Keep checking, prediction, provider access, settings, and encrypted derived-model persistence behind focused modules; use character-indexed immutable results and a document generation counter so asynchronous work never writes stale results into the GTK buffer.

**Tech Stack:** Rust 2024 (MSRV 1.85), GTK4/libadwaita/GtkSourceView 5, libspelling 0.4.1 + Enchant/Hunspell, harper-core 2.5, unicode-segmentation, Tokio, reqwest/rustls, serde, oo7 GNOME Keyring, SQLCipher/sqlx, Xvfb integration tests.

## Global Constraints

- Spelling, offline English grammar, and offline prediction are enabled by default; online AI is disabled by default.
- Preserve Rich Text, Markdown, Plain Text, and Code behavior; Code checks and predicts only inside `comment` and `string` GtkSourceView context classes.
- View Only and Trash suppress checks, predictions, and note-level assistance controls without changing stored preferences.
- Cloud grammar sends only the current paragraph capped at 2,000 Unicode characters; cloud prediction sends nearby sentence context capped at 800 Unicode characters.
- Cloud requests never include title, tags, whole-note metadata, complete unrelated paragraphs, other notes, account data, or encryption material.
- Remote providers require HTTPS and an API key; loopback `localhost`, `127.0.0.1`, and `[::1]` providers may use HTTP and omit the key.
- Provider request and response bodies, note text, and API keys must never be logged.
- The API key lives only in GNOME Keyring; global non-secret settings use a private atomic `0600` JSON file; per-note overrides remain in encrypted note metadata.
- Local prediction learns from Active and Archived note bodies, excludes Trash, stays under 50,000 n-gram entries, and is stored as derived data inside the encrypted SQLCipher database.
- All engine offsets are Unicode character offsets. Reject out-of-range, cross-region, control-character, oversized, duplicated, or stale results.
- Assistance tags and ghost text are transient: they must not enter note content, rich snapshots, undo history, autosave, export, search, or character counts.
- Missing dictionaries, unsupported grammar languages, keyring failures, network failures, malformed model data, and engine failures must preserve editing and degrade to a non-modal unavailable/offline status.
- Do not add an animation dependency; use GTK/CSS only and disable cosmetic transitions when reduced motion is requested.

---

### Task 1: Domain Overrides and Effective Settings

**Files:**
- Modify: `crates/domain/src/note.rs`
- Modify: `crates/domain/src/lib.rs`
- Modify: `crates/domain/tests/note_model.rs`

**Interfaces:**
- Produces: `WritingAssistanceOverrides { spelling: Option<bool>, grammar: Option<bool>, offline_prediction: Option<bool>, cloud: Option<bool> }`.
- Produces: `EditorPreferences.writing_assistance: WritingAssistanceOverrides` with serde defaulting.
- Preserves: `Note::duplicate` copies the override while new/legacy notes inherit global settings through `None`.

- [ ] **Step 1: Write failing compatibility and duplication tests**

```rust
#[test]
fn legacy_editor_preferences_default_to_global_writing_settings() {
    let value = serde_json::json!({
        "zoom_percent": 100, "word_wrap": true, "cursor_offset": 0,
        "scroll_offset": 0, "bookmarks": [], "view_only": false
    });
    let preferences: EditorPreferences = serde_json::from_value(value).unwrap();
    assert_eq!(preferences.writing_assistance, WritingAssistanceOverrides::default());
    assert_eq!(preferences.writing_assistance.spelling, None);
    assert_eq!(preferences.writing_assistance.grammar, None);
    assert_eq!(preferences.writing_assistance.offline_prediction, None);
    assert_eq!(preferences.writing_assistance.cloud, None);
}

#[test]
fn duplicating_a_note_copies_writing_assistance_overrides() {
    let mut note = Note::new(Utc::now());
    note.editor_preferences.writing_assistance.spelling = Some(false);
    note.editor_preferences.writing_assistance.cloud = Some(true);
    let copy = note.duplicate(Utc::now());
    assert_eq!(copy.editor_preferences.writing_assistance,
               note.editor_preferences.writing_assistance);
}
```

- [ ] **Step 2: Run the focused domain tests and confirm the missing-field/type failure**

Run: `cargo test -p noor-domain --test note_model writing_assistance -- --nocapture`

Expected: compilation fails because `WritingAssistanceOverrides` and `writing_assistance` do not exist.

- [ ] **Step 3: Add the backward-compatible domain value**

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritingAssistanceOverrides {
    #[serde(default)] pub spelling: Option<bool>,
    #[serde(default)] pub grammar: Option<bool>,
    #[serde(default)] pub offline_prediction: Option<bool>,
    #[serde(default)] pub cloud: Option<bool>,
}

// In EditorPreferences:
#[serde(default)]
pub writing_assistance: WritingAssistanceOverrides,
```

Initialize it with `WritingAssistanceOverrides::default()` in `EditorPreferences::default()` and re-export the type from `crates/domain/src/lib.rs`.

- [ ] **Step 4: Run domain tests**

Run: `cargo test -p noor-domain --test note_model && cargo test -p noor-domain`

Expected: all domain tests pass, including legacy JSON and duplicate behavior.

- [ ] **Step 5: Commit**

```bash
git add crates/domain/src/note.rs crates/domain/src/lib.rs crates/domain/tests/note_model.rs
git commit -m "feat: add per-note writing assistance overrides"
```

### Task 2: Private Global Preferences and Keyring Secret

**Files:**
- Modify: `Cargo.toml`
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/writing_assistance/mod.rs`
- Create: `apps/noor-notes/src/writing_assistance/preferences.rs`
- Modify: `apps/noor-notes/src/key_store.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Create: `apps/noor-notes/tests/writing_assistance_preferences.rs`
- Modify: `apps/noor-notes/tests/key_store.rs`

**Interfaces:**
- Produces: `WritingAssistancePreferences`, `ResolvedWritingAssistance`, `ProviderConfiguration`, `WritingAssistanceStore`.
- Produces: `WritingAssistancePreferences::resolve(&WritingAssistanceOverrides) -> ResolvedWritingAssistance`.
- Produces: `validate_provider_endpoint(&str) -> Result<reqwest::Url, PreferenceError>`.
- Produces: `SecretKind::WritingAssistanceApiKey` stored under account `provider`.

- [ ] **Step 1: Write failing tests for defaults, override resolution, URL policy, corruption, permissions, and key isolation**

```rust
#[test]
fn local_features_are_on_and_cloud_is_off_by_default() {
    let value = WritingAssistancePreferences::default();
    assert!(value.spelling && value.grammar && value.offline_prediction);
    assert!(!value.cloud_enabled);
    assert_eq!(value.language, "auto");
}

#[test]
fn per_note_values_override_only_the_selected_global_values() {
    let global = WritingAssistancePreferences::default();
    let overrides = WritingAssistanceOverrides {
        grammar: Some(false), cloud: Some(true), ..Default::default()
    };
    let effective = global.resolve(&overrides);
    assert!(effective.spelling);
    assert!(!effective.grammar);
    assert!(effective.offline_prediction);
    assert!(!effective.cloud); // provider has not been validated
}

#[test]
fn endpoint_policy_rejects_remote_http_but_accepts_loopback_http() {
    assert!(validate_provider_endpoint("http://example.com").is_err());
    assert!(validate_provider_endpoint("http://localhost:11434").is_ok());
    assert!(validate_provider_endpoint("http://127.0.0.1:8080/v1").is_ok());
    assert!(validate_provider_endpoint("https://api.example.com").is_ok());
}
```

Also test that malformed JSON returns safe defaults without rewriting the malformed file, successful saves use mode `0600`, changing endpoint/model clears `provider_validated` and `cloud_enabled`, remote validation requires a key, and `WritingAssistanceApiKey` cannot be retrieved as `DatabaseKey`.

- [ ] **Step 2: Run the tests and confirm they fail on missing modules and enum variant**

Run: `cargo test -p noor-notes --test writing_assistance_preferences --test key_store`

Expected: compilation fails on the new types and `WritingAssistanceApiKey`.

- [ ] **Step 3: Add exact dependency pins and preference types**

```toml
# workspace dependencies
url = "2"

# apps/noor-notes dependencies
reqwest.workspace = true
url.workspace = true
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfiguration {
    pub base_url: String,
    pub model: String,
    pub provider_validated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritingAssistancePreferences {
    pub spelling: bool,
    pub grammar: bool,
    pub offline_prediction: bool,
    pub cloud_enabled: bool,
    pub language: String,
    pub provider: ProviderConfiguration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedWritingAssistance {
    pub spelling: bool,
    pub grammar: bool,
    pub offline_prediction: bool,
    pub cloud: bool,
}
```

Use the same temp-file, `sync_all`, rename, directory sync, and Unix `OpenOptionsExt::mode(0o600)` sequence as `appearance/preferences.rs`, with the final path `<config-dir>/noor-notes/writing-assistance.json`. Limit endpoint to 2,048 characters, model/language to 128, trim surrounding whitespace, and compute effective cloud as `requested && provider_validated && endpoint/model policy valid`.

- [ ] **Step 4: Add the keyring variant and verify tests**

Map `SecretKind::WritingAssistanceApiKey` to `writing-assistance-api-key`, keep it isolated by the existing `(application, kind, account)` attributes, then run:

Run: `cargo test -p noor-notes --test writing_assistance_preferences --test key_store`

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock apps/noor-notes/Cargo.toml apps/noor-notes/src/writing_assistance apps/noor-notes/src/key_store.rs apps/noor-notes/src/lib.rs apps/noor-notes/tests/writing_assistance_preferences.rs apps/noor-notes/tests/key_store.rs
git commit -m "feat: persist private writing assistance settings"
```

### Task 3: Common GtkSourceView Editor and Checkable Regions

**Files:**
- Modify: `apps/noor-notes/src/editor/source_adapter.rs`
- Modify: `apps/noor-notes/src/editor/rich_adapter.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Create: `apps/noor-notes/src/writing_assistance/scope.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Modify: `apps/noor-notes/tests/source_editor.rs`
- Modify: `apps/noor-notes/tests/rich_editor.rs`
- Create: `apps/noor-notes/tests/writing_assistance_scope.rs`

**Interfaces:**
- Produces: `SourceEditorAdapter::new_rich(text, theme) -> SourceEditorAdapter` with syntax and line-number presentation disabled.
- Produces: `CheckRegion { start: usize, end: usize }` and `checkable_regions(buffer, mode) -> Vec<CheckRegion>`.
- Preserves: `RichEditorAdapter` and `RichBuffer` operate through GTK base-class upcasts without changing snapshot data.

- [ ] **Step 1: Write failing editor-substrate and Unicode region tests**

```rust
#[test]
fn rich_editor_uses_a_source_buffer_without_syntax_highlighting() {
    gtk::init().unwrap();
    let editor = SourceEditorAdapter::new_rich("hello", EffectiveTheme::Light);
    assert!(!editor.buffer().is_highlight_syntax());
    assert!(!editor.view().shows_line_numbers());
    let gtk_buffer: gtk::TextBuffer = editor.buffer().clone().upcast();
    assert_eq!(gtk_buffer.text(&gtk_buffer.start_iter(), &gtk_buffer.end_iter(), true), "hello");
}

#[test]
fn character_regions_do_not_split_bengali_text() {
    let text = "আমি লিখি";
    let regions = plain_text_regions(text);
    assert_eq!(regions, vec![CheckRegion { start: 0, end: text.chars().count() }]);
}
```

Add GTK tests that Markdown excludes fenced/inline code and `no-spell-check` context, while Code includes only `comment` and `string` contexts and never returns a region crossing excluded syntax.

- [ ] **Step 2: Run the focused tests and confirm the new constructor/scope failures**

Run: `xvfb-run -a cargo test -p noor-notes --test source_editor --test rich_editor --test writing_assistance_scope`

Expected: compilation fails on `new_rich`, `CheckRegion`, and region functions.

- [ ] **Step 3: Standardize construction and implement region coalescing**

```rust
pub fn new_rich(text: &str, theme: EffectiveTheme) -> Self {
    let value = Self::new_with_theme(text, None, theme);
    value.buffer.set_highlight_syntax(false);
    value.view.set_show_line_numbers(false);
    value.view.set_highlight_current_line(false);
    value.view.set_auto_indent(false);
    value
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRegion { pub start: usize, pub end: usize }
```

In `note_window.rs`, always keep concrete `sourceview5::Buffer`/`View` values, upcast clones only for existing formatting/autosave APIs, and use `new_rich` for Rich mode. Build regions by walking character offsets and coalescing adjacent included characters; call `buffer.iter_has_context_class(iter, "comment")`, `"string"`, `"no-spell-check"`, and `"path"` according to mode.

- [ ] **Step 4: Verify editor behavior and rich snapshot compatibility**

Run: `xvfb-run -a cargo test -p noor-notes --test source_editor --test rich_editor --test rich_formatting_persistence --test autosave --test writing_assistance_scope`

Expected: all tests pass and rich snapshots remain byte-for-byte semantically equivalent.

- [ ] **Step 5: Commit**

```bash
git add apps/noor-notes/src/editor apps/noor-notes/src/note_window.rs apps/noor-notes/src/writing_assistance apps/noor-notes/tests/source_editor.rs apps/noor-notes/tests/rich_editor.rs apps/noor-notes/tests/writing_assistance_scope.rs
git commit -m "refactor: use GtkSourceView across editor modes"
```

### Task 4: Offline Spelling Adapter

**Files:**
- Modify: `Cargo.toml`
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/writing_assistance/spelling.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Create: `apps/noor-notes/tests/writing_assistance_spelling.rs`

**Interfaces:**
- Produces: `SpellService::attach(buffer, view, language, enabled) -> SpellSession`.
- Produces: `SpellSession::set_enabled(bool)`, `set_language(&str)`, `is_available()`, and `installed_languages()`.
- Owns: `libspelling::TextBufferAdapter` for the lifetime of the editor.

- [ ] **Step 1: Install the native development prerequisite used by the Rust binding**

Run: `sudo apt-get install -y libspelling-1-dev enchant-2 hunspell-en-us`

Expected: `pkg-config --modversion libspelling-1` prints a 0.4-compatible version and `enchant-2 -l` includes `en_US`.

- [ ] **Step 2: Write a failing Xvfb integration test**

```rust
#[test]
fn spelling_attaches_actions_and_can_be_toggled() {
    gtk::init().unwrap();
    libspelling::init();
    let editor = SourceEditorAdapter::new_rich("mispelled", EffectiveTheme::Light);
    let session = SpellService::attach(editor.buffer(), editor.view(), "auto", true);
    assert!(session.is_enabled());
    assert!(editor.view().action_group("spelling").is_some());
    session.set_enabled(false);
    assert!(!session.is_enabled());
}
```

Also assert that an unknown dictionary reports unavailable without panicking, installed language codes/names are unique and sorted, the extra menu exposes a visible “Spelling” category label, Add to Dictionary remains available when the provider supports it, Markdown skips `no-spell-check` regions, and Code spelling marks can occur only in `comment`/`string` contexts.

- [ ] **Step 3: Run the test and confirm the dependency/module failure**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_spelling`

Expected: compilation fails because `libspelling` and `SpellService` are absent.

- [ ] **Step 4: Add libspelling 0.4.1 and implement the GNOME-supported attachment flow**

```toml
# workspace
libspelling = "=0.4.1"

# app
libspelling.workspace = true
```

```rust
let checker = libspelling::Checker::default();
let adapter = libspelling::TextBufferAdapter::new(buffer, &checker);
view.set_extra_menu(Some(&adapter.menu_model()));
view.insert_action_group("spelling", Some(&adapter));
adapter.set_enabled(enabled);
```

For `auto`, select the current locale only when `Provider::default().supports_language(code)` succeeds; otherwise leave the checker unavailable. Expose provider languages for the settings dropdown. Wrap the adapter menu model in a `gio::Menu` section labelled “Spelling” so correction meaning is not conveyed by underline colour alone. Keep libspelling’s restrained red spelling underline and built-in replacement/add-to-dictionary actions.

- [ ] **Step 5: Run spelling and editor tests**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_spelling --test source_editor --test rich_editor`

Expected: all tests pass with `hunspell-en-us`; missing-language test returns unavailable.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock apps/noor-notes/Cargo.toml apps/noor-notes/src/writing_assistance apps/noor-notes/tests/writing_assistance_spelling.rs
git commit -m "feat: add offline spelling assistance"
```

### Task 5: Offline English Grammar Engine

**Files:**
- Modify: `Cargo.toml`
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/writing_assistance/issue.rs`
- Create: `apps/noor-notes/src/writing_assistance/grammar.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Create: `apps/noor-notes/tests/writing_assistance_grammar.rs`

**Interfaces:**
- Produces: `AssistanceIssue { range: Range<usize>, category, message, replacements, source }` using character offsets.
- Produces: `GrammarService::check(text, language, regions) -> Vec<AssistanceIssue>`.
- Produces: `replacement_for(Suggestion, original_span) -> String`, handling replace, insert-after, and remove.

- [ ] **Step 1: Write failing pure grammar tests**

```rust
#[test]
fn english_grammar_returns_character_indexed_replacement() {
    let issues = GrammarService::default().check(
        "This is an test.", "en-US", &[CheckRegion { start: 0, end: 16 }]
    );
    assert!(issues.iter().any(|issue|
        issue.message.contains("article") && issue.replacements.iter().any(|r| r == "a")
    ));
}

#[test]
fn unsupported_language_and_excluded_ranges_return_no_findings() {
    let service = GrammarService::default();
    assert!(service.check("This is an test.", "bn", &[CheckRegion { start: 0, end: 16 }]).is_empty());
    assert!(service.check("This is an test.", "en", &[]).is_empty());
}
```

Add a Bengali-prefix test proving a Harper span is shifted by Unicode character count rather than UTF-8 byte count, and reject any lint that leaves its submitted `CheckRegion`.

- [ ] **Step 2: Run the test and confirm the engine/type failure**

Run: `cargo test -p noor-notes --test writing_assistance_grammar`

Expected: compilation fails on `GrammarService` and `AssistanceIssue`.

- [ ] **Step 3: Add Harper and map its lints without enabling its duplicate spellchecker**

```toml
# workspace
harper-core = { version = "=2.5.0", default-features = false, features = ["concurrent"] }

# app
harper-core.workspace = true
```

Build each region with `PlainEnglish`, `Document::new_curated`, `FstDictionary::curated`, and `LintGroup::new_curated(..., Dialect::American)`. Filter out `lint.lint_kind == LintKind::Spelling` so libspelling remains the sole spelling provider. Convert `ReplaceWith`, `InsertAfter`, and `Remove` into complete replacement strings for the issue range, cap each issue to five replacements of 256 characters, and label Harper categories with `LintKind::to_string()`.

- [ ] **Step 4: Run grammar tests**

Run: `cargo test -p noor-notes --test writing_assistance_grammar`

Expected: all grammar, Unicode, region, and unsupported-language tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock apps/noor-notes/Cargo.toml apps/noor-notes/src/writing_assistance apps/noor-notes/tests/writing_assistance_grammar.rs
git commit -m "feat: add private offline grammar checking"
```

### Task 6: Encrypted Bounded Prediction Model

**Files:**
- Modify: `Cargo.toml`
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/writing_assistance/prediction.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Create: `apps/noor-notes/tests/writing_assistance_prediction.rs`
- Create: `crates/storage/migrations/0004_writing_assistance.sql`
- Create: `crates/storage/src/writing_assistance.rs`
- Modify: `crates/storage/Cargo.toml`
- Modify: `crates/storage/src/repository.rs`
- Modify: `crates/storage/src/lib.rs`
- Create: `crates/storage/tests/writing_assistance.rs`

**Interfaces:**
- Produces: `PredictionModel::train(&[String])`, `suggest(context, partial, limit) -> Vec<String>`, `prune(50_000)`.
- Produces: `PredictionModelRecord { schema_version: u32, corpus_watermark: String, model_json: String, updated_at: DateTime<Utc> }`.
- Produces: `SqliteNoteRepository::{prediction_corpus,load_prediction_model,replace_prediction_model}`.

- [ ] **Step 1: Write failing prediction and storage tests**

```rust
#[test]
fn ranks_trigrams_before_bigrams_and_filters_partial_unicode_tokens() {
    let mut model = PredictionModel::default();
    model.train(&["আমি আজ লিখি। আমি আজ পড়ি। clear support works. clear support helps.".into()]);
    assert_eq!(model.suggest("clear support", "h", 3)[0], "helps");
    assert_eq!(model.suggest("আমি আজ", "প", 3)[0], "পড়ি");
}

#[test]
fn pruning_is_deterministic_and_never_exceeds_the_bound() {
    let mut model = model_with_distinct_entries(50_100);
    model.prune(50_000);
    assert_eq!(model.entry_count(), 50_000);
}
```

Storage tests must open an encrypted temp repository, prove Active/Archived bodies are returned, prove Trash/title/tags are excluded, round-trip the model record, recover from malformed model JSON by returning `None`, and prove trash/restore/permanent delete changes the corpus watermark.

- [ ] **Step 2: Run tests and confirm missing model/table failures**

Run: `cargo test -p noor-notes --test writing_assistance_prediction && cargo test -p noor-storage --test writing_assistance`

Expected: compilation fails on prediction and repository interfaces.

- [ ] **Step 3: Implement Unicode n-grams and deterministic pruning**

```toml
# workspace and app
unicode-segmentation = "1.12"
```

Tokenize with `UnicodeSegmentation::unicode_words`, lowercase with Unicode `to_lowercase`, retain original candidate spelling, count bigrams and trigrams, rank by trigram frequency then bigram frequency then case-folded lexical order, deduplicate case-insensitively, and return at most `limit.min(3)`. Prune lowest frequency first with lexical tie-breaking until `entry_count <= 50_000`. Add `sha2.workspace = true` to `crates/storage/Cargo.toml` for the corpus watermark.

- [ ] **Step 4: Add encrypted derived-model persistence and corpus watermark**

```sql
CREATE TABLE IF NOT EXISTS writing_prediction_model (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL,
    corpus_watermark TEXT NOT NULL,
    model_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

Run migration 0004 unconditionally after the existing conditional migrations. Build the watermark as SHA-256 over sorted tuples of note UUID, revision, and state for Active/Archived notes; return body strings only to the trainer. Replace row `id = 1` in one transaction. Treat schema mismatch or malformed JSON as absent derived data, without modifying notes.

- [ ] **Step 5: Run prediction/storage tests**

Run: `cargo test -p noor-notes --test writing_assistance_prediction && cargo test -p noor-storage --test writing_assistance --test encrypted_repository --test lifecycle`

Expected: all tests pass and the encrypted repository never exposes trashed content to the prediction corpus.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock apps/noor-notes/Cargo.toml apps/noor-notes/src/writing_assistance apps/noor-notes/tests/writing_assistance_prediction.rs crates/storage/Cargo.toml crates/storage/migrations/0004_writing_assistance.sql crates/storage/src crates/storage/tests/writing_assistance.rs
git commit -m "feat: learn encrypted local writing predictions"
```

### Task 7: Privacy-Scoped OpenAI-Compatible Client

**Files:**
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/writing_assistance/cloud.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Create: `apps/noor-notes/tests/writing_assistance_cloud.rs`

**Interfaces:**
- Produces: `CloudAssistanceClient::test_connection`, `check_grammar`, and `predict`.
- Produces: `paragraph_scope(text, cursor, 2_000)` and `sentence_scope(text, cursor, 800)` with base character offsets.
- Consumes: validated `ProviderConfiguration` and zeroized API key bytes; returns character-indexed `AssistanceIssue`/up to three suggestion strings.

- [ ] **Step 1: Write failing wiremock tests for the complete provider contract**

```rust
#[tokio::test]
async fn grammar_sends_only_the_capped_current_paragraph() {
    let server = MockServer::start().await;
    let client = test_client(&server, "secret");
    let note = format!("title-like first paragraph\n\n{}", "x".repeat(2_300));
    let issues = client.check_grammar(&note, note.chars().count(), Some("en")).await.unwrap();
    assert!(issues.is_empty());
    let request = server.received_requests().await.unwrap().pop().unwrap();
    let body = String::from_utf8(request.body).unwrap();
    assert!(!body.contains("title-like first paragraph"));
    assert!(extract_user_text(&body).chars().count() <= 2_000);
}
```

Add tests for: `/v1/chat/completions` path normalization; HTTPS/loopback policy; Authorization header without body key leakage; 800-character prediction scope; no title/tags/other-note fields; parsing `choices[0].message.content`; out-of-range offsets; more than three/duplicate/control-character suggestions; replacement bounds; 10-second timeout; 429/5xx/malformed JSON errors; and connection-test success/failure.

- [ ] **Step 2: Run cloud tests and confirm the client failure**

Run: `cargo test -p noor-notes --test writing_assistance_cloud`

Expected: compilation fails because `CloudAssistanceClient` is absent.

- [ ] **Step 3: Add the test dependency and exact request/response DTOs**

```toml
[dev-dependencies]
wiremock.workspace = true
```

```rust
#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    temperature: f32,
    messages: [ChatMessage<'a>; 2],
}

#[derive(Deserialize)]
struct GrammarPayload { issues: Vec<CloudIssue> }

#[derive(Deserialize)]
struct PredictionPayload { suggestions: Vec<String> }
```

Append `/v1/chat/completions` only when absent, use reqwest rustls with a 10-second timeout, set bearer auth only when a key is present, use deterministic temperature `0.0`, and parse content as JSON after the outer response succeeds. Convert snippet-relative character offsets to document offsets only after all bounds and region checks pass. Never include bodies in error variants or tracing.

- [ ] **Step 4: Run cloud tests**

Run: `cargo test -p noor-notes --test writing_assistance_cloud`

Expected: all request scoping, validation, failure, and response parsing tests pass.

- [ ] **Step 5: Commit**

```bash
git add apps/noor-notes/Cargo.toml apps/noor-notes/src/writing_assistance apps/noor-notes/tests/writing_assistance_cloud.rs
git commit -m "feat: add privacy-scoped optional AI assistance"
```

### Task 8: Controller, Grammar Tags, Corrections, and Stale-Result Safety

**Files:**
- Create: `apps/noor-notes/src/writing_assistance/controller.rs`
- Create: `apps/noor-notes/src/writing_assistance/issue_popover.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/ui/editor_status_bar.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Create: `apps/noor-notes/tests/writing_assistance_controller.rs`

**Interfaces:**
- Produces: `WritingAssistanceController::new`, `set_preferences`, `set_suppressed`, `notify_content_changed`, and `shutdown`.
- Produces: `AssistanceStatus::{Idle,Checking,Offline,Cloud,Unavailable}` and status-bar text/accessible label updates.
- Applies: grammar replacements inside one `begin_user_action`/`end_user_action` pair.

- [ ] **Step 1: Write failing paused-time and Xvfb tests**

```rust
#[tokio::test(start_paused = true)]
async fn stale_generation_results_are_discarded() {
    let harness = ControllerHarness::with_delayed_grammar();
    harness.edit("This is an test.");
    tokio::time::advance(Duration::from_millis(450)).await;
    harness.edit("This is a test.");
    harness.complete_first_request();
    assert!(harness.visible_issues().is_empty());
}
```

Add tests for exact 450 ms grammar debounce, one grammar task at a time, code/Markdown regions passed to the engine, blue grammar underline tags, category/message/replacement/Ignore once popover labels, one-step undo after replacement, transient tags absent from rich snapshots/autosave/export, status transitions, and full suppression in View Only/Trash.

- [ ] **Step 2: Run the controller tests and confirm missing controller/UI failures**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_controller`

Expected: compilation fails on controller, status, and popover types.

- [ ] **Step 3: Implement generation-safe debounce and transient issue rendering**

```rust
fn notify_content_changed(&self) {
    let generation = self.generation.get().wrapping_add(1);
    self.generation.set(generation);
    self.clear_issues_and_prediction();
    self.abort_pending_grammar();
    self.schedule_grammar(Duration::from_millis(450), generation);
}

fn accept_result(&self, generation: u64, issues: Vec<AssistanceIssue>) {
    if generation != self.generation.get() || self.suppressed.get() { return; }
    self.render_issues(issues);
}
```

Use named transient `gtk::TextTag`s with single blue underline styling and accessible issue descriptions. On click/right-click, resolve the issue at the character offset and show `IssuePopover`; replacement deletes/inserts within one user action, then restores editor focus. Keep ignored issue IDs only for the current generation. Run Harper through `tokio::task::spawn_blocking` and cloud work on the existing Tokio runtime; send accepted immutable results back to GTK’s main context. Keep one abortable request per cloud kind and apply exponential retry delays of 1, 2, 4, 8, then 30 seconds after rate-limit or transient 5xx responses; any edit invalidates the request immediately.

- [ ] **Step 4: Wire the controller to edits, mode/state, and status bar**

Create the controller after the concrete source buffer/view in `NoteWindow::new`. Notify it from the existing changed handler without altering autosave timing. Suppress it when `editor_preferences.view_only` or `NoteState::Trashed`; shut it down when the window closes. Add a third compact status label with text plus accessible label, not color alone.

- [ ] **Step 5: Run controller and persistence tests**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_controller --test rich_formatting_persistence --test autosave --test export --test view_only_mode`

Expected: all tests pass; stale results and transient GTK tags never affect persisted/exported text.

- [ ] **Step 6: Commit**

```bash
git add apps/noor-notes/src/writing_assistance apps/noor-notes/src/note_window.rs apps/noor-notes/src/ui/editor_status_bar.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/writing_assistance_controller.rs
git commit -m "feat: surface grammar issues safely in the editor"
```

### Task 9: Inline Ghost Prediction and Keyboard Alternatives

**Files:**
- Create: `apps/noor-notes/src/writing_assistance/prediction_overlay.rs`
- Modify: `apps/noor-notes/src/writing_assistance/controller.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/ui/editor_canvas.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Create: `apps/noor-notes/tests/writing_assistance_prediction_ui.rs`

**Interfaces:**
- Produces: `PredictionOverlay::new(canvas, view)`, `show`, `dismiss`, `show_alternatives`, `accept_selected`.
- Controller behavior: local suggestion after 250 ms, optional cloud merge, generation guard, at most three deduplicated choices.
- Keyboard contract: Tab accepts only a visible suggestion; Escape dismisses it; Alt+Down opens alternatives; arrows select; Enter accepts.

- [ ] **Step 1: Write failing Xvfb keyboard and non-mutation tests**

```rust
#[test]
fn ghost_text_never_mutates_the_buffer_until_tab_acceptance() {
    gtk::init().unwrap();
    let mut harness = PredictionUiHarness::new("clear support ", &["helps", "works"]);
    harness.show_prediction();
    assert_eq!(harness.buffer_text(), "clear support ");
    assert_eq!(harness.undo_depth(), 0);
    harness.press(gdk::Key::Tab, gdk::ModifierType::empty());
    assert_eq!(harness.buffer_text(), "clear support helps");
    assert_eq!(harness.undo_depth(), 1);
}
```

Add tests for normal Tab behavior without a suggestion, Escape dismissal, Alt+Down plus arrow/Enter navigation, a three-item maximum, cursor/selection/edit/focus-loss dismissal, 250 ms debounce, code-context filtering, reduced-motion CSS class, focus restoration, and accessible announcement changing once per new suggestion.

- [ ] **Step 2: Run the prediction UI test and confirm missing overlay behavior**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_prediction_ui`

Expected: compilation fails on `PredictionOverlay` and harness-facing controller methods.

- [ ] **Step 3: Build a non-buffer GTK overlay and alternatives popover**

Wrap the existing editor canvas in `gtk::Overlay`, add a non-targetable `gtk::Fixed` ghost layer and subdued label, and position it from `TextView::iter_location` plus widget coordinate translation. Put alternatives in a `gtk::Popover` anchored below the cursor. The label and popover hold strings only; never insert a text anchor/tag for ghost content.

```rust
fn accept(&self, suggestion: &str) {
    self.buffer.begin_user_action();
    self.buffer.insert_at_cursor(suggestion);
    self.buffer.end_user_action();
    self.dismiss_prediction();
    self.view.grab_focus();
}
```

Install one capture-phase `gtk::EventControllerKey`; return `Propagation::Stop` only when the prediction layer consumed the key, otherwise return `Proceed` so existing shortcuts and Tab behavior remain intact.

- [ ] **Step 4: Merge local/cloud candidates and enforce dismissal rules**

After 250 ms, take the current allowed region, derive preceding two tokens and a partial token, query the encrypted local model, then optionally request cloud candidates with the 800-character scope. Keep local candidates when cloud fails. Deduplicate case-insensitively, reject controls/blank/overlong strings, limit to three, and accept only when generation/cursor/selection still match.

- [ ] **Step 5: Run prediction UI and persistence tests**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_prediction_ui --test autosave --test editor_history --test rich_formatting_persistence --test search --test export`

Expected: all tests pass; ghost content is absent from every persistence and history surface.

- [ ] **Step 6: Commit**

```bash
git add apps/noor-notes/src/writing_assistance apps/noor-notes/src/note_window.rs apps/noor-notes/src/ui/editor_canvas.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/writing_assistance_prediction_ui.rs
git commit -m "feat: add inline predictive writing suggestions"
```

### Task 10: Global and Per-Note Controls

**Files:**
- Create: `apps/noor-notes/src/ui/writing_assistance_settings.rs`
- Create: `apps/noor-notes/src/ui/note_writing_assistance.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Create: `apps/noor-notes/tests/writing_assistance_settings_ui.rs`

**Interfaces:**
- Produces: `WritingAssistanceSettings::new(app, store, keys)` and cached `app.writing-assistance-settings` action.
- Produces: `NoteWritingAssistancePopover::new(global, overrides, mode)` and a changed callback returning `WritingAssistanceOverrides`.
- Consumes: `SpellSession::installed_languages`, `WritingAssistanceStore`, key store, controller settings updates.

- [ ] **Step 1: Write failing GTK UI tests**

```rust
#[test]
fn settings_show_safe_defaults_and_cloud_consent_gate() {
    gtk::init().unwrap();
    let ui = SettingsHarness::default();
    assert!(ui.switch("Spelling").is_active());
    assert!(ui.switch("Grammar").is_active());
    assert!(ui.switch("Offline predictions").is_active());
    assert!(!ui.switch("Online AI assistance").is_active());
    assert!(!ui.switch("Online AI assistance").is_sensitive());
    assert!(ui.text().contains("current paragraph"));
    assert!(ui.text().contains("nearby sentence"));
}
```

Add tests that the main menu action opens one reused preferences window; installed dictionaries populate Automatic + unique languages; endpoint/model/key edits revoke validation; Test Connection controls cloud sensitivity; API key is never present in the JSON file; note override switch exposes four controls; save updates encrypted note metadata/controller immediately; Code shows “Checks comments and strings only”; View Only/Trash hide the note control; all controls have accessible names and keyboard focus.

- [ ] **Step 2: Run the settings UI test and confirm missing windows/actions**

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_settings_ui`

Expected: compilation fails on the settings and note-popover types.

- [ ] **Step 3: Build the global preferences window and validation flow**

Follow `AppearanceSettings`/`adw::PreferencesWindow` conventions. Use four switch rows, language dropdown, endpoint/model/key entry rows, Test Connection button, validation status row, and explicit privacy copy. Configure long endpoint, model, validation, and privacy labels to wrap rather than widen the window. On key save/delete use `SecretKind::WritingAssistanceApiKey`; immediately zeroize temporary key bytes. Disable cloud before every validation attempt and enable its switch only after endpoint/model plus connection test pass.

- [ ] **Step 4: Build note overrides and connect persistence**

Add a Writing Assistance menu button inside the existing More popover. “Override global settings for this note” controls four switches; otherwise store all four as `None`. Save the updated `EditorPreferences` through the existing note/autosave path and call `controller.set_preferences(global.resolve(&overrides))` immediately. Do not show the button while assistance is suppressed.

- [ ] **Step 5: Register the app action and run UI/accessibility tests**

Add `Writing Assistance…` beside Appearance Settings in the main menu and cache its window with the same `Rc<RefCell<Option<_>>>` pattern. Run:

Run: `xvfb-run -a cargo test -p noor-notes --test writing_assistance_settings_ui --test accessibility --test toolbar_actions --test view_only_mode`

Expected: all tests pass and every switch/button/dropdown is keyboard reachable and named.

- [ ] **Step 6: Commit**

```bash
git add apps/noor-notes/src/ui apps/noor-notes/src/managed_app.rs apps/noor-notes/src/note_window.rs apps/noor-notes/tests/writing_assistance_settings_ui.rs
git commit -m "feat: add writing assistance controls"
```

### Task 11: Shared Runtime, Model Rebuilds, and Lifecycle Invalidations

**Files:**
- Create: `apps/noor-notes/src/writing_assistance/runtime.rs`
- Modify: `apps/noor-notes/src/writing_assistance/mod.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/services/trash_command.rs`
- Modify: `apps/noor-notes/src/autosave.rs`
- Create: `apps/noor-notes/tests/writing_assistance_runtime.rs`

**Interfaces:**
- Produces: cloneable `WritingAssistanceRuntime` containing preferences, key access, grammar service, shared prediction model, optional cloud client, repository, and rebuild scheduler.
- Produces: `schedule_model_rebuild(Duration)`, `rebuild_if_stale()`, and `resolved_for(&Note)`.
- Consumes: save/archive/trash/restore/permanent-delete notifications; model replacement occurs only after a successful complete rebuild.

- [ ] **Step 1: Write failing paused-time runtime tests**

```rust
#[tokio::test(start_paused = true)]
async fn rebuild_is_debounced_and_excludes_newly_trashed_notes() {
    let harness = RuntimeHarness::with_active_note("private phrase").await;
    harness.schedule_rebuild();
    harness.trash_note().await;
    tokio::time::advance(Duration::from_secs(4)).await;
    assert_eq!(harness.rebuild_count(), 0);
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(harness.rebuild_count(), 1);
    assert!(!harness.suggest("private", "").contains(&"phrase".into()));
}
```

Add tests for startup rebuild only when schema/watermark is stale, one rebuild after clustered autosaves, archive inclusion, restore inclusion, permanent-delete removal, atomic retention of the previous model when rebuild fails, corrupt-model recovery, cloud absent when keyring/config is invalid, and local services remaining available.

- [ ] **Step 2: Run runtime tests and confirm missing scheduler/runtime**

Run: `cargo test -p noor-notes --test writing_assistance_runtime`

Expected: compilation fails on `WritingAssistanceRuntime`.

- [ ] **Step 3: Implement the shared runtime and five-second rebuild scheduler**

Use an `Arc<RwLock<PredictionModel>>` for reads, train a new model off-thread from `repository.prediction_corpus()`, persist it, then swap the Arc contents only after persistence succeeds. Store/abort one pending rebuild handle so later events reset the five-second timer. On startup compare schema version and exact corpus watermark before deciding whether to rebuild.

- [ ] **Step 4: Wire all lifecycle events and note windows**

Construct one runtime after repository/keyring initialization in `managed_app::run`, start `rebuild_if_stale`, and pass clones to every `NoteWindow` and `MainWindow` call site. Schedule rebuilds after successful autosave and successful archive/trash/restore/permanent delete. Keep scheduling failure non-fatal and body-free in diagnostics.

- [ ] **Step 5: Run runtime, lifecycle, autosave, and controller tests**

Run: `cargo test -p noor-notes --test writing_assistance_runtime --test autosave --test trash_actions --test library_archive_action && cargo test -p noor-storage --test lifecycle --test writing_assistance`

Expected: all tests pass and lifecycle changes remove/add prediction influence after the debounce.

- [ ] **Step 6: Commit**

```bash
git add apps/noor-notes/src/writing_assistance apps/noor-notes/src/managed_app.rs apps/noor-notes/src/note_window.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/src/services/trash_command.rs apps/noor-notes/src/autosave.rs apps/noor-notes/tests/writing_assistance_runtime.rs
git commit -m "feat: coordinate writing assistance services"
```

### Task 12: Packaging, CI, Documentation, and Full Verification

**Files:**
- Modify: `scripts/install-ubuntu.sh`
- Modify: `tests/install_ubuntu.sh`
- Modify: `.github/workflows/security.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `snap/snapcraft.yaml`
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Regenerate: `packaging/flatpak/cargo-sources.json`
- Modify: `tests/snap_manifest.sh`
- Modify: `tests/flatpak_manifest.sh`
- Modify: `README.md`

**Interfaces:**
- Packages: native `libspelling-1`, Enchant, and `hunspell-en-us` in Ubuntu/Snap/Flatpak environments.
- Documents: local defaults, per-note overrides, installed dictionaries, offline English grammar, learned local predictions, optional provider privacy limits, keyring storage, and shortcuts.

- [ ] **Step 1: Write failing installer and manifest assertions**

```bash
require_token "$installer" "libspelling-1-dev"
require_token "$installer" "enchant-2"
require_token "$installer" "hunspell-en-us"
```

Add equivalent Snap stage/build package assertions. For Flatpak, assert GNOME runtime 50, the bundled `hunspell-en-us` dictionary module, and unchanged network permissions. Extend workflow tests so CI images install `libspelling-1-dev`, `libenchant-2-2`, and `hunspell-en-us` before compiling.

- [ ] **Step 2: Run packaging tests and confirm missing dependency failures**

Run: `bash tests/install_ubuntu.sh && bash tests/snap_manifest.sh && bash tests/flatpak_manifest.sh && bash tests/release_workflow.sh`

Expected: tests fail because spelling packages/manifests are not yet declared and Cargo sources are stale.

- [ ] **Step 3: Update native, Snap, Flatpak, and CI dependencies**

Add `libspelling-1-dev enchant-2 hunspell-en-us` to Ubuntu build/install prerequisites; add `libspelling-1-dev` to Snap build packages and `libspelling-1-2 libenchant-2-2 hunspell-en-us` to stage packages. GNOME Platform/SDK 50 already supplies libspelling and Enchant. Add this deterministic Flatpak module before the application module so US English works even when the locale extension does not contain it; do not add a new network permission:

```yaml
  - name: hunspell-en-us
    buildsystem: simple
    sources:
      - type: archive
        url: https://github.com/LibreOffice/dictionaries/archive/c011d96c90cc9c6c8b6d95ec6d83d11daacce994.tar.gz
        sha256: cdbc8d6d79425b2749f7eee077c107ef96fb23c44e4f0ca450897f0a4402581a
    build-commands:
      - install -Dm644 en/en_US.aff /app/share/hunspell/en_US.aff
      - install -Dm644 en/en_US.dic /app/share/hunspell/en_US.dic
```

- [ ] **Step 4: Regenerate Cargo sources deterministically**

Run the official Flatpak builder tool against the committed `Cargo.lock`:

```bash
flatpak_tools_dir="$(mktemp -d)"
git clone --depth 1 https://github.com/flatpak/flatpak-builder-tools.git "$flatpak_tools_dir"
python3 "$flatpak_tools_dir/cargo/flatpak-cargo-generator.py" Cargo.lock -o packaging/flatpak/cargo-sources.json
```

Then run `bash tests/flatpak_manifest.sh`; expected result is PASS with exactly the registry packages/checksums in the lockfile.

- [ ] **Step 5: Update the user documentation**

Document that local checks are default-on, Harper grammar is English offline, spelling uses installed system dictionaries, predictions learn only from non-trashed encrypted note bodies, cloud is opt-in, payload caps are 2,000/800 Unicode characters, the key is in GNOME Keyring, Code checks comments/strings only, and shortcuts are Tab/Escape/Alt+Down/arrows/Enter.

- [ ] **Step 6: Run formatter, lint, all tests, packaging checks, and release build**

Run:

```bash
cargo fmt --all -- --check
cargo +1.85.0 check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
xvfb-run -a cargo test -p noor-notes
bash tests/install_ubuntu.sh
bash tests/snap_manifest.sh
bash tests/flatpak_manifest.sh
bash tests/release_workflow.sh
cargo build --workspace --release
git diff --check
git status --short
```

Expected: every command exits 0; `git status --short` lists only the intended implementation/docs files plus the two pre-existing untracked Snap artifacts, which must remain untouched.

- [ ] **Step 7: Perform manual visual/privacy review**

Launch under both light and dark themes and verify Rich/Markdown/Plain/Code, installed language selection, offline startup, missing dictionary, global/per-note switches, View Only/Trash suppression, popover placement near viewport edges, keyboard-only correction/prediction, focus return, reduced motion, provider consent/revocation, and no console body/key output. Inspect a local provider capture to confirm only current paragraph/sentence scopes are transmitted.

- [ ] **Step 8: Commit**

```bash
git add scripts/install-ubuntu.sh tests/install_ubuntu.sh .github/workflows/security.yml .github/workflows/release.yml snap/snapcraft.yaml packaging/flatpak/io.github.saamaamr.NoorNotes.yml packaging/flatpak/cargo-sources.json tests/snap_manifest.sh tests/flatpak_manifest.sh README.md
git commit -m "build: package and document writing assistance"
```

Do not add, delete, commit, or modify `noor-notes_0.1.0_amd64.snap` or `noor-notes_0.1.1_amd64.snap`.
