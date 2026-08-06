# Theme-Matched Source Editor Palettes Design

## Problem

Rich Text uses a normal `GtkTextView` and receives Noor Notes CSS directly. Markdown, Plain Text, and Code use `GtkSourceView`. Their `GtkSourceBuffer` currently keeps GtkSourceView's fixed default `classic` style scheme, whose base is black text on white. That scheme also owns syntax, gutter, current-line, selection, search, and cursor colours, so it conflicts with Noor Notes Light, Graphite, Midnight, OLED, and note-window styling.

## Goals

- Make Markdown, Plain Text, and Code readable in every Noor Notes theme.
- Retain professional syntax highlighting in Markdown and Code.
- Give Plain Text a clean, uniform body colour.
- Match source-editor canvas, gutter, selection, cursor, current line, search, and syntax colours to the active Noor Notes palette.
- Update already-open source editors immediately when the user changes theme.
- Preserve note content, editor mode, language, undo history, and database compatibility.
- Fail safely to a readable built-in scheme if custom resources cannot load.

## Palette System

Add four GtkSourceView style schemes:

| Scheme | Canvas | Body text | Gutter | Current line | Accent |
| --- | --- | --- | --- | --- | --- |
| Noor Light | `#ffffff` | `#202124` | `#f5f5f4` | `#f3f5fb` | `#4d6fdc` |
| Noor Graphite | `#282a2f` | `#f5f6f8` | `#222428` | `#303238` | `#9aafff` |
| Noor Midnight | `#1a2638` | `#f2f7ff` | `#141e2c` | `#203049` | `#73b7ff` |
| Noor OLED | `#121216` | `#fafaff` | `#0a0a0c` | `#19191f` | `#b1a0ff` |

Each scheme defines base text, cursor, gutter text, active line number, current line, selection, search match, bracket match, and semantic syntax groups. Comments use a quieter secondary foreground; keywords and Markdown headings use the palette accent; strings use an accessible green; constants use a warm purple or amber; links use an accessible blue; errors use the semantic error colour. Syntax colours must retain readable contrast against that scheme's canvas.

Plain Text has no source language and therefore shows only the base body colour. Markdown and Code keep language-specific syntax highlighting through the same theme-matched scheme.

## Embedded Resources

Store the schemes under `apps/noor-notes/resources/styles/` and list them in `noor-notes.gresource.xml`. Add a package-local `build.rs` using the standard `glib-build-tools` build-time helper to compile the resource. Register the compiled bytes at application startup, prepend the embedded style directory to `GtkSourceStyleSchemeManager`, and force one rescan.

This adds one small build-only dependency. It adds no runtime service, remote resource, system package installation, or network behavior. Existing system GtkSourceView remains the runtime dependency.

## Runtime Architecture

Create `editor/source_palette.rs` with these responsibilities:

- Map `EffectiveTheme::{Light, Graphite, Midnight, Oled}` to the matching Noor scheme ID.
- Register the embedded GResource exactly once.
- Resolve and apply a scheme to a `sourceview5::Buffer`.
- Fall back to `Adwaita` for Light and `Adwaita-dark` for dark themes if the Noor scheme is unavailable.
- Return the applied scheme ID so tests and diagnostics can verify the result without inspecting rendered pixels.

`SourceEditorAdapter` applies a readable initial scheme when constructed and exposes theme application through the palette module. `NoteWindow` passes the current effective theme when opening Markdown, Plain Text, or Code notes.

For live switching, `NoteWindow` subscribes to `AppearanceManager` only for a source buffer. The callback stores a weak buffer reference, upgrades it when invoked, and applies the new scheme. This prevents closed editor windows from being retained by the appearance listener list.

Rich Text remains unchanged.

## CSS Interaction

GtkSourceView schemes become authoritative for the source editor's inner text, canvas, gutter, current line, selection, cursor, search, and syntax colours. Noor Notes CSS remains authoritative for outer window surfaces, toolbar, title, tags, scroller, and status bar.

The shared `.nn-writing-canvas` class continues to control padding, font size, and zoom. It must not force a competing source-editor foreground or background. Rich Text retains its existing theme-specific canvas CSS through a new `.nn-rich-writing-canvas` class, preventing any source-scheme conflict.

## Failure Behavior

- If GResource registration fails, startup continues.
- If a custom scheme is missing, the buffer receives the appropriate built-in Adwaita scheme.
- If both custom and fallback lookup fail, the adapter reports the failure in tests and leaves GtkSourceView operational; note content is never modified.
- Theme changes never recreate the buffer, so text, selection, undo/redo history, cursor, and scroll position remain intact.

## Testing

Add tests that verify:

- The default `classic` scheme is replaced for every source mode.
- All four Noor scheme IDs are discoverable from the registered resource path.
- Each scheme defines text, background, cursor, line-number, current-line, selection, and search styles.
- Base foreground and background pairs meet at least WCAG AA `4.5:1` contrast.
- Light, Graphite, Midnight, and OLED map to the correct scheme IDs.
- Applying a new theme changes only the buffer scheme and preserves Unicode text, cursor, selection, undo, and redo.
- Plain Text has no language styling while Markdown and Code retain language syntax.
- Missing custom schemes select a readable Adwaita fallback.
- Existing source-editor search, bookmarks, line numbers, conversion, and persistence tests remain green.

Manual verification covers Bangla, Arabic, Latin, Markdown headings and links, and representative code tokens in all four themes.

## Data, Security, and Packaging

There is no database migration or note-format change. The scheme XML is compiled into the application binary and performs no file writes or network access. Snap metadata and Snap Store revisions are not modified. A future Snap build would naturally include the embedded resource through the Rust binary, but this task performs no Snap build, upload, or release action.
