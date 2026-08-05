# Noor Notes Quality and Productivity Upgrade Design

**Date:** 2026-08-05
**Status:** Approved design

## Objective

Raise Noor Notes toward a top-tier Linux notes experience through four ordered phases: reliability and data safety, professional UI/UX and accessibility, advanced productivity, and release hardening. Preserve all existing notes and keep unsupported cloud synchronization clearly separated from user-facing features.

## Scope and sequencing

1. Reliability foundation.
2. Professional UI/UX and accessibility.
3. Advanced productivity features.
4. Release hardening, documentation, and installation.

Each phase must remain independently testable and commit-ready. Reliability work may change internal boundaries needed by later phases, but later features must not weaken persistence or recovery guarantees.

## Reliability foundation

- Queue note content, title, rich formatting, style, geometry, pin, workspace, colour, and tag changes through one observable save path.
- Show Saving, Saved, and Save failed states. A failure must be visible and retryable; background save errors must never be silently discarded.
- Flush pending changes before a note window closes. If flushing fails, keep the note window open and preserve its in-memory draft.
- Test opening and migrating legacy databases, including databases without title, colour, tags, or current rich-text fields.
- Keep existing database content and serialized note payloads backward-compatible.
- Cover list behavior for cursor-only edits, multiline selections, repeated toggles, switching kinds, numbered continuation, empty-item exit, and Unicode.
- Replace unsafe Xpad filename assumptions with validated parsing and actionable import errors.
- Keep permanent deletion explicitly confirmed and limited to the selected note.

## Professional UI/UX and accessibility

- Preserve the warm-yellow visual identity, 28 px note toolbar controls, 12 px toolbar icons, and 3 px note corner radius.
- Improve hierarchy, spacing, contrast, hover, checked, disabled, and keyboard-focus states without expanding the toolbar.
- Keep title editing compact and visually distinct.
- Add save-state feedback without interrupting typing.
- Make formatting state follow the cursor and selection, including lists, marks, alignment, colours, and size where determinable.
- Provide polished empty states for active notes, archived notes, Trash, and no-result searches.
- Keep destructive actions visually separated, fully labelled, and confirmed.
- Give controls descriptive accessible labels/tooltips, logical focus order, and keyboard operation.
- Keep note windows functional at narrow sizes by moving secondary actions into compact menus when necessary.

## Advanced productivity

### Undo and redo

- Use the GTK text buffer undo stack when supported, expose toolbar actions, and bind Ctrl+Z and Ctrl+Shift+Z.
- Undo/redo changes flow through normal autosave and save-state handling.

### Note colours

- Provide a curated accessible palette with current warm yellow as default.
- Persist the selected palette identifier rather than arbitrary CSS.
- Apply colour consistently to title, editor surface, and compatible chrome while maintaining readable contrast.

### Tags and sorting

- Store normalized display tags per note, prevent duplicate tags case-insensitively, and retain user-visible spelling.
- Search titles, bodies, and tags.
- Display compact tags in library rows.
- Sort by most recently updated, title, or creation date; persist the local selection.

### Duplicate note

- Duplicate title, content, rich document, and visual style into a new active note with a new identifier, fresh timestamps, default window placement, and no deletion/archive state.

### Find in note

- Ctrl+F opens an in-note search bar.
- Provide next and previous match navigation, current/total match count, case-insensitive matching, and a clear no-results state.
- Closing find restores focus to the editor.

### Export

- Export one note through a native file chooser as UTF-8 plain text or Markdown.
- Markdown export preserves supported inline marks, lists, and links where representable, and safely falls back to readable plain content.
- Export never mutates the note or its save state.

### Shortcuts

- Support new note, application search, rename, archive, close, undo, redo, find, and existing formatting shortcuts.
- Provide an application shortcuts reference window.

## Architecture

- `noor-domain` owns durable note metadata and pure transformations.
- `noor-storage` owns schema migrations, persistence, indexed querying, sorting, duplication transactions, and corruption handling.
- Focused editor modules own rich-buffer commands, list behavior, find state, undo/redo, and export conversion.
- Window modules compose GTK widgets and delegate behavior; persistence and transformations remain testable without a live window where possible.
- Split large UI modules only along save-state, dialog, or editor-action responsibilities required by this work.
- Experimental sync remains isolated. The app must not claim cloud synchronization is available without a configured account and server flow.

## Data model and compatibility

- Add optional/defaulted colour and tags fields to serialized notes so older payloads deserialize safely.
- Add additive SQLite migrations only. New columns/tables use safe defaults and preserve existing rows.
- Search and sort APIs use explicit enums rather than stringly typed SQL fragments.
- Existing yellow notes remain visually unchanged until users choose another colour.

## Error handling

- User-correctable failures show plain-language messages and a Retry action.
- Save failures retain pending drafts.
- Import and export errors include the affected path when safe to display.
- Unsupported desktop window features remain disabled with explanatory tooltips.
- Malformed stored rich text falls back to the durable plain-text body.

## Testing strategy

- Follow red-green-refactor for every behavior change.
- Add pure domain tests for metadata normalization, duplication, sorting inputs, and Markdown conversion.
- Add storage tests for every migration path, tag search, sorting, and transactional duplication.
- Add GTK tests under Xvfb for list edge cases, formatting state, undo/redo, find, compact sizing, accessibility labels, and responsive controls.
- Add autosave tests for visible state transitions, retry, close-time flush, and failure preservation.
- Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, GTK/X11 tests under Xvfb, the GNOME extension contract test, installation tests, and `git diff --check` before completion.

## Documentation and release

- Update README feature, shortcut, data, recovery, export, packaging, and limitation sections.
- Keep version and Store status claims factual.
- Install the verified release build for the current user after integration.
- Preserve unrelated local artifacts, including existing Snap packages.

## Success criteria

- No tested editing path silently loses a pending draft.
- Existing databases open and migrate without losing note content.
- All listed productivity features are discoverable and keyboard-operable.
- The compact note window remains usable at its supported minimum size.
- All automated verification passes on the merged result.
