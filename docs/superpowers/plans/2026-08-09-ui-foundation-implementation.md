# Noor Notes UI Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reconstruct Noor Notes' library and editor presentation into a calm, adaptive, accessible desktop interface without changing note persistence or rich-document storage.

**Architecture:** Preserve the repository, autosave, editor adapters, and library state as authoritative services. Extract presentation builders from `note_window.rs`, replace wrapping controls with priority-based compact components, and route every theme through semantic CSS tokens. Characterization and behavior tests precede each production change.

**Tech Stack:** Rust 1.85+, GTK4 4.14, libadwaita 1.5, GtkSourceView 5, SQLx SQLite/SQLCipher, Tokio, project integration tests.

## Global Constraints

- Preserve existing notes, settings, SQLite schema, application ID, autosave, recovery, and mode-conversion behavior.
- Do not add network services, telemetry, analytics, remote assets, or machine-level dependencies.
- Do not modify Snap packaging or perform Snap Store actions.
- Do not add a second UI toolkit or replace GtkSourceView.
- Use failing tests before each production behavior change.
- Keep generated files, screenshots, local databases, Snap artifacts, and temporary audit data out of commits.

---

## File structure

- `ui/adaptive_layout.rs`: pure breakpoint classification and pane visibility contract.
- `ui/editor_header.rs`: title, save state, metadata, pin/favorite/appearance/overflow header widgets.
- `ui/editor_status_bar.rs`: live status presentation and compact field visibility.
- `ui/formatting_popover.rs`: grouped rich-formatting and color controls.
- `ui/editor_toolbar.rs`: non-wrapping primary actions and overflow composition.
- `ui/library_window.rs`: library coordinator using adaptive layout.
- `ui/library_sidebar.rs`: navigation presentation and collapsed mode.
- `ui/note_card.rs`: accessible note-card presentation and context actions.
- `ui/note_preview.rs`, `ui/empty_state.rs`: reading surface and contextual empty states.
- `note_window.rs`: lifecycle/action coordinator consuming extracted components.
- `resources/design-system.css`: semantic theme tokens and component states.

### Task 1: Semantic design token contract

**Files:**
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/tests/dark_palette_css.rs`

**Interfaces:**
- Produces CSS tokens `@nn_app_bg`, `@nn_surface`, `@nn_surface_raised`, `@nn_editor_bg`, `@nn_sidebar_bg`, `@nn_text`, `@nn_text_secondary`, `@nn_text_muted`, `@nn_border`, `@nn_border_subtle`, `@nn_accent`, `@nn_accent_hover`, `@nn_accent_soft`, `@nn_danger`, `@nn_warning`, `@nn_success`, `@nn_focus`, `@nn_hover`, and `@nn_selected`.
- Produces shared classes used by all later presentation tasks.

- [ ] **Step 1: Extend CSS contract tests and verify RED**

Add assertions that every semantic token occurs in the base palette and every named theme assigns component colors through tokens rather than component-specific literal backgrounds. Assert the presence of `.nn-writing-canvas`, `.nn-source-canvas`, `.nn-note-card:selected`, `.nn-sidebar-row:selected`, and `.nn-focus-ring` token consumers.

Run: `cargo test -p noor-notes --test design_system --test dark_palette_css`
Expected: FAIL because the expanded token set and consumers do not yet exist.

- [ ] **Step 2: Implement the token layer and verify GREEN**

Define the missing semantic colors and shared geometry variables at the top of `design-system.css`. Rewrite Light, Graphite, Midnight, and OLED blocks to assign semantic values. Migrate library/editor selectors to those values and remove only legacy rules whose migrated call sites are proven unused.

Run: `cargo test -p noor-notes --test design_system --test dark_palette_css`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/resources/design-system.css apps/noor-notes/tests/design_system.rs apps/noor-notes/tests/dark_palette_css.rs
git commit -m "style: establish semantic Noor Notes design tokens"
```

### Task 2: Adaptive library layout model

**Files:**
- Create: `apps/noor-notes/src/ui/adaptive_layout.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Create: `apps/noor-notes/tests/adaptive_layout.rs`

**Interfaces:**
- Produces `LibraryLayoutMode::{Wide, Medium, Narrow}`.
- Produces `LibraryLayoutMode::for_width(width: i32) -> LibraryLayoutMode`.
- Produces `LibraryPaneVisibility { sidebar: bool, collection: bool, content: bool, back: bool }` via `visibility(showing_content: bool)`.

- [ ] **Step 1: Write breakpoint behavior tests and verify RED**

Test that widths at 1199/1200 and 759/760 select deterministic modes; Wide exposes all panes, Medium supports collapsible sidebar with collection/content, and Narrow displays collection or content with Back based on `showing_content`.

Run: `cargo test -p noor-notes --test adaptive_layout`
Expected: FAIL because `adaptive_layout` is absent.

- [ ] **Step 2: Implement pure layout policy and verify GREEN**

Add the enum and visibility value object without GTK dependencies. Integrate it into `MainWindow` width notifications so preview visibility is no longer a one-off `width >= 920` boolean. Preserve note activation and selection behavior.

Run: `cargo test -p noor-notes --test adaptive_layout --test library_ui`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/src/ui/adaptive_layout.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/tests/adaptive_layout.rs
git commit -m "feat: add adaptive library pane policy"
```

