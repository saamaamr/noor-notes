# Noor Notes Preview Editor and Sticky Read-only Window

## Goal

Make the main library preview the primary note editor, keep the current rich-text behavior and autosave guarantees, and reserve a separate window for an explicitly requested read-only sticky view. Archive and trash actions must refresh the library in place without closing Noor Notes, and archived notes must be restorable to All Notes.

## User experience

- Selecting a note loads its title, metadata, rich body, formatting controls, writing assistance, and note actions in the right-side Preview Body.
- New Note creates an editable note in the Preview Body; it does not open a normal editor window.
- The Preview Body has a clear Read-only toggle. Turning it on makes the preview non-editable and opens a separate Sticky Note Window for the selected note.
- The Sticky Note Window is a compact, read-only view with an Always on top toggle. Turning read-only off closes the sticky window and returns the Preview Body to editing mode.
- Archiving a note keeps the main window open, switches or refreshes the active library section, and exposes Restore to All Notes for archived notes.
- Moving a note to Trash keeps the main window open and exposes Restore and Delete permanently actions in Trash.

## Architecture

Introduce a reusable `NoteEditorSurface` that owns the shared note document UI and behavior:

- title and metadata presentation;
- rich-text body/editor and the existing 5px top-bottom / 8px left-right Rich Text spacing;
- formatting toolbar, find/replace, writing assistance, and editor preferences;
- edit/read-only state transitions;
- autosave draft scheduling and save-status updates;
- archive, trash, restore, and permanent-delete action callbacks.

`NotePreview` embeds this surface inside the main library window. The current normal `NoteWindow` creation path is removed. A focused `StickyNoteWindow` wrapper reuses the same surface in read-only mode and adds the always-on-top control. No note storage schema or persistence format changes are required.

The shared surface communicates changes through explicit callbacks supplied by the host (`MainWindow` or `StickyNoteWindow`). It must not own the repository or close the application. This prevents editor actions from accidentally terminating the library window.

## Data and lifecycle flow

1. `MainWindow` selects or creates a `Note` and passes it to `NoteEditorSurface`.
2. Surface edits update the in-memory note and schedule the existing `AutosaveQueue` draft flow.
3. A successful save updates the collection cache and status labels without replacing the selected note or closing any window.
4. Archive/trash/restore/delete actions call the existing repository command path, then refresh the active `LibrarySection` and selection.
5. Read-only enablement sets the surface non-editable, opens or updates one `StickyNoteWindow` for the note, and records the sticky window reference.
6. Read-only disablement closes that sticky window reference and returns the main surface to editable mode.
7. Window close handlers only close the window that owns them; note actions never call application quit or close `MainWindow`.

## Archive and trash behavior

- Active notes show Archive and Move to Trash actions.
- Archived notes show Restore to All Notes and Move to Trash.
- Trashed notes show Restore and Delete permanently.
- After every action, the repository result is applied to the in-memory list and the current section is reprojected. If the selected note leaves the section, select the next visible note or show the existing empty state.
- Confirmation behavior for destructive permanent deletion remains unchanged.

## Sticky window behavior

- The sticky window is created only when the user enables Read-only mode.
- It contains the shared surface in read-only mode, preserves note color and title, and uses the existing window-controller abstraction for always-on-top behavior. If the current platform cannot provide always-on-top, the control is disabled with an explanatory tooltip.
- It has a visible accessible toggle/button for Always on top and a close action.
- Closing the sticky window does not change the main Preview Body edit state; Read-only remains enabled until the user turns it off.
- Opening read-only for another note reuses or replaces the existing sticky window rather than creating an unbounded number of windows.

## Compatibility and non-goals

- Preserve note IDs, local database contents, keyring behavior, themes, localization, keyboard shortcuts, Rich Text spacing, and existing Snap/Flatpak identities.
- Do not redesign unrelated library panes or replace the existing repository/autosave implementation.
- Do not remove the ability to use a standalone sticky view; only the normal editable `NoteWindow` path is replaced.
- The initial implementation does not add multi-window editing or synchronized editing between two editable surfaces.

## Testing and verification

Add or update tests for:

- creating and editing a note entirely from `MainWindow` Preview Body;
- autosave persistence after switching to another note and reopening the app;
- read-only toggle opening/closing exactly one sticky window;
- always-on-top action routing through the window controller;
- archive action keeping `MainWindow` open and making Restore available;
- restore from Archived returning the note to All Notes;
- trash action keeping `MainWindow` open and supporting Restore/Delete permanently;
- section refresh and empty-state selection after actions;
- preservation of Rich Text 5px/8px spacing and existing dark/light theme behavior;
- no duplicate application/window ownership or unexpected application quit.

Run formatter, Clippy, the full workspace test suite (including GTK/Xvfb tests), security checks, desktop validation, and a manual smoke pass for create, edit, read-only sticky, archive, restore, trash, and theme switching.
