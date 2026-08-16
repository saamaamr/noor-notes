# Writing Assistance Design

**Date:** 2026-08-16
**Status:** Approved

## Objective

Add privacy-first writing assistance to Noor Notes: spelling, grammar, and predictive suggestions are enabled locally by default; an optional configurable cloud provider can improve multilingual grammar and prediction after explicit user consent. Users can disable each capability globally or override the global choices for an individual note.

## Product Principles

- Editing must remain fully functional offline and when every assistance engine is unavailable or disabled.
- No correction is applied automatically. The user explicitly accepts every replacement or prediction.
- Local spelling, grammar, and prediction are enabled by default. Cloud assistance is disabled by default.
- Cloud assistance never receives a complete note, title, tags, another note, encryption material, or account data.
- Missing dictionaries and unsupported grammar languages degrade quietly instead of interrupting editing.
- Assistance is available in Rich Text, Markdown, Plain Text, and Code. Code mode checks only comments and strings.
- Visual styling is accompanied by text labels and accessible descriptions; colour is never the only indicator.

## Architecture

### Common editor substrate

All note modes use `sourceview5::Buffer` and `sourceview5::View`. Rich Text continues to use the existing `RichBuffer` formatting and persistence APIs through the GTK base types because `GtkSourceBuffer` and `GtkSourceView` extend `GtkTextBuffer` and `GtkTextView`. Rich mode has syntax highlighting disabled. Markdown, Plain Text, and Code retain their current language and palette behavior.

This common substrate allows libspelling, GtkSourceView context classes, completion proposals, and one assistance controller to work across every mode. The migration must not alter saved plain text or rich-document snapshots.

### Writing assistance controller

Each editable note window owns one `WritingAssistanceController`. It receives:

- the source buffer and view;
- the resolved global and per-note settings;
- the editor mode and source language;
- the local spelling, grammar, and prediction services;
- an optional cloud client;
- a monotonically increasing document generation number.

The controller owns transient issue tags, the inline prediction overlay, the alternatives model, debounce timers, and in-flight task cancellation. Every content edit increments the generation. Results whose generation no longer matches are discarded without touching the buffer.

Spelling updates through libspelling. Grammar checks begin after 450 milliseconds without a content edit. Prediction begins after 250 milliseconds without a content edit. CPU-heavy local checks use a blocking worker; network work uses the existing async runtime. Neither path blocks GTK's main thread.

### Independent services

`SpellService` wraps libspelling and resolves the automatic language from the current locale and installed Enchant/Hunspell dictionaries. The language selector also exposes installed dictionaries explicitly. If no matching dictionary exists, spelling reports Unavailable and performs no check.

`GrammarService` wraps Harper for private offline English grammar and style checks. Non-English text receives no local grammar findings unless a later offline engine supports that language. When cloud assistance is enabled, the cloud service may supply grammar results for other languages.

`PredictionService` provides local predictions from an encrypted, bounded word-and-phrase frequency model. It tokenizes by Unicode word boundaries, records bigrams and trigrams, filters candidates by any partially typed token, and returns at most three ranked suggestions. Exact token context naturally separates writing systems without claiming language-specific grammar knowledge.

`CloudAssistanceClient` talks to a user-configured OpenAI-compatible HTTP endpoint. It is absent until configuration is valid and the user enables cloud assistance.

## Checkable Text Regions

- Rich Text and Plain Text: the complete editable body.
- Markdown: prose regions; fenced code, inline code, paths, and other `no-spell-check` regions are excluded.
- Code: only GtkSourceView `comment` and `string` context classes are included. Identifiers, keywords, paths, and executable code are excluded.
- View Only and Trash: no checking, predictions, or assistance controls.

Offsets crossing excluded regions are rejected. All engine boundaries use Unicode character offsets. Byte offsets are converted before results reach GTK, preventing corruption around Bengali and other multibyte scripts.

## User Interface

### Global settings

The application menu gains **Writing Assistance**, which opens a dedicated settings window containing:

- **Spelling** — on by default;
- **Grammar** — on by default;
- **Offline predictions** — on by default;
- **Online AI assistance** — off by default;
- **Language** — Automatic by default, with installed dictionaries listed;
- provider base URL;
- provider model name;
- API key entry;
- **Test connection**;
- a privacy notice describing exactly what text can leave the device.

