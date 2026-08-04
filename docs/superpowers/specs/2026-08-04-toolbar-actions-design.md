# Noor Notes Toolbar Actions Design

## Goal

Make every visible note-window toolbar control functional, with reliable persistence and clear destructive-action behavior.

## Behavior

- New Note activates the existing `app.new-note` action and opens a fresh note window.
- Archive changes the current note state to `Archived`, saves it immediately, and closes the note window.
- Delete opens a confirmation dialog. Cancel leaves the note unchanged. Confirm changes the state to `Trashed` with the current timestamp, saves immediately, and closes the note window.
- Existing pin, all-workspaces, opacity, rich formatting, list, alignment, color, emoji, and keyboard controls retain their current behavior.
- State changes use the existing repository-backed autosave queue and its immediate `flush` path so closing a window cannot lose Archive or Delete.

## Architecture

Add a small `note_actions` module containing pure state-transition functions for archive and trash. `NoteWindow` owns UI orchestration: it invokes the pure transition, schedules and flushes persistence, then closes only after a successful save. New Note delegates to the existing application action instead of duplicating note construction.

The delete confirmation uses an Adwaita alert dialog with Cancel and Move to Trash responses. The destructive response receives destructive styling and is not the default.

## Failure Handling

If an immediate save fails, the note window remains open. The action button is re-enabled and the user receives an in-window error indication; the application must not imply that the note was archived or deleted.

## Testing

- Unit tests prove archive and trash transitions preserve note content and assign the correct state/timestamp.
- A UI contract test proves New Note, Archive, and Delete have click handlers/action wiring.
- Existing autosave, storage lifecycle, rich editor, and workspace suites must remain green.
- Final verification includes strict Clippy, full workspace tests, installation, installed-binary smoke test, and matching local/remote Git commit hashes.
