# Document Editor Toolbar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Give Noor Notes a two-level, mode-aware document editor toolbar with shared command behavior while preserving existing note editing and storage.

**Architecture:** Keep `EditorToolbar` as the shared GTK toolbar used by the current preview and legacy editor paths. Add a small `EditorMenuBar` UI and central command helpers in `editor_actions.rs`; reuse existing `RichBuffer`, popovers, mode buttons, and shortcuts rather than introducing a new editor dependency.

**Tech Stack:** Rust, GTK4, Libadwaita, existing CSS token system, current `RichBuffer` and `EditorToolbar` components.

**Spec:** User-provided Google Docs-inspired Noor Notes editor toolbar requirements (2026-08-19).

## Global Constraints

- Preserve note storage, autosave, editor modes, Rich Text margins, themes, and keyboard behavior.
- Reuse current GTK icon library and existing formatting popover.
- Expose only commands already implemented by Noor Notes.
- Keep toolbar compact and responsive; secondary actions remain in More menus.
- Do not add dependencies or rewrite the editor widget.

### Task 1: Centralize reusable editor command helpers

**Files:**
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Test: `apps/noor-notes/tests/rich_editor_ui.rs`

- [ ] Add small public helper functions for undo, redo, find activation, zoom, list toggles, and rich formatting that call existing `RichBuffer`/toolbar actions.
- [ ] Route existing toolbar callbacks and keyboard handlers through those helpers without changing shortcut semantics.
- [ ] Add source-level assertions covering shared command helper usage and existing toolbar accessibility.
- [ ] Run the focused editor tests and formatting checks.

### Task 2: Add reusable two-level menu bar

**Files:**
- Create: `apps/noor-notes/src/ui/editor_menu_bar.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Test: `apps/noor-notes/tests/rich_editor_ui.rs`

- [ ] Implement `EditorMenuBar` with File, Edit, View, Insert, Format, and Tools menu buttons.
- [ ] Populate only existing actions: new/duplicate/export/delete, undo/redo/find, zoom/wrap, emoji, formatting, mode selector, go-to-line, settings, and view-only.
- [ ] Wire menu items to the shared command callbacks supplied by the editor surface; do not duplicate persistence logic.
- [ ] Add semantic labels, keyboard focus, compact spacing, and a light/dark token-based style.
- [ ] Keep the menu bar content-fit and avoid forcing the editor width.

### Task 3: Make toolbar mode-aware and document-editor styled

**Files:**
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/ui/formatting_popover.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Test: `apps/noor-notes/tests/rich_editor_ui.rs`

- [ ] Add a compact style/mode selector and contextual visibility method for Rich, Markdown, Plain Text, and Code modes.
- [ ] Keep Rich Text quick controls (undo, redo, style, size, B/I/U, formatting, emoji) visible; move advanced colors/alignment/clear formatting to the popover.
- [ ] Hide unsupported Rich Text controls in Markdown, Plain Text, and Code modes while retaining each mode’s existing actions.
- [ ] Add grouped separators, active/open states, tooltips, accessible labels, and compact responsive overflow behavior.
- [ ] Preserve the existing Rich Text 5px vertical / 8px horizontal editor margin.

### Task 4: Integrate menu bar and toolbar into both editor surfaces

**Files:**
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/ui/editor_header.rs`
- Test: `apps/noor-notes/tests/preview_editor_surface.rs`
- Test: `apps/noor-notes/tests/editor_presentation.rs`

- [ ] Place the menu row and formatting row in one aligned content container with title, metadata, and editor body.
- [ ] Connect mode changes to contextual toolbar visibility in preview and legacy editor paths.
- [ ] Ensure read-only mode hides editing chrome while sticky title/body behavior remains unchanged.
- [ ] Verify no duplicate GTK parent assignments or hydration-like widget reuse errors occur.

### Task 5: Verify and document the finished toolbar

**Files:**
- Modify: `README.md` only if the editor controls or shortcuts require user-facing documentation.

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run focused editor, preview, accessibility, and presentation tests.
- [ ] Run `cargo check -p noor-notes` and the production build path.
- [ ] Manually verify all four modes, formatting popover, shortcuts, read-only, light/dark themes, and narrow window behavior.
- [ ] Inspect `git diff --check` and ensure no unrelated files are included.