The cloud switch remains insensitive until the endpoint and model are valid and a connection test succeeds. Successful validation is stored as a non-secret boolean associated with the endpoint and model; editing either field or replacing/removing the key clears validation and disables cloud assistance until the next successful test. An API key is required for remote endpoints and optional for loopback endpoints.

### Per-note settings

The editor More menu gains a **Writing Assistance** section. Notes use global defaults unless **Override global settings for this note** is enabled. The override exposes spelling, grammar, offline prediction, and cloud-assistance switches. Code mode also displays the fixed explanation **Checks comments and strings only**.

Duplicating a note copies its overrides. New notes use global defaults. View Only does not modify stored settings.

### Issues and corrections

Spelling issues use a restrained red underline. Grammar and style issues use a distinct blue underline. Clicking or right-clicking an issue opens a popover containing:

- a text category such as **Spelling** or **Grammar**;
- a concise explanation;
- ordered replacement buttons;
- **Ignore once**;
- **Add to Dictionary** when the active spelling provider supports it.

Selecting a replacement performs one GTK user action so undo, autosave, status counts, and rich formatting continue to behave normally. Issue tags are transient and are never serialized into rich content.

### Predictive suggestions

The highest-ranked prediction appears as subdued inline ghost text positioned at the insertion cursor without inserting text into the buffer. Therefore it cannot enter autosave snapshots, undo history, exports, search results, or character counts.

- `Tab` accepts a visible prediction; otherwise Tab keeps its existing editor behavior.
- `Escape` dismisses a visible prediction before other Escape handling runs.
- `Alt+Down` opens up to three alternatives below the cursor.
- Arrow keys move through alternatives and `Enter` accepts the selected value.
- Cursor movement, selection changes, edits, loss of focus, switching to View Only, or disabling prediction dismisses the current suggestion.

The status bar exposes an accessible writing-assistance status: Idle, Checking, Offline, Cloud, or Unavailable. It does not display modal errors.

## Preferences and Persistence

Global settings are serialized to `writing-assistance.json` under the Noor Notes configuration directory. The store uses the project's private atomic-write pattern, creates the file with mode `0600`, validates URLs and lengths on load, and falls back to safe defaults without overwriting malformed input.

Provider base URL and model name are stored in that file. The API key is stored through the existing GNOME Keyring integration under a separate writing-assistance schema. The key is never serialized, logged, exported, or synchronized.

`EditorPreferences` gains a backward-compatible `WritingAssistanceOverrides` value with `Option<bool>` fields for spelling, grammar, offline prediction, and cloud assistance. `None` means use the current global value. Serde defaults preserve compatibility with existing notes and databases.

The local phrase model is derived data stored in a new table inside the existing SQLCipher database. The table stores a schema version, serialized bounded n-gram counts, and the revision watermark used to build them. The model is rebuilt asynchronously from active and archived notes at startup when stale, after five seconds of model-update inactivity, and after archive, trash, restore, or permanent-delete lifecycle changes. Trash never contributes. Rebuilding replaces the model atomically, so removed notes stop influencing subsequent predictions.

The model contains at most 50,000 n-gram entries. Lowest-frequency entries are discarded first. The model is local-only derived data and is not synchronized between devices; synchronized note content can contribute after a local rebuild.

## Cloud Provider Contract

The configured base URL must use HTTPS, except that `http://localhost`, `http://127.0.0.1`, and `http://[::1]` are accepted for local providers. Noor Notes appends `/v1/chat/completions` unless that path is already present.

Requests use an OpenAI-compatible chat-completions JSON shape with the configured model, deterministic temperature, an instruction message, and one scoped text message. A bearer token is added when a key exists. The response must expose `choices[0].message.content` containing JSON.

Grammar response JSON contains an `issues` array. Every issue contains `offset`, `length`, `category`, `message`, and a bounded `replacements` array. Prediction response JSON contains a `suggestions` array of at most three strings. Invalid JSON, offsets outside the submitted snippet, oversized replacements, duplicate predictions, and control characters are rejected.

Cloud grammar sends only the current paragraph, capped at 2,000 Unicode characters. Cloud prediction sends only nearby sentence context, capped at 800 Unicode characters. Both include a language hint when known. Requests never contain titles, tags, full-note metadata, complete unrelated paragraphs, or content from other notes.