### Task 3: Calm sidebar and note-card presentation

**Files:**
- Modify: `apps/noor-notes/src/ui/library_sidebar.rs`
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/src/ui/note_collection.rs`
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/tests/note_card_archive.rs`
- Modify: `apps/noor-notes/tests/accessibility.rs`

**Interfaces:**
- Adds `LibrarySidebar::set_collapsed(bool)`.
- Preserves `connect_selected` and `set_count`.
- Preserves `CardAction` storage semantics and collection activation callbacks.

- [ ] **Step 1: Add presentation and accessibility tests and verify RED**

Assert sidebar rows retain labels/counts and expose collapsed tooltips, cards contain a dedicated color indicator and separate metadata/tag regions, trash destructive actions remain context-only, and icon controls have accessible labels.

Run: `cargo test -p noor-notes --test library_ui --test note_card_archive --test accessibility`
Expected: FAIL on missing collapsed/sidebar/card contracts.

- [ ] **Step 2: Recompose sidebar and cards and verify GREEN**

Use compact rows with a narrow selection indicator, secondary counts, and icon-only collapsed state. Recompose cards into title/action, preview, tags, and metadata rows. Keep permanent deletion behind the existing confirmation path and do not change `CardAction` persistence.

Run: `cargo test -p noor-notes --test library_ui --test note_card_archive --test accessibility --test library_state`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/src/ui/library_sidebar.rs apps/noor-notes/src/ui/note_card.rs apps/noor-notes/src/ui/note_collection.rs apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/note_card_archive.rs apps/noor-notes/tests/accessibility.rs
git commit -m "feat: redesign library navigation and note cards"
```

### Task 4: Library preview, empty states, and shell integration

**Files:**
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/src/ui/empty_state.rs`
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/tests/search.rs`

**Interfaces:**
- Preserves `NotePreview::show_note(&Note)`.
- Adds `NotePreview::clear()` and `MainWindow` narrow Back behavior.
- Preserves debounced generation-based global search.

- [ ] **Step 1: Add shell/empty-state tests and verify RED**

Test contextual copy for every `LibrarySection`, no-result search state, preview clear behavior, and presence of semantic Light-theme surfaces rather than literal black preview backgrounds.

Run: `cargo test -p noor-notes --test library_ui --test search`
Expected: FAIL on the new preview/empty-state contracts.

- [ ] **Step 2: Implement integrated shell and verify GREEN**

Replace fixed navigation composition with adaptive pane presentation, a constrained readable preview, explicit empty states, and narrow Back navigation. Keep search/sort state and repository refresh logic intact.

Run: `cargo test -p noor-notes --test library_ui --test search --test library_performance`
Expected: PASS, including the 5,000-note projection test.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/src/ui/library_window.rs apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/src/ui/empty_state.rs apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/search.rs
git commit -m "feat: integrate adaptive library shell and preview"
```

### Task 5: Extract editor header and status bar

**Files:**
- Create: `apps/noor-notes/src/ui/editor_header.rs`
- Create: `apps/noor-notes/src/ui/editor_status_bar.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/tests/save_indicator.rs`
- Modify: `apps/noor-notes/tests/editor_status.rs`
- Modify: `apps/noor-notes/tests/accessibility.rs`

**Interfaces:**
- Produces `EditorHeader::new()` with public title entry, save label, pin/favorite, appearance, More, archive, and trash controls required by existing wiring.
- Produces `EditorStatusBar::new()` and `update(&EditorStatus, EditorMode, SaveState)`.

- [ ] **Step 1: Add component contract tests and verify RED**

Test full-title expansion, explicit accessible names for title/tags/body, textual save states, and mode-relevant status fields. Keep existing `EditorStatus` Unicode line/column/word behavior assertions.

Run: `cargo test -p noor-notes --test save_indicator --test editor_status --test accessibility`
Expected: FAIL because extracted components do not exist.

- [ ] **Step 2: Extract presentation and verify GREEN**

Move header/status widget construction from `note_window.rs` into focused modules. Retain existing callbacks and save state machine. Give title horizontal priority and present tags as secondary metadata. Do not move repository calls into these components.

