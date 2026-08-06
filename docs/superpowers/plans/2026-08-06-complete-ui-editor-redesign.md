# Noor Notes Complete UI and Dual-Mode Editor Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace every visible Noor Notes interface with an adaptive premium native library and dual-mode Rich/Markdown editor while retaining all existing notes and encrypted storage.

**Architecture:** Extract the 603-line library window and 1,030-line editor into focused GTK components backed by pure state/services. Use GTK TextView through `RichEditorAdapter` for legacy rich notes and GtkSourceView through `SourceEditorAdapter` for Markdown/plain/code notes; both consume one `EditorSession` and autosave contract. The library uses GTK list models and factories so UI allocation remains bounded for 5,000+ notes.

**Tech Stack:** Rust 1.85+, GTK4 0.10/GTK 4.14, libadwaita 0.8/Adwaita 1.5, GtkSourceView 5, SQLCipher SQLite, Tokio, serde JSON.

## Global Constraints

- Preserve application ID, encrypted database, note IDs, rich documents, and lifecycle behavior.
- Legacy note payloads must deserialize without migration; all new fields use serde defaults.
- No analytics, telemetry, cloud API, remote asset, Snap build, Snap upload, or release action.
- Do not install GtkSourceView system packages until the user explicitly approves that machine-level change.
- Work on `main`, use test-first changes, commit focused verified phases, and push only after the full completion gate.
- The two untracked `.snap` artifacts must never be staged.

---

### Task 1: Editor-mode domain contract

**Files:**
- Modify: `crates/domain/src/note.rs`
- Modify: `crates/domain/src/lib.rs`
- Test: `crates/domain/tests/note_metadata.rs`

**Interfaces:** Produces `EditorMode`, `SourceLanguage`, `EditorViewState`, and serde-defaulted `Note.editor_mode`/view restoration metadata.

- [ ] Write failing legacy JSON, round-trip, invalid-language, and conversion-metadata tests.
- [ ] Run `cargo test -p noor-domain --test note_metadata` and confirm missing-type failures.
- [ ] Implement default `Rich` mode, validated language IDs, cursor/scroll/bookmark state, and recovery metadata without changing SQL schema.
- [ ] Rerun domain tests and `cargo test -p noor-storage --test repository --test metadata`.
- [ ] Commit `feat: add backward-compatible editor modes`.

### Task 2: Semantic design system replacement

**Files:**
- Create: `apps/noor-notes/src/ui/mod.rs`
- Create: `apps/noor-notes/src/ui/design_system.rs`
- Create: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/app.rs`
- Modify: `apps/noor-notes/resources/noor-notes.gresource.xml`
- Delete after replacement: `apps/noor-notes/resources/style.css`, `modern.css`
- Test: `apps/noor-notes/tests/design_system.rs`

**Interfaces:** Produces `PaperPalette`, semantic CSS classes, breakpoints, icon sizes, typography scale, and one stylesheet loader.

- [ ] Write failing token/palette/contrast and stylesheet-loading tests.
- [ ] Define light/dark semantic tokens and eight paper foreground/selection mappings.
- [ ] Add spacing, radius, typography, focus, hover, selected, disabled, reduced-motion, and high-contrast classes.
- [ ] Switch startup to the single new stylesheet and remove both rejected stylesheets.
- [ ] Run focused UI tests under `xvfb-run` and commit `feat: replace visual design system`.

### Task 3: Library state and virtualized collection

**Files:**
- Create: `apps/noor-notes/src/library/state.rs`
- Create: `apps/noor-notes/src/library/section.rs`
- Create: `apps/noor-notes/src/library/note_item.rs`
- Create: `apps/noor-notes/src/library/search_controller.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Test: `apps/noor-notes/tests/library_state.rs`
- Test: `apps/noor-notes/tests/library_performance.rs`

**Interfaces:** Produces `LibraryState`, `LibrarySection`, `NoteListItem`, debounced/cancellable filtering, counts, and stable sort keys independent of widgets.

