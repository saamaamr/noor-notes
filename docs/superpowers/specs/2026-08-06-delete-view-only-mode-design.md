# Delete discoverability and per-note View-Only Mode design

## Goal

Make moving notes to Trash immediately discoverable and add a persistent,
minimal, read-only presentation for individual notes without risking existing
content, formatting, or storage compatibility.

## Delete interaction

Active notes expose **Move to Trash** in three places:

1. A visible destructive-hover trash icon in the editor header.
2. The editor More menu.
3. The note-card right-click menu.

Every entry point uses the same application command and confirmation dialog.
The note is flushed before its state changes. A successful operation refreshes
the library and closes an open editor for that note. A failed save or repository
operation leaves the note open and displays an actionable error; it never
silently discards pending text.

Trashed notes do not show Move to Trash. They retain Restore and Permanently
Delete. Permanent deletion remains restricted to Trash and requires explicit
confirmation.

## View-Only Mode

The editor More menu contains **View Only** for active and archived notes. When
enabled, the same note window transitions to a minimal reading presentation.

Visible elements:

- Native header bar and minimize, maximize, and close controls.
- Scrollable note body rendered with its saved rich or source formatting.

Hidden elements:

- Editable title and save-state indicator.
- Pin, favorite, appearance, colour, and trash controls.
- Primary editor toolbar and its popovers.
- Tag entry.
- Find-and-replace panel.
- Editor status bar.

The body remains selectable and copyable but cannot be modified. Double-clicking
the body or pressing Escape returns to Edit Mode. Entering View-Only Mode first
flushes pending edits. If flushing fails, Noor Notes stays in Edit Mode and
shows the save failure.

View-Only Mode does not weaken existing restrictions: trashed notes stay
read-only and use their existing Restore/Permanently Delete presentation.

## Persistence and compatibility

Add `view_only: bool` to `EditorPreferences` with `#[serde(default)]`. Existing
notes deserialize as `false`, so they open in Edit Mode. The value is stored in
the existing encrypted `payload_json`; no SQL migration or application-ID change
is required.

Switching modes updates the note revision and saves through the existing
autosave/repository path. Duplicated notes start in Edit Mode so a reference note
does not unexpectedly create a locked copy.

## Architecture

- `EditorPreferences` owns the persisted per-note preference.
- A small editor-presentation controller owns visibility and editability changes
  instead of scattering mode checks through callbacks.
- All Move to Trash controls dispatch one shared note-window/library command.
- Existing repository transactions and `note_actions::trash` remain the only
  storage transition path.

No new dependency, network access, analytics, or database table is introduced.

## Accessibility and keyboard behavior

- The header trash button has the accessible label and tooltip “Move to Trash”.
- The More-menu View Only row exposes its state and purpose.
- Escape exits View-Only Mode before performing other transient-panel behavior.
- Double-click is an additional convenience, not the sole exit mechanism.
- Focus returns to the note body when entering View-Only Mode and to the title or
  editor body when returning to Edit Mode.

## Verification

Automated tests cover:
