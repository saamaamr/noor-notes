# Noor Notes Trash Actions Design

## Goal

Give users complete control over trashed notes through visible row actions, a right-click menu, and trash-specific note-window controls.

## User Experience

- Every row in the Trash tab displays **Restore** and **Permanently Delete** action buttons.
- Right-clicking a Trash row opens a context menu containing the same two actions.
- Opening a trashed note replaces the normal Archive and Move to Trash controls with Restore and Permanently Delete controls.
- Restore moves the note to the active Notes list immediately.
- Permanently Delete always opens a destructive confirmation dialog. Cancel changes nothing.
- Successful actions refresh the Notes, Archived, and Trash lists immediately.
- Restore and permanent-delete controls are only exposed for notes whose state is `Trashed`.

## Persistence

Restore uses the existing revision-aware state transition to `NoteState::Active` and saves before updating the UI.

Permanent Delete physically removes the note and its dependent local records from the Noor Notes SQLite database in one transaction. The implementation removes associated style, geometry, and change-journal records before deleting the note record, without modifying unrelated notes.

Cloud synchronization is currently not configured, so permanent deletion is local. A future cloud implementation must introduce a server-recognized purge operation before promising cross-device permanent deletion.

## Architecture

- `SqliteNoteRepository` gains a narrowly scoped `delete_permanently(NoteId)` operation.
- A reusable trash-action coordinator performs restore/delete operations and reports success or failure.
- `MainWindow` renders row buttons and a context menu that call the same coordinator.
- `NoteWindow` receives repository access so a trashed note can expose the same actions in its toolbar.
- Common confirmation and error copy stays consistent across all entry points.

## Error Handling

- Buttons are disabled while an operation is running to prevent duplicate actions.
- On storage failure, the row/window stays visible, controls are re-enabled, and an error message is shown.
- A note window closes only after a successful Restore or Permanent Delete.

## Testing

- Storage tests prove permanent deletion removes only the selected note and all dependent records.
- Transition tests prove Restore advances revision and timestamp while preserving content.
- UI contract tests prove all three entry points expose both actions only for trashed notes.
- Existing lifecycle, autosave, rich editor, sync, and full workspace tests remain green.
