# Preview Editor and Sticky Window Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move normal note editing into the library Preview Body, use a separate read-only sticky window only when requested, and keep archive/trash actions from closing the application.

**Architecture:** Extract the document/editor behavior currently concentrated in `NoteWindow` into a reusable `NoteEditorSurface`. `NotePreview` hosts the editable surface inside `MainWindow`; `StickyNoteWindow` hosts the same surface in read-only mode with always-on-top support. Repository actions remain owned by `MainWindow`, which refreshes the active section after every mutation.

**Tech Stack:** Rust workspace, GTK4/libadwaita, existing `AutosaveQueue`, `SqliteNoteRepository`, `WindowController`, Xvfb GTK tests, and existing CSS/theme system.

**Spec:** `docs/superpowers/specs/2026-08-19-preview-editor-sticky-window-design.md`

## Global Constraints

- Preserve note IDs, SQLite storage, keyring behavior, themes, localization, keyboard shortcuts, and existing Snap/Flatpak identities.
- Preserve Rich Text spacing at top-bottom `5px` and left-right `8px`.
- Keep `MainWindow` alive after Archive, Restore, Trash, and Delete permanently actions.
- Do not create more than one active Sticky Note Window.
- Do not add a new dependency.
- Keep the existing autosave debounce and repository APIs.

---

### Task 1: Lock the failing behaviors with tests

**Files:**
- Modify: `apps/noor-notes/tests/note_preview_edit.rs`
- Modify: `apps/noor-notes/tests/note_card_archive.rs`
- Modify: `apps/noor-notes/tests/trash_actions.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`
- Create: `apps/noor-notes/tests/preview_editor_surface.rs`
- Create: `apps/noor-notes/tests/sticky_note_window.rs`

**Interfaces:**
- Consumes existing `NotePreview`, `CardAction`, `MainWindow`, and `WindowController` seams.
- Produces executable regression tests that must fail before the refactor and pass after it.

- [ ] **Step 1: Add a failing Preview Body edit persistence test.**

Assert that an edited Preview Body note is sent through the existing `preview_edit_handler`, updates the collection cache, and schedules an `AutosaveQueue` draft before another note is selected.

- [ ] **Step 2: Add a failing archive lifecycle test.**

Exercise the `CardAction::Archive` path and assert that the repository changes the note state while the main window remains present and the Archived section exposes a restore action.

- [ ] **Step 3: Add a failing sticky-window lifecycle test.**

Assert that enabling read-only requests one sticky window, enabling it again does not create a second window, and disabling it closes the sticky reference.

- [ ] **Step 4: Run focused tests and verify they fail for the missing behavior.**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test preview_editor_surface --test sticky_note_window --test note_card_archive --test trash_actions
```

Expected: failures should identify the absent shared surface/sticky lifecycle and the archived restore control, not unrelated compilation errors.

- [ ] **Step 5: Commit the red tests.**

```bash
git add apps/noor-notes/tests
git commit -m "test: define preview editor and sticky window behavior"
```

### Task 2: Extract the shared editor surface

**Files:**
- Create: `apps/noor-notes/src/ui/note_editor_surface.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/ui/editor_header.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`

**Interfaces:**
- Consumes `Note`, `AutosaveQueue`, `SqliteNoteRepository`, `WindowController`, `WritingAssistanceRuntime`, and host callbacks.
- Produces `NoteEditorSurface::new(...)`, `set_note(Note)`, `set_read_only(bool)`, `is_read_only()`, `set_action_handler(...)`, and `widget()` APIs for `NotePreview` and `StickyNoteWindow`.

- [ ] **Step 1: Move document construction into `NoteEditorSurface`.**

Move the title, metadata, rich buffer/editor canvas, editor presentation, status, toolbar, writing-assistance setup, and existing 5px/8px Rich Text spacing into the new surface. Keep all state in the surface or explicit `Rc` handles; do not move repository ownership into it.

- [ ] **Step 2: Define explicit host callbacks.**

Use callbacks for `on_note_changed(Note)`, `on_action(NoteId, CardAction)`, `on_read_only_changed(Note, bool)`, and `on_close_requested()`. The surface must never call `Application::quit()` or close a host window as a side effect of archive/trash.

- [ ] **Step 3: Preserve existing keyboard and save behavior.**

Port the current body/title input handlers, autosave scheduling, view-only suppression, writing-assistance suppression, Escape/double-click transitions, and save-status updates without changing debounce duration or persistence format.

- [ ] **Step 4: Make the standalone implementation use the shared surface.**

Reduce `NoteWindow` to a compatibility wrapper while the new Preview host is wired. Keep standalone behavior compiling during the migration; do not duplicate editor logic.

- [ ] **Step 5: Run editor-focused tests.**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test note_preview_edit --test editor_presentation --test rich_editor
```

- [ ] **Step 6: Commit the extraction.**

```bash
git add apps/noor-notes/src/ui apps/noor-notes/src/note_window.rs
git commit -m "refactor: share note editor surface"
```

### Task 3: Make Preview Body the normal editor

**Files:**
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/app.rs`
- Modify: `apps/noor-notes/src/lib.rs`

**Interfaces:**
- Consumes `NoteEditorSurface` from Task 2.
- Produces a `MainWindow` flow where selecting or creating a note updates Preview Body instead of presenting `NoteWindow`.

- [ ] **Step 1: Replace the current lightweight `NotePreview` body with `NoteEditorSurface`.**

Keep the existing preview container and clamp layout, but mount the shared surface and preserve preview CSS classes. Empty-state text remains visible when no note is selected.

- [ ] **Step 2: Route New Note to Preview Body.**

Change `app.new-note` to create a `Note`, insert it through the existing repository/autosave path, and select it in `MainWindow`. Remove normal `NoteWindow::new(...).present()` from the New Note path.

- [ ] **Step 3: Keep selection and save state synchronized.**

On note selection call `surface.set_note(note)`. On edit callback update `MainWindow.notes`, `NoteCollection::update_note`, and `AutosaveQueue`. Avoid replacing the selected object while a text edit is in progress.

- [ ] **Step 4: Run the Preview Body and create-note tests.**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test preview_editor_surface --test note_preview_edit --test autosave --test library_ui
```

