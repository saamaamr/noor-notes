# Rich Note Editor and Modern UI Design

## Goal

Redesign Noor Notes as a polished, modern sticky-note editor while adding fully persistent rich-text formatting and preserving existing notes, wording, URLs, window dimensions, window controls, search, import, and encrypted synchronization.

## Compatibility Contract

`Note.content` remains canonical plain text for search, previews, Xpad import, and compatibility with older sync clients. A new optional `rich_content` field stores a versioned structured JSON document. Existing notes with no rich document open exactly as plain text; a rich document is created only when formatting is applied or rich content is edited.

SQLite gains a nullable rich-content column through a forward-only migration. Sync payloads gain an optional rich document and continue accepting payloads without it. Encryption behavior is unchanged: both plain and rich note data are encrypted before upload.

## Rich Document Model

A document contains ordered blocks. Each block contains its paragraph alignment, optional bullet or numbered-list role, and ordered spans. Each span contains text plus bold, italic, underline, strikethrough, font-size, foreground-color, and highlight-color attributes. The JSON starts with an explicit schema version so future clients can migrate it safely.

The GTK editor maps document attributes to `TextTag` objects when loading. Autosave converts the current buffer and tags back to versioned JSON while also deriving plain text. Unsupported or malformed rich JSON falls back to the canonical plain text without losing the original database value.

## Window and Toolbar

The note preserves its saved width and height. The editor uses a warm-yellow surface, a lighter cream header, a subtle border and shadow, and an approximately 18-pixel rounded visual treatment where GNOME client-side decorations permit it. Typography uses the system sans-serif stack with improved padding and line spacing. URLs receive readable dark-blue styling.

The header uses dark symbolic outline icons with consistent 40–44-pixel targets and three groups:

- Left: Always on Top and New Note.
- Center: Formatting and Emoji.
- Right: Archive, Delete, Settings, and standard window controls.

All controls use pale hover surfaces. Delete and Close are dark by default and may become red only on hover. Existing all-workspaces and opacity controls move into Settings so the compact primary toolbar remains usable at sticky-note dimensions.

## Formatting Popover

The formatting button opens a compact popover containing bold, italic, underline, strikethrough, bullet list, numbered list, paragraph alignment, font sizes, text colors, and highlight colors. Formatting applies to the current selection. With no selection, inline styles become typing attributes for newly inserted text. Active formatting states are visibly selected. Ctrl+B, Ctrl+I, and Ctrl+U invoke their matching actions.

## Emoji Popover

The emoji button opens a compact, visually consistent grid of commonly used emoji with a search/filter entry. Selecting an emoji inserts it at the current cursor position, retains the current typing attributes, returns focus to the editor, and triggers autosave.

## Data Flow

Opening a note loads and validates rich JSON when present, otherwise populates the editor from plain text. Editor text or tag changes schedule the existing debounced autosave with both canonical plain text and serialized rich content. Search continues using canonical plain text. Import creates plain notes. Sync merge treats rich content as part of the encrypted note revision and preserves the existing conflict-copy behavior.

## Error Handling

Malformed or unsupported rich JSON never prevents a note from opening. Noor Notes displays canonical plain text, logs an actionable warning, and does not overwrite malformed rich data until the user edits the note. Missing fonts fall back to the desktop sans-serif font. Unsupported compositor-specific window effects degrade to the current rectangular window without affecting editing.

## Verification

- Unit tests cover JSON serialization, malformed input fallback, formatting attributes, list/alignment blocks, and plain-text derivation.
- Storage tests cover migration, nullable rich content, and save/reopen round trips.
- Application tests cover selection formatting, typing attributes, emoji insertion, autosave, and URL tagging.
- Sync tests cover new-to-new and old-to-new optional rich payload compatibility, encryption, and conflict copies.
- GTK smoke tests verify toolbar targets, popover actions, window-size preservation, and no regression in pin, workspaces, opacity, archive, trash, or Xpad import.
- The full workspace test, Clippy, formatting, X11, GNOME policy, and encrypted sync suites must pass before installation and publication.