Only one cloud request per assistance kind may be active for a note. Edits cancel or invalidate older requests. Timeouts, rate limits, TLS failures, malformed responses, and provider errors fall back to local results and set a non-modal status. Request and response bodies are never logged.

## Packaging

The Rust workspace adds official libspelling bindings, Harper, Unicode tokenization support, and only the minimal serialization needed by the model. Ubuntu installation and CI add the libspelling development package. Native runtime packaging adds libspelling, Enchant, and a baseline English dictionary while continuing to discover any other dictionaries installed by the user.

Snap and Flatpak manifests must include the spell-check runtime, dictionary provider, and baseline dictionary inside their sandboxes. They must not gain network access solely for local assistance; existing optional network access is used only when the user enables cloud assistance.

## Accessibility

- Issue underlines have category text in their popovers and accessible descriptions.
- Suggestion controls are keyboard reachable and expose meaningful labels.
- The ghost suggestion is announced once when it changes, without announcing on every keystroke.
- Reduced-motion preferences suppress suggestion fade transitions; functionality remains unchanged.
- Focus returns to the editor after applying or dismissing a suggestion.
- Disabled and unavailable states include explanatory text, not colour alone.

## Failure Handling

- Missing libspelling runtime or dictionary: disable spelling for that language and show Unavailable.
- Unsupported offline grammar language: retain spelling and prediction; use cloud grammar only when enabled.
- Corrupt global settings: load safe defaults and preserve the malformed file for diagnosis.
- Keyring unavailable: retain local assistance and keep cloud assistance disabled.
- Corrupt or incompatible phrase model: discard only the derived model and rebuild it from encrypted notes.
- Local engine panic or task failure: discard that result and preserve editing.
- Cloud failure: retain local results, apply bounded backoff, and never block autosave or window closing.

## Testing Strategy

Unit tests cover:

- safe global defaults and atomic preference round trips;
- malformed settings and endpoint validation;
- backward-compatible per-note overrides;
- Unicode offset conversion and excluded-region boundaries;
- English grammar results and unsupported-language fallback;
- n-gram training, ranking, partial-token filtering, the 50,000-entry bound, and multilingual scripts;
- encrypted model-store round trips, stale watermark handling, and lifecycle rebuild inputs;
- cloud request scoping, character caps, response validation, cancellation, timeout, and fallback;
- settings resolution from global values and per-note overrides.

GTK/Xvfb integration tests cover:

- libspelling attachment and toggling;
- spelling and grammar issue tags and correction user actions;
- rich formatting persistence with transient tags present;
- inline ghost rendering without buffer mutation;
- Tab acceptance, Escape dismissal, and alternatives keyboard navigation;
- View Only and Trash suppression;
- Markdown exclusions and Code comment/string filtering;
- accessible labels, focus restoration, and status changes.

Repository verification includes `cargo fmt --all -- --check`, workspace Clippy with warnings denied, the full workspace test suite, GTK tests under Xvfb, packaging manifest tests, installer tests, and a release build. Manual review covers light and dark themes, installed-language selection, offline startup, provider consent, and responsive popover placement.

## Acceptance Criteria

- New and existing notes have local spelling, grammar, and prediction enabled by default.
- Users can disable each feature globally and override it per note.
- Installed dictionaries provide spelling suggestions without network access.
- English grammar works offline; unsupported languages fail quietly and can use the optional cloud provider.
- Predictions appear inline, provide up to three alternatives, and never change note content until accepted.
- Local predictions learn from non-trashed notes and the derived model remains inside the encrypted database.
- Cloud assistance is impossible until explicitly configured, tested, and enabled.
- Cloud payload tests prove that titles, tags, complete notes, and unrelated notes are excluded.
- Code mode checks only comments and strings.
- No assistance state is serialized into rich formatting, undo history, exports, or search results.
- Editing, autosave, encryption, formatting, and offline operation continue when assistance is disabled or unavailable.

## Non-Goals

- Automatically rewriting or correcting note text.
- Claiming full offline grammar coverage for every language.
- Bundling a large local language model or Java LanguageTool server.
- Synchronizing the derived prediction model or global provider credentials.
- Sending whole notes to a cloud provider.
- Adding collaboration, document review, or tracked-change workflows.
