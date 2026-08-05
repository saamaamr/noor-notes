# Noor Notes Quality and Productivity Upgrade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a reliable, accessible, productivity-focused Noor Notes release while preserving existing note data and the compact visual identity.

**Architecture:** Extend the domain and SQLite layers with additive compatible metadata, centralize observable persistence in the autosave queue, and keep editor commands in focused pure/testable modules. GTK windows compose these services and expose state without owning data transformations.

**Tech Stack:** Rust 2024 workspace, GTK4, Libadwaita, Tokio, SQLx/SQLite, Serde, Xvfb, Cargo tests.

## Global Constraints

- Preserve existing notes and use additive database migrations only.
- Preserve the warm-yellow default, 28 px note toolbar controls, 12 px icons, and 3 px note radius.
- Cloud synchronization remains unavailable in user-facing claims until a configured server/account workflow exists.
- Every production behavior change follows red-green-refactor.
- Preserve the unrelated `noor-notes_0.1.0_amd64.snap` artifact.

---

### Task 1: Observable autosave and close-time safety

**Files:**
- Modify: `apps/noor-notes/src/autosave.rs`
- Create: `apps/noor-notes/src/save_status.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Test: `apps/noor-notes/tests/autosave.rs`
- Create: `apps/noor-notes/tests/save_status.rs`

**Interfaces:**
- Produces: `SaveState::{Idle, Saving, Saved, Failed(String)}` and `AutosaveQueue::subscribe(NoteId) -> watch::Receiver<SaveState>`.
- Produces: `AutosaveQueue::retry(NoteId) -> Result<(), StorageError>` and close-request flushing.

- [ ] Add failing tests proving schedule transitions to Saving then Saved, failed saves retain the latest draft, retry persists it, and close-time flush failure is surfaced.
- [ ] Run `cargo test -p noor-notes --test autosave --test save_status` and confirm failures describe missing observable state/retry behavior.
- [ ] Implement per-note pending drafts and watch channels; never remove a draft until its repository write succeeds.
- [ ] Add a compact save-state label and Retry button to the note title row; connect `close-request` to async flush and inhibit close on failure.
- [ ] Run focused tests, then `cargo test -p noor-notes`.
- [ ] Commit with `git commit -m "fix: make note saving observable and recoverable"`.

### Task 2: Data model, migrations, tags, colours, sorting, and duplication

**Files:**
- Modify: `crates/domain/src/note.rs`
- Create: `crates/domain/src/note_metadata.rs`
- Modify: `crates/domain/src/lib.rs`
- Create: `crates/storage/migrations/0003_note_metadata.sql`
- Modify: `crates/storage/src/repository.rs`
- Modify: `crates/storage/src/lib.rs`
- Test: `crates/domain/tests/note_model.rs`
- Create: `crates/storage/tests/metadata.rs`
- Create: `crates/storage/tests/legacy_migrations.rs`

**Interfaces:**
- Produces: `NoteColor` closed enum with yellow default and CSS class mapping.
- Produces: `Note::set_tags(Vec<String>)`, normalized case-insensitive uniqueness, and `Note::duplicate(DateTime<Utc>) -> Note`.
- Produces: `NoteSort::{UpdatedDesc, TitleAsc, CreatedDesc}` and `SqliteNoteRepository::search_notes_sorted(query, sort)`.

- [ ] Write failing domain tests for legacy JSON defaults, tag trimming/deduplication, colour serialization, and duplicate-note identity/state/timestamps.
- [ ] Write failing storage tests that open schema versions 0001 and 0002, migrate without content loss, search tags, sort deterministically, and duplicate transactionally.
- [ ] Run the focused domain/storage tests and verify expected failures.
- [ ] Implement metadata types and the additive migration with indexed tag rows or normalized searchable metadata.
- [ ] Implement explicit sorting and transactional duplication without interpolating unchecked SQL.
- [ ] Run `cargo test -p noor-domain -p noor-storage` and commit with `git commit -m "feat: add durable note metadata and sorting"`.

### Task 3: Rich editor correctness, undo/redo, and formatting state

**Files:**
- Modify: `apps/noor-notes/src/rich_buffer.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/src/modern_toolbar.rs`
- Modify: `apps/noor-notes/tests/list_editing.rs`
- Modify: `apps/noor-notes/tests/rich_editor.rs`
- Create: `apps/noor-notes/tests/editor_history.rs`

**Interfaces:**
- Produces: `FormattingState` derived from cursor/selection.
- Produces: `RichBuffer::undo`, `redo`, `can_undo`, `can_redo`, and consistent list transformations.

- [ ] Add failing tests for cursor-only list toggling, partial/multiline selection, reversed selection, list switching, Unicode, numbering continuation, empty exit, and formatting state.
- [ ] Add failing undo/redo tests proving edits and formatting operations reverse and reapply.
- [ ] Run GTK tests under `xvfb-run -a cargo test -p noor-notes --test list_editing --test rich_editor --test editor_history` and verify failures.
- [ ] Implement list operations around complete logical lines without losing unrelated text tags; serialize list semantics consistently.
- [ ] Enable GTK buffer undo and expose dark 12 px undo/redo controls with sensitivity bound to history state.
- [ ] Synchronize toolbar checked states without invoking edit callbacks.
- [ ] Re-run GTK tests and commit with `git commit -m "feat: harden rich editing and add history"`.

### Task 4: Find, export, and note commands

**Files:**
- Create: `apps/noor-notes/src/note_find.rs`
- Create: `apps/noor-notes/src/export.rs`
- Create: `apps/noor-notes/src/note_commands.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Create: `apps/noor-notes/tests/note_find.rs`
- Create: `apps/noor-notes/tests/export.rs`
- Create: `apps/noor-notes/tests/note_commands.rs`

