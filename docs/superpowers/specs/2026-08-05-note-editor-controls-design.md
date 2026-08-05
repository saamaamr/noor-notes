# Note Editor Controls Design

## Goal

Complete Noor Notes' note identity and rich-text controls by adding durable note titles, fully functional bullet and numbered lists, unrestricted positive custom font sizes, and a substantially more compact editor chrome.

## Note titles

Each note will have a real `title` field independent of its body. The title is part of the domain model, SQLite persistence, Xpad import result, encrypted payload, conflict copies, and synchronization data so it remains consistent across restarts and devices.

The note window will show an editable single-line title field above the body. It autosaves through the existing debounced save flow. A Rename action in the note menu opens a focused rename dialog; both controls edit the same stored value. Blank or whitespace-only titles normalize to `Untitled note` for display.

Existing database rows will migrate without data loss. When an older note has no stored title, migration derives it from the first non-empty plain-text body line, trimmed to a reasonable display length; notes with no usable line receive `Untitled note`. The body remains unchanged.

## Lists

Bullet and numbered list controls will operate on complete selected lines, or the current line when there is no selection.

- Activating Bullet adds exactly one `• ` marker to each target line.
- Activating Numbered converts targets to sequential `1. `, `2. `, and so on.
- Clicking the currently active list control removes its markers.
- Switching list type replaces the existing markers instead of stacking them.
- Repeated clicks never create duplicate markers.
- Pressing Enter at the end of a non-empty list item creates the next item and increments ordered numbering.
- Pressing Enter on an empty list item removes its marker and exits the list.
- Applying a list to a multi-line selection transforms each line once and preserves its text.

The implementation will centralize marker parsing and line transformations in the rich-buffer layer. The toolbar will reflect the list type at the cursor without recursive signal handling.

## Font size

The formatting popover will retain common presets and add a custom whole-number entry. Any positive whole-number pixel value is accepted; zero, negative values, decimals, empty text, and non-numeric text are rejected without changing the document. Applying a size affects the selection or future typed text through the existing rich-text mark mechanism. Existing preset values remain one-click choices.

## Compact editor chrome

The note editor will use these exact visual dimensions:

- Top bar and toolbar clickable height: 28 px.
- Toolbar icons: 12 px.
- Note window corner radius: 3 px.

Spacing and padding will be reduced proportionally while keeping focus indication, tooltips, keyboard navigation, and button state visible. The main library window is unchanged unless it shares a CSS rule that must be scoped to prevent accidental alteration.

## Compatibility and data safety

Database migration is additive and idempotent. Old note JSON and encrypted payloads without a title remain readable through a defaulted field. New payloads include the title. Permanent delete, archive, restore, import, search, conflict resolution, and autosave preserve the title.

No Store sandbox permissions change. The Snap and Flatpak manifests remain unaffected.

## Verification

Automated tests will cover:

- Old-row and old-JSON title migration.
- Title autosave, Rename action, blank-title normalization, persistence, import, and sync round trips.
- List add/remove/convert behavior, multi-line selection, repeated-click idempotence, Enter continuation, ordered incrementing, and empty-item exit.
- Preset and arbitrary positive font sizes plus invalid custom values.
- Exact compact CSS dimensions and scoping.
- Existing toolbar, rich editor, storage, import, synchronization, and workspace regression suites.

An Xvfb-backed GTK test will verify the relevant editor interactions without requiring a visible desktop session.