- [ ] Write failing tests for all seven sections, counts, tags, pinned/favorite, stale search cancellation, and 5,000-note filtering.
- [ ] Implement pure projection/filter/sort logic and generation cancellation.
- [ ] Prove the 5,000-note fixture remains bounded and deterministic.
- [ ] Commit `feat: add scalable library state`.

### Task 4: Completely replace the main window

**Files:**
- Create: `apps/noor-notes/src/ui/library_window.rs`
- Create: `apps/noor-notes/src/ui/library_sidebar.rs`
- Create: `apps/noor-notes/src/ui/note_collection.rs`
- Create: `apps/noor-notes/src/ui/note_card.rs`
- Create: `apps/noor-notes/src/ui/note_preview.rs`
- Create: `apps/noor-notes/src/ui/empty_state.rs`
- Replace: `apps/noor-notes/src/main_window.rs` with a compatibility facade
- Modify: `apps/noor-notes/src/managed_app.rs`
- Test: `apps/noor-notes/tests/library_ui.rs`

**Interfaces:** Consumes `LibraryState`; produces `LibraryWindow` with refresh/search/present/status API compatible with application actions.

- [ ] Write failing widget-tree and accessibility tests proving the old ViewStack/boxed-list hierarchy is absent.
- [ ] Build adaptive native header with New, expandable search, sort menu, application menu, and controls only.
- [ ] Build navigation sidebar with section counts and keyboard selection.
- [ ] Build `gio::ListStore` + filter/sort model + `gtk::ListView` factory and modern cards with two tags maximum.
- [ ] Add selection-driven preview, context actions, confirmations, and narrow navigation transitions.
- [ ] Validate Notes/Pinned/Favorites/Tags/Recent/Archive/Trash flows against a disposable repository.
- [ ] Commit `feat: replace notes library interface`.

### Task 5: Shared editor session and adapter contract

**Files:**
- Create: `apps/noor-notes/src/editor/mod.rs`
- Create: `apps/noor-notes/src/editor/session.rs`
- Create: `apps/noor-notes/src/editor/adapter.rs`
- Create: `apps/noor-notes/src/editor/search.rs`
- Create: `apps/noor-notes/src/editor/statistics.rs`
- Create: `apps/noor-notes/src/editor/autosave_controller.rs`
- Test: `apps/noor-notes/tests/editor_session.rs`

**Interfaces:** Produces `EditorAdapter` capabilities and `EditorSession` save/dirty/find/view state shared by both widgets.

- [ ] Write failing fake-adapter tests for five-step undo/redo, search, replace, statistics, save transitions, flush-on-close, and recovery.
- [ ] Implement capability-driven commands so unsupported actions cannot appear enabled.
- [ ] Move duplicated save/search/statistics logic out of window widgets.
- [ ] Commit `feat: add editor session architecture`.

### Task 6: Completely replace rich-note editor UI

**Files:**
- Create: `apps/noor-notes/src/editor/rich_adapter.rs`
- Create: `apps/noor-notes/src/ui/editor_window.rs`
- Create: `apps/noor-notes/src/ui/editor_header.rs`
- Create: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Create: `apps/noor-notes/src/ui/editor_canvas.rs`
- Create: `apps/noor-notes/src/ui/editor_status_bar.rs`
- Create: `apps/noor-notes/src/ui/find_panel.rs`
- Create: `apps/noor-notes/src/ui/tag_chips.rs`
- Replace: `apps/noor-notes/src/note_window.rs` with a compatibility facade
- Delete: `apps/noor-notes/src/modern_toolbar.rs`
- Test: `apps/noor-notes/tests/rich_editor_ui.rs`

**Interfaces:** Implements `EditorAdapter` over existing `RichBuffer`; produces the entirely new editor shell.

