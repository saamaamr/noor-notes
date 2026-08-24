# Integrated Editor Mode, Save As, and Responsive Editing Design

## Goal

Keep `MainWindow -> NotePreview` as Noor Notes' integrated editor while making its real formatting entry points reliable, exposing editor-mode conversion directly, exporting the live document to five useful formats, and adapting the editing surface to the available window ratio.

The export contains the title as its document heading followed by the body. Edited metadata and tags remain application data and are not included.

## Integrated editor chrome

The integrated menu row contains only real commands:

```text
Save As | Editor Mode | Format
```

- Save As: DOCX, PDF, HTML, TXT, Markdown.
- Editor Mode: Rich Text, Markdown, Plain Text, Code.
- Format: the existing rich formatting commands, lists, emoji, and the shared advanced-formatting popover.

The active mode uses the same selected styling and accessible selected state as other application menus. Mode changes reuse the existing conversion preview, warning, recovery-copy, autosave, and repository paths.

## Formatting reliability

The toolbar `A` control and `Format -> More formatting` proxy the same existing `FormattingPopover`. GTK could dismiss the original oversized popover when the integrated toolbar had insufficient space above or below it. Its content is therefore placed in a bounded vertical scroller: the popup remains usable at normal and short window heights without duplicating any formatting command.

## Export architecture

`ExportDocument` is an immutable snapshot derived from the live in-memory `Note`/`RichDocument`. Format renderers consume this one representation:

- DOCX: native Office Open XML with Unicode text, rich runs, alignment, and lists.
- PDF: Cairo PDF surface and Pango shaping/pagination.
- HTML: escaped semantic UTF-8 HTML with a compact stylesheet.
- Markdown: readable Markdown mappings and safe degradation of visual-only styles.
- TXT: UTF-8 title/body with readable list markers and no inline styling.

The GTK dialog supplies a sanitized default filename and format filter. The chosen extension is enforced. Rendering runs on Gio's blocking pool, writing uses asynchronous Gio replacement, and local files receive owner-only permissions. Cancellation is silent; failure never mutates the note.

No browser engine, LibreOffice, Pandoc, or shell command is used. `docx-rs` is pinned to the small compatible release without image dependencies; PDF reuses the application's existing Cairo/Pango stack.

## Responsive behavior

Sidebar and collection retain the existing approximately 10%/18% desktop allocation. The editor receives the remainder.

The writing container is calculated from the live editor-pane width rather than one fixed desktop width:

- narrow pane: 100% of available width;
- medium pane: 92%;
- wide pane: 78%.

At smaller widths the toolbar moves secondary commands behind existing menus, Save As/Editor Mode labels shorten without losing tooltips, and at very narrow widths the title/action header stacks vertically. Source and code canvases continue using their appropriate full-width behavior. The standalone legacy editor receives the same ratio-based canvas and compact chrome logic so it cannot regress when opened.

## Compatibility and verification

SQLite schema, note IDs, local content, `RichDocument`, `AutosaveQueue`, lifecycle actions, themes, shortcuts, cursor state, and sticky-note behavior remain unchanged.

Verification covers normalized export data, real DOCX/PDF structure, Unicode, formatting popover activation, menu/mode routing, integrated and standalone responsive widths, strict Clippy, full Xvfb workspace tests, and an optimized release-size comparison.