**Interfaces:**
- Produces: `FindResults { ranges, current }` with case-insensitive next/previous navigation.
- Produces: `export_plain(&Note) -> String` and `export_markdown(&Note) -> String`.
- Produces note-window actions `win.find`, `win.rename`, `win.archive`, `win.duplicate`, and `win.export`.

- [ ] Write failing pure tests for Unicode-safe matching, wraparound, zero matches, plain export, and Markdown marks/lists fallback.
- [ ] Write failing command contract tests for shortcuts and action availability.
- [ ] Run focused tests and confirm failures.
- [ ] Implement pure find/export modules, then compose an inline search bar with count and previous/next buttons.
- [ ] Add a native save dialog for `.txt` and `.md`; report errors without mutating note state.
- [ ] Wire commands and accelerators including Ctrl+F, Ctrl+Shift+S, and duplicate.
- [ ] Re-run focused tests and commit with `git commit -m "feat: add find export and note commands"`.

### Task 5: Library productivity and polished empty states

**Files:**
- Modify: `apps/noor-notes/src/main_window.rs`
- Create: `apps/noor-notes/src/library_preferences.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/resources/modern.css`
- Create: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/tests/search.rs`

**Interfaces:**
- Consumes: `NoteSort`, note tags, colour, and repository sorted search.
- Produces: persisted local sort preference and tag-aware library rows.

- [ ] Add failing search tests for title/body/tag matches and failing UI contract tests for every empty state and sort choice.
- [ ] Run focused tests and verify failures.
- [ ] Add sort control, tag chips, colour indicators, and separate empty-state pages for Notes, Archived, Trash, and no search results.
- [ ] Persist sort selection using `gio::Settings` when schema installation is available, with a safe in-memory default otherwise.
- [ ] Add Duplicate Note to row context actions and refresh after successful persistence.
- [ ] Run focused tests and commit with `git commit -m "feat: improve note library productivity"`.

### Task 6: Accessible responsive note styling and colour picker

**Files:**
- Modify: `apps/noor-notes/src/modern_toolbar.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/resources/modern.css`
- Modify: `apps/noor-notes/tests/compact_ui.rs`
- Create: `apps/noor-notes/tests/accessibility.rs`

**Interfaces:**
- Consumes: `NoteColor` CSS mapping.
- Produces: colour picker and accessible/responsive control contract.

- [ ] Add failing tests checking accessible names/tooltips, focusability, destructive separation, exact compact dimensions, palette classes, and narrow-layout fallback.
- [ ] Run GTK/UI contract tests and verify failures.
- [ ] Implement the curated colour picker with contrast-safe palette classes while retaining yellow as default.
- [ ] Group secondary commands into an overflow menu at narrow widths while keeping title, formatting, and close controls reachable.
- [ ] Add strong keyboard focus rings and complete accessible descriptions.
- [ ] Re-run tests and commit with `git commit -m "feat: polish accessible note interface"`.

### Task 7: Shortcuts reference and import hardening

**Files:**
- Modify: `apps/noor-notes/src/managed_app.rs`
- Create: `apps/noor-notes/resources/shortcuts.ui`
- Modify: `apps/noor-notes/resources/noor-notes.gresource.xml`
- Modify: `crates/xpad-import/src/parser.rs`
- Modify: `crates/xpad-import/tests/import.rs`
- Create: `apps/noor-notes/tests/shortcuts.rs`

**Interfaces:**
- Produces: `app.shortcuts` window and complete accelerator registration.
- Produces: panic-free Xpad path parsing with typed skipped-file reasons.

- [ ] Add a failing parser test using a path without a filename and a failing shortcuts contract test covering all documented actions.
- [ ] Run `cargo test -p noor-xpad-import` and the shortcuts test to observe failures.
- [ ] Replace filename unwraps with validated path handling and retain import preview reporting.
- [ ] Add the shortcuts resource, application action, help-menu entry, and accelerators.
- [ ] Re-run tests and commit with `git commit -m "feat: add shortcuts help and harden import"`.

### Task 8: Documentation, packaging, final verification, and installation

**Files:**
- Modify: `README.md`
- Modify: `packaging/snap/snapcraft.yaml` if resource/schema installation requires it
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml` if resource/schema installation requires it
- Modify: `scripts/install-local.sh`
- Modify: `scripts/install-ubuntu.sh`
- Modify: relevant `tests/*.sh` packaging contracts

**Interfaces:**
- Consumes all completed features and limitations.
- Produces a verified source install and package metadata with accurate documentation.

- [ ] Add failing packaging/install assertions for new resources, schemas, and desktop integration.
- [ ] Run the relevant shell contract tests and confirm failures.
- [ ] Update install/package manifests and README sections for features, shortcuts, tags, colours, export, recovery, and current sync limitations.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run `cargo test --workspace` and GTK/X11 tests under Xvfb.
- [ ] Run `gjs -m extensions/gnome/tests/test-policy.js`, `bash tests/e2e/two_device_sync.sh`, packaging tests, and `git diff --check`.
- [ ] Commit with `git commit -m "docs: document quality and productivity upgrade"`.
- [ ] Use the branch-finishing workflow, verify the merged result again, and run `PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh`.