- [ ] Write failing hierarchy and action tests proving the old toolbar/layout are absent.
- [ ] Build full-title header, save state, pin, palette, and overflow.
- [ ] Build the eight-action 36–40 px toolbar and capability-driven menus.
- [ ] Build centered neutral writing canvas, tag chips, live status segments, and adaptive margins.
- [ ] Wire shortcuts, find/replace, formatting, wrap, zoom, autosave, close safety, archive/trash/export.
- [ ] Verify rich formatting save-close-reopen against a disposable encrypted repository.
- [ ] Commit `feat: replace rich note editor`.

### Task 7: GtkSourceView dependency gate and source adapter

**Files:**
- Modify after approval: root `Cargo.toml`, `Cargo.lock`, `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/editor/source_adapter.rs`
- Create: `apps/noor-notes/src/editor/languages.rs`
- Test: `apps/noor-notes/tests/source_editor.rs`

**Interfaces:** Implements `EditorAdapter` with GtkSourceView search context, language manager, undo manager, line display, marks, current-line highlight, and regex.

- [ ] Request explicit approval to install the Ubuntu GtkSourceView 5 development package; stop this task if unavailable.
- [ ] Add the compatible gtk-sourceview Rust crate and compile-only dependency test.
- [ ] Write failing tests for Markdown/plain/code modes, regex, match highlighting, line numbers, bookmarks, language validation, and Unicode.
- [ ] Implement source adapter and settings without altering Snap metadata.
- [ ] Run focused source tests under Xvfb and commit `feat: add markdown and code editor modes`.

### Task 8: Safe mode conversion and multi-document sessions

**Files:**
- Create: `apps/noor-notes/src/editor/conversion.rs`
- Create: `apps/noor-notes/src/services/session_store.rs`
- Create: `apps/noor-notes/src/services/recent_items.rs`
- Modify: `apps/noor-notes/src/ui/editor_window.rs`
- Test: `apps/noor-notes/tests/editor_conversion.rs`
- Test: `apps/noor-notes/tests/session_restore.rs`

**Interfaces:** Produces previewable conversion, recovery snapshot, editor tabs, recent notes/files, and validated session restoration.

- [ ] Write failing loss-warning, recovery, invalid-session, tab-close, and recent-order tests.
- [ ] Implement explicit conversion preview/confirmation and recovery copy.
- [ ] Add editor-only tabs with dirty markers, safe close, reorder, reopen, and separate-window action.
- [ ] Restore only valid note IDs and clamp cursor/scroll/window values.
- [ ] Commit `feat: add safe editor modes and sessions`.

### Task 9: Full accessibility, performance, and visual verification

**Files:**
- Modify focused components based on findings
- Create: `apps/noor-notes/tests/redesign_accessibility.rs`
- Create temporary visual harness, then remove it before commit
- Output screenshots only under `/tmp/noor-notes-complete-redesign/`

**Interfaces:** Produces measured evidence and ten fresh screenshot artifacts, not production dependencies.

- [ ] Run keyboard-only flows for all navigation, editor, search, dialogs, and tabs.
- [ ] Verify screen-reader labels, focus order/rings, high contrast, reduced motion, 125/150/200% scaling.
- [ ] Exercise five rich/source notes across all sections, long Unicode content, autosave failure, and 5,000-note search.
- [ ] Capture Main, Sidebar, Preview, Editor, Find, Dark, Light, Narrow, Trash, and Pinned screenshots from actual widgets.
- [ ] Remove harness and verify no generated data, screenshots, logs, credentials, or binaries are staged.

### Task 10: Completion gate, installation, commit, and push

**Files:** All planned source/tests; no Snap or release artifacts.

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test --workspace` and all GUI tests under Xvfb where required.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo build --release` and `git diff --check`.
- [ ] Review every changed file and compare fresh screenshots against the rejected baseline.
- [ ] Install only the verified user-local binary, then manually relaunch it without modifying user data.
- [ ] Commit focused remaining changes, push `main`, verify local/origin hashes, and report untracked Snap artifacts separately.