- [ ] **Step 5: Commit the Preview integration.**

```bash
git add apps/noor-notes/src apps/noor-notes/tests
git commit -m "feat: make preview body the primary note editor"
```

### Task 4: Add the read-only Sticky Note Window

**Files:**
- Create: `apps/noor-notes/src/sticky_note_window.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Modify: `apps/noor-notes/src/ui/note_editor_surface.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `crates/windowing/src/controller.rs`
- Modify: `crates/windowing/src/gnome.rs`
- Modify: `crates/windowing/src/x11.rs`

**Interfaces:**
- Consumes `NoteEditorSurface` and `WindowController` capability/state APIs.
- Produces `StickyNoteWindow::present(note, app, controller, on_read_only_disabled)` and one-window replacement/close behavior.

- [ ] **Step 1: Add a failing always-on-top capability test.**

Verify the sticky toolbar exposes an accessible Always on top control, reflects the current note preference, and disables the control with a reason when the controller reports unsupported capability.

- [ ] **Step 2: Implement `StickyNoteWindow`.**

Build a compact `adw::ApplicationWindow` with the shared surface in read-only mode. Track a single weak/strong window reference in the host, close the previous sticky window before replacing it, and keep closing it independent from `MainWindow`.

- [ ] **Step 3: Wire read-only transitions.**

When Preview Body enables read-only, persist the preference through the existing autosave path and present/update the sticky window. When disabled, close the sticky window and restore editing controls in Preview Body.

- [ ] **Step 4: Add the always-on-top action.**

Route the toggle through the existing window controller, persist `Note.always_on_top`, and update the control state after the controller result. Preserve GNOME/X11 capability handling and do not force unsupported platforms.

- [ ] **Step 5: Run sticky and windowing tests.**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test sticky_note_window --test view_only_mode
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-windowing
```

- [ ] **Step 6: Commit sticky mode.**

```bash
git add apps/noor-notes/src crates/windowing/src apps/noor-notes/tests
git commit -m "feat: add read-only sticky note window"
```

### Task 5: Fix archive, restore, trash, and close behavior

**Files:**
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/library/state.rs`
- Modify: `apps/noor-notes/src/ui/empty_state.rs`
- Modify: `apps/noor-notes/tests/library_archive_action.rs`
- Modify: `apps/noor-notes/tests/note_card_archive.rs`
- Modify: `apps/noor-notes/tests/trash_actions.rs`

**Interfaces:**
- Consumes existing `CardAction` and repository methods `archive`, `restore`, `trash`, and `delete_permanently`.
- Produces in-place refresh behavior and restore controls for Archived and Trash sections.

- [ ] **Step 1: Add the archived Restore to All Notes card action.**

Extend the `NoteState::Archived` action list with `("Restore to All Notes", CardAction::Restore, false)` before the destructive action. Keep the note color rail and overflow menu structure unchanged.

- [ ] **Step 2: Make action handling refresh instead of closing.**

After `apply_saved_card_action` succeeds, reload notes, rebuild `LibraryState`, update sidebar counts, update `NoteCollection`, and select the next visible note or show `EmptyState`. Remove any `window.close()` or `app.quit()` from archive/trash action callbacks.

- [ ] **Step 3: Keep destructive confirmation scoped.**

Only `Delete permanently` continues through confirmation. Archive, Restore, and Move to Trash must not close `MainWindow` or the sticky window unless the selected note is no longer available to that surface.

- [ ] **Step 4: Run action regression tests.**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test note_actions --test library_archive_action --test note_card_archive --test trash_actions
```

- [ ] **Step 5: Commit action fixes.**

```bash
git add apps/noor-notes/src apps/noor-notes/tests
git commit -m "fix: keep library open and restore archived notes"
```

### Task 6: Documentation, formatting, and complete verification

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-08-19-preview-editor-sticky-window-design.md` only if implementation decisions require a documented correction
- Test: workspace test suites and existing security/installer checks

- [ ] **Step 1: Update README behavior notes.**

Document that normal editing occurs in the Preview Body, Read-only opens a Sticky Note Window, Archive has Restore to All Notes, and Trash actions do not close the application.

- [ ] **Step 2: Run formatting and static checks.**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets --all-features -- -D warnings
```

- [ ] **Step 3: Run the full GTK and workspace suites.**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test --workspace
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test --workspace --all-features
```

- [ ] **Step 4: Run repository checks.**

```bash
bash tests/security_workflow.sh
bash tests/install_ubuntu.sh
gjs -m extensions/gnome/tests/test-policy.js
desktop-file-validate data/io.github.saamaamr.NoorNotes.Devel.desktop
git diff --check
```

- [ ] **Step 5: Perform manual smoke verification.**

Check create/edit/save, switch notes, Rich Text spacing, Read-only sticky open/close, Always on top, archive/restore, trash/restore/delete, narrow layout, and Light → Dark → Light theme switching. Confirm the main window remains open after every note action.

- [ ] **Step 6: Commit documentation and verification-ready state.**

```bash
git add README.md docs/superpowers
git commit -m "docs: describe preview editor and sticky mode"
```
