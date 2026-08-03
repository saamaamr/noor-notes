# Rich Note Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a modern sticky-note window with persistent structured rich text, compact formatting and emoji popovers, and backward-compatible storage and encrypted sync.

**Architecture:** The domain owns a versioned rich-document model while retaining canonical plain text. GTK-specific buffer/tag conversion and formatting commands live in focused application modules. Existing note JSON remains the storage and encrypted-sync payload, so adding a serde-defaulted optional field preserves older data and clients without a separate database column.

**Tech Stack:** Rust, Serde JSON, GTK4 `TextBuffer`/`TextTag`, Libadwaita, SQLite/SQLx, ChaCha20-Poly1305 sync envelopes, CSS

## Global Constraints

- Preserve existing content, wording, URLs, saved window dimensions, note-taking purpose, Xpad import, search, archive, trash, opacity, Always on Top, and all-workspaces behavior.
- `Note.content` remains canonical plain text and `Note.rich_content` is optional and serde-defaulted.
- Rich content is encrypted inside the existing serialized `Note` payload before upload.
- Formatting controls live in a popover rather than a permanently visible second toolbar.
- Toolbar icons are dark symbolic icons with 40–44 pixel targets; destructive controls turn red only on hover.
- Malformed or unsupported rich JSON falls back to canonical plain text without preventing the note from opening.

---

### Task 1: Versioned rich-document domain model

**Files:**
- Create: `crates/domain/src/rich_text.rs`
- Modify: `crates/domain/src/lib.rs`
- Modify: `crates/domain/src/note.rs`
- Create: `crates/domain/tests/rich_text.rs`

**Interfaces:**
- Produces: `RichDocument { version: u8, blocks: Vec<RichBlock> }`, `RichBlock { alignment, list, spans }`, `RichSpan { text, marks }`, `TextMarks`, `Alignment`, `ListKind`, `RichDocument::from_plain_text(&str)`, and `RichDocument::plain_text()`
- Extends: `Note` with `#[serde(default, skip_serializing_if = "Option::is_none")] pub rich_content: Option<RichDocument>`

- [ ] Write tests proving plain-text conversion, styled JSON round-trip, list/alignment preservation, and deserialization of an old `Note` JSON without `rich_content`.
- [ ] Run `cargo test -p noor-domain --test rich_text` and confirm failure because the types do not exist.
- [ ] Implement the minimal version-1 document model, normalized color strings, and plain-text derivation.
- [ ] Run all `noor-domain` tests and commit with `feat: add rich note document model`.

### Task 2: Storage, autosave, and sync compatibility

**Files:**
- Modify: `crates/storage/tests/repository.rs`
- Modify: `apps/noor-notes/tests/autosave.rs`
- Modify: `crates/sync/tests/conflicts.rs`
- Modify: `crates/sync/tests/offline.rs`
- Modify: `crates/sync/src/merge.rs`

**Interfaces:**
- Consumes: serde-defaulted `Note.rich_content` inside existing `payload_json` and encrypted journal payloads
- Produces: storage/autosave rich-document round trips and conflict copies that preserve the remote rich document while prefixing canonical conflict text

- [ ] Add failing repository, autosave, encrypted offline, and conflict-copy assertions for optional rich content.
- [ ] Run the focused tests and verify the conflict-copy preservation assertion fails.
- [ ] Update conflict construction to preserve or safely downgrade rich content; no SQL schema change is needed because `payload_json` already stores the complete note.
- [ ] Run focused storage, app autosave, crypto, and sync tests; commit with `feat: persist rich notes through storage and sync`.

### Task 3: GTK rich-buffer adapter and formatting commands

**Files:**
- Create: `apps/noor-notes/src/rich_buffer.rs`
- Create: `apps/noor-notes/src/formatting.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Create: `apps/noor-notes/tests/rich_editor.rs`

**Interfaces:**
- Produces: `RichBuffer::load(buffer, note)`, `RichBuffer::snapshot(buffer) -> (String, Option<RichDocument>)`, selection commands for bold/italic/underline/strikethrough, alignment/list commands, font size, foreground, highlight, URL tagging, and `insert_emoji`
- Consumes: `RichDocument` and GTK `TextBuffer`

- [ ] Add failing GTK tests for load/snapshot round-trip, selection formatting, collapsed-cursor typing marks, emoji insertion, URL tagging, and malformed-rich fallback.
- [ ] Run `xvfb-run -a cargo test -p noor-notes --test rich_editor` and confirm missing-module failures.
- [ ] Implement named `TextTag` registration, block/span loading, range inspection, snapshot conversion, typing attributes via insert-mark handling, list prefixes, alignment tags, URL detection, and emoji insertion.
- [ ] Run the focused GTK test and commit with `feat: add GTK rich text editing engine`.

### Task 4: Compact modern toolbar and popovers

**Files:**
- Rewrite: `apps/noor-notes/src/note_toolbar.rs`
- Create: `apps/noor-notes/src/format_popover.rs`
- Create: `apps/noor-notes/src/emoji_popover.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Create: `apps/noor-notes/tests/toolbar.rs`

**Interfaces:**
- Produces: left/center/right toolbar groups; `new_note`, `pin`, `format`, `emoji`, `archive`, `trash`, `settings`; formatting action buttons; searchable emoji grid; settings controls for all-workspaces and opacity
- Consumes: formatting commands from Task 3 and existing window controller capabilities

- [ ] Add failing widget-contract tests for 44-pixel targets, dark symbolic icon names, popover contents, settings relocation, and emoji insertion signal.
- [ ] Implement the three groups and both popovers with tooltips, accessible labels, and stateful formatting toggles.
- [ ] Run toolbar and rich-editor tests; commit with `feat: redesign note editing toolbar`.

### Task 5: Window integration, shortcuts, and visual system

**Files:**
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/resources/style.css`
- Modify: `apps/noor-notes/src/app.rs`
- Modify: `apps/noor-notes/tests/autosave.rs`
- Create: `apps/noor-notes/tests/note_window_ui.rs`

**Interfaces:**
- Connects: toolbar/popover actions to `RichBuffer`, autosave snapshots to `Note.content` and `Note.rich_content`, new-note action to the application, existing window controls to settings
- Produces: Ctrl+B/Ctrl+I/Ctrl+U shortcuts, warm-yellow/cream styling, dark icons, pale hover states, destructive-hover red, rounded content, subtle border/shadow, improved editor spacing, and URL color

- [ ] Add failing window contract tests for size preservation, CSS classes, shortcut actions, and rich autosave scheduling.
- [ ] Integrate the rich buffer and popovers without changing saved geometry or compositor controller behavior.
- [ ] Replace CSS with the approved visual tokens and verify selectors cover header, editor, toolbar buttons, popovers, links, and destructive hover.
- [ ] Run app tests under Xvfb and commit with `feat: deliver modern rich note window`.

### Task 6: Regression, installation, and publication

**Files:**
- Modify: `README.md` only if rich-text capabilities need documenting

**Interfaces:**
- Produces: verified release build installed at `~/.local/bin/noor-notes` and synchronized GitHub `main`

- [ ] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [ ] Run `xvfb-run -a cargo test -p noor-notes`, `xvfb-run -a cargo test -p noor-windowing`, `gjs -m extensions/gnome/tests/test-policy.js`, `bash tests/e2e/two_device_sync.sh`, and `git diff --check`.
- [ ] Build and install with `PATH=$HOME/.cargo/bin:$PATH bash scripts/install-local.sh`; smoke-test the installed binary.
- [ ] Merge the verified branch into `main`, rerun the full workspace suite, push `main`, and verify `HEAD` equals `origin/main`.