Run: `cargo test -p noor-notes --test save_indicator --test editor_status --test accessibility --test autosave`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/src/ui/editor_header.rs apps/noor-notes/src/ui/editor_status_bar.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/note_window.rs apps/noor-notes/tests/save_indicator.rs apps/noor-notes/tests/editor_status.rs apps/noor-notes/tests/accessibility.rs
git commit -m "refactor: extract editor header and status presentation"
```

### Task 6: Non-wrapping toolbar and grouped formatting

**Files:**
- Create: `apps/noor-notes/src/ui/formatting_popover.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/tests/compact_ui.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`
- Modify: `apps/noor-notes/tests/rich_editor_ui.rs`

**Interfaces:**
- Changes `EditorToolbar::widget` from `gtk::FlowBox` to a non-wrapping `gtk::Box`.
- Preserves existing public action handles while moving secondary actions into structured overflow content.
- Produces `FormattingPopover` exposing the existing persisted mark, alignment, font, foreground, highlight, and clear controls.

- [ ] **Step 1: Add compact toolbar tests and verify RED**

Assert the primary toolbar is not a `FlowBox`, primary actions preserve accessible tooltips, formatting groups have labels, and unsupported schema-dependent operations are not presented as working controls.

Run: `cargo test -p noor-notes --test compact_ui --test toolbar_actions --test rich_editor_ui`
Expected: FAIL because the current toolbar wraps and formatting is an unlabeled grid.

- [ ] **Step 2: Implement compact composition and verify GREEN**

Use horizontal action groups with separators and a stable More menu. Move lower-priority actions out of the permanent row. Extract formatting groups while preserving existing signal handles and rich-format persistence. Ensure source modes disable rich-only controls.

Run: `cargo test -p noor-notes --test compact_ui --test toolbar_actions --test rich_editor_ui --test rich_formatting_persistence`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/src/ui/formatting_popover.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/note_window.rs apps/noor-notes/tests/compact_ui.rs apps/noor-notes/tests/toolbar_actions.rs apps/noor-notes/tests/rich_editor_ui.rs
git commit -m "feat: replace wrapping editor toolbar"
```

### Task 7: Calm editor canvas and mode-safe theme styling

**Files:**
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/editor/source_palette.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/tests/source_palettes.rs`
- Modify: `apps/noor-notes/tests/dark_palette_css.rs`
- Modify: `apps/noor-notes/tests/rich_editor_ui.rs`

**Interfaces:**
- Preserves the current rich buffer and `SourceEditorAdapter` storage conversion.
- Makes source palette resolution depend on the semantic appearance theme.
- Adds `nn-writing-canvas` and `nn-source-canvas` presentation classes.

- [ ] **Step 1: Add palette/canvas tests and verify RED**

Test readable foreground/background/selection combinations for Light, Graphite, Midnight, and OLED; assert Rich Text receives a constrained neutral paper surface while source modes use source-specific semantic surfaces.

Run: `cargo test -p noor-notes --test source_palettes --test dark_palette_css --test rich_editor_ui`
Expected: FAIL on the new semantic palette contracts.

- [ ] **Step 2: Implement theme-safe canvases and verify GREEN**

Apply mode-specific classes during editor-mode transitions. Map GtkSourceView palette colors from the active appearance and retain syntax schemes, gutters, current-line highlight, selection, and cursor contrast. Keep note colors as restrained tint classes.

Run: `cargo test -p noor-notes --test source_palettes --test dark_palette_css --test rich_editor_ui --test source_editor`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add apps/noor-notes/src/note_window.rs apps/noor-notes/src/editor/source_palette.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/source_palettes.rs apps/noor-notes/tests/dark_palette_css.rs apps/noor-notes/tests/rich_editor_ui.rs
git commit -m "style: create calm theme-safe editor canvases"
```

### Task 8: Regression, visual, and delivery verification

**Files:**
- Modify only tests or documentation needed to record verified behavior.
- Create screenshots only under an ignored temporary verification directory; do not commit them unless explicitly requested.

**Interfaces:**
- No production API changes.

- [ ] **Step 1: Run automated verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release
git diff --check
```

Expected: every command exits 0 with no warnings promoted to errors.

- [ ] **Step 2: Run isolated manual verification**

Launch with isolated `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, and `XDG_CACHE_HOME`. Create sample notes and check library sections, search/no-result states, Rich/Markdown/Plain/Code modes, find/replace, formatting persistence, word wrap, zoom, view-only, sticky behavior, keyboard navigation, four themes, and narrow/medium/wide windows. Capture new before/after evidence outside tracked paths.

- [ ] **Step 3: Verify user-data isolation**

Record the production database hash before and after the isolated session and require equality. Confirm no migration occurred and no network or Snap action was performed.

- [ ] **Step 4: Review and commit final corrections**

```bash
git status --short
git diff --check
git add apps/noor-notes/src/ui apps/noor-notes/src/note_window.rs apps/noor-notes/src/editor/source_palette.rs apps/noor-notes/tests apps/noor-notes/resources/design-system.css docs/superpowers/plans/2026-08-09-ui-foundation-implementation.md
git commit -m "test: verify redesigned Noor Notes foundation"
```

- [ ] **Step 5: Push verified main**

```bash
git push origin main
git rev-parse HEAD
git rev-parse origin/main
git status --short
```

Expected: local and remote hashes match; only the two pre-existing Snap artifacts remain untracked.
