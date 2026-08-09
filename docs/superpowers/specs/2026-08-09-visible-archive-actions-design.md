# Visible Archive Actions Design

## Goal

Make archiving discoverable without adding clutter. Active notes gain a visible Archive action in both the editor header and the currently selected library card. The existing More-menu Archive action remains available as a fallback.

## Editor

- Add a compact symbolic Archive button to the editor header beside Move to Trash.
- Use the existing `folder-symbolic` icon, the tooltip and accessible label “Archive note,” and the normal non-destructive button appearance.
- Show the button only while the note is active. Archived and trashed notes do not show it.
- Clicking it reuses the existing archive transition: capture pending editor content, persist the Archived state, refresh the library, then close the editor.
- Disable the button while the archive save is in progress to prevent duplicate requests.
- If saving fails, restore the active state, keep the editor open, re-enable the button, and show the existing save-error feedback.
- View-Only mode hides the Archive button with the rest of the editor chrome.

## Library

- Add a compact Archive quick-action button to an active note card.
- Reveal it only when that card is selected; unselected cards remain visually unchanged.
- Do not show it in Archived or Trash views.
- Keep Archive in the card action menu for active notes so keyboard and context-menu workflows remain available.
- Clicking either library Archive action calls the existing transactional repository archive operation, refreshes notes and sidebar counts, and moves selection to the next available note.
- On failure, leave the note active and show the existing library status error.

## Accessibility and Interaction

- Use a native symbolic icon rather than text or emoji.
- Provide the accessible label and tooltip “Archive note.”
- Keep the action keyboard-focusable when visible and absent from the focus order when hidden.
- Do not request confirmation because Archive is reversible and non-destructive.
- Do not change Trash or permanent-deletion confirmation behavior.

## Architecture

- Extend `CardAction` with `Archive`.
- Return a small note-card component handle so `NoteCollection` can bind the selected state to the quick action’s visibility without CSS-only hiding or stale signal handlers.
- Dispatch library archive operations through `MainWindow::handle_card_action` and `SqliteNoteRepository::archive`.
- Add a dedicated editor-header Archive button while retaining the existing More-menu button; connect both to the same archive command path to avoid duplicate persistence logic.
- No database migration, dependency, application-ID, packaging, or Snap change is required.

## Tests

- Prove the new header Archive button is visible for active notes and hidden for archived/trashed notes.
- Prove the selected-card Archive action is present, accessible, and only revealed for an active selected card.
- Prove both editor Archive controls use one save-and-refresh path.
- Prove library `CardAction::Archive` persists the Archived transition.
- Keep existing archive lifecycle, accessibility, Trash, autosave, and toolbar tests passing.

## Non-Goals

- No bulk archive operation.
- No archive confirmation dialog.
- No redesign of note cards or the editor header.
- No change to restore, Trash, permanent deletion, or storage schemas.
