# Noor Notes Complete UI and Dual-Mode Editor Redesign

## Goal

Replace every visible Noor Notes surface with a premium native GTK4/libadwaita interface whose main library and editor are structurally and visually distinct from the rejected prototype, while preserving encrypted storage, existing notes, autosave, and safe lifecycle behavior.

## Product boundaries

- Native GTK4/libadwaita only; no Electron, web view, telemetry, remote assets, or cloud dependency.
- Existing application ID, encrypted database, note IDs, rich documents, and lifecycle states remain compatible.
- No Snap build, upload, release, or store action belongs to this redesign.
- The UI is replaced rather than restyled. Existing `main_window`, `note_window`, toolbar, and CSS are decomposition sources, not layout templates.
- GtkSourceView is introduced only for Markdown/plain-text/code notes. Rich notes continue using GTK TextView.

## Information architecture

### Main library

The main window uses `AdwNavigationSplitView`/`AdwOverlaySplitView` patterns with three adaptive regions:

1. A compact navigation sidebar for All Notes, Recent, Pinned, Favorites, Tags, Archived, and Trash, with icons, counts, keyboard selection, and a restrained local-only footer.
2. A virtualized notes collection displaying document-like cards with an 18 px title, two-line preview, modified time, two tags maximum, color strip, pin/favorite indicators, focus state, and contextual actions.
3. A document preview with reading typography and metadata. It collapses on narrow windows and becomes a navigation destination instead of leaving unused space.

The header contains only New Note, search, sort, application menu, title, and native controls. Search expands inline, is debounced, reports result count, and produces a dedicated empty state.

### Editor

The editor has five explicit layers:

1. Native compact header with full editable title, save state, library pin, paper color, overflow, and window controls.
2. A 36–40 px command strip containing Undo, Redo, Find, Bold, Italic, Checklist, Insert/Emoji, and More. Unsupported actions are hidden or disabled with explanation.
3. A subtle metadata row for tag chips and editor mode.
4. A centered writing canvas that occupies the window, uses neutral paper by default, and adapts margins to width.
5. A 28–32 px status bar showing save state, line, column, selection, words, characters, UTF-8, mode/language, and zoom.

## Dual-mode editor architecture

Each note has an `EditorMode` metadata field with serde defaults:

- `Rich`: existing structured `RichDocument` rendered by GTK TextView.
- `Markdown`: UTF-8 Markdown rendered by GtkSourceView.
- `PlainText`: GtkSourceView without syntax highlighting.
- `Code(language_id)`: GtkSourceView with a validated GtkSourceView language identifier.

Legacy notes default to `Rich`; no SQL migration is required because the encrypted payload JSON remains authoritative. Mode changes are explicit. Rich-to-Markdown conversion shows a preview, warns about unsupported colors/alignment, creates a recovery snapshot, and commits only after confirmation. Plain text always remains recoverable.

Shared `EditorSession` state owns title, tags, buffer mode, dirty generation, save status, cursor/scroll positions, wrap, zoom, find options, and statistics. Widgets do not call the database directly. `AutosaveController` snapshots the active editor adapter after debounce and flushes safely on close.

`RichEditorAdapter` and `SourceEditorAdapter` implement a common interface for text, selection, undo/redo, find/replace, cursor location, wrap, zoom, snapshot, and restoration. Mode-specific commands are advertised through capabilities so the toolbar never presents fake behavior.

## Productivity behavior

The shared inline search panel supports next, previous, replace, replace all, match case, whole word, result count, and Escape. GtkSourceView mode additionally supports safe regex search and visible match highlighting. Go To Line, line numbers, current-line highlighting, bookmarks, language selection, recent notes/files, encoding display, and syntax highlighting are Source mode capabilities.

Multiple-document support uses tabs only in the full editor window, never in floating sticky-note windows. Closing a dirty tab flushes autosave; failures keep the tab open and show a recoverable error. Session restoration records note IDs, active tab, cursor, and scroll state without duplicating note content.

## Visual system

A new stylesheet replaces `style.css` and `modern.css` with semantic tokens and component classes. Light and dark palettes define background, surface, elevated surface, border, primary/secondary text, accent, success, warning, error, selection, hover, focus, and disabled colors.

Spacing uses 4/8/12/16/24/32/48 px. Cards use 12 px radius; compact controls use 6–8 px. Shadows are restricted to transient elevated surfaces. Typography uses 28 px application/empty-state display, 20 px section headings, 18 px note titles, 16 px editor/body, 13 px metadata, and 12 px captions. Native symbolic icons are 16/20/24 px with accessible labels and tooltips.

Paper colors are Warm White, Cream, Light Yellow, Light Blue, Light Green, Light Pink, Light Purple, and Dark Slate. Each maps to a tested foreground/selection palette. Color never communicates state alone.

Animations use libadwaita transitions and CSS state transitions under 180 ms and respect reduced-motion settings.

## Component boundaries

- `ui/design_system.rs`: semantic classes, density, theme, and paper palette.
- `ui/library_window.rs`: adaptive window coordinator only.
- `ui/library_sidebar.rs`: sections, counts, tags, and keyboard navigation.
- `ui/note_collection.rs`: virtualized model, selection, card factory, and context actions.
- `ui/note_preview.rs`: read-only document preview.
- `ui/editor_window.rs`: editor shell and tab/session coordinator.
- `ui/editor_header.rs`, `editor_toolbar.rs`, `editor_status_bar.rs`, `find_panel.rs`: focused reusable UI components.
- `editor/session.rs`: mode-independent state and save transitions.
- `editor/adapter.rs`, `rich_adapter.rs`, `source_adapter.rs`: editor capability boundary.
- `editor/search.rs`, `commands.rs`, `statistics.rs`: testable behavior without windows.
- `services/session_store.rs`, `recent_items.rs`: local-only session metadata.

## Performance

The library uses `gio::ListStore`, `gtk::FilterListModel`, `gtk::SortListModel`, and `gtk::ListView`/`GridView` factories so thousands of notes do not allocate one permanent widget each. Search is debounced and stale generations are discarded. Preview loading is selection-driven. Long-note autosaves snapshot after debounce, never on each keypress. Signal connections are owned and released with their component.

## Accessibility

All icon-only actions receive accessible labels and tooltips. Navigation, cards, preview, editor, search, popovers, tabs, and dialogs have logical focus order and keyboard activation. Focus rings remain visible in both palettes. Text scaling and 200% interface scaling are tested. Status uses text and icons, not color alone. Destructive actions require confirmation.

## Verification

Tests cover legacy payload defaults, editor mode round trips, conversion/recovery, adapters, undo/redo depth, find/replace/regex, statistics including Unicode, autosave transitions/failure, session restoration, list filtering/sorting, archive/trash/pin/favorite, accessibility labels, and adaptive breakpoints.

Required commands are `cargo fmt --all -- --check`, `cargo test --workspace`, strict workspace Clippy, and `cargo build --release`. Manual verification uses a disposable encrypted profile with at least five notes and newly captures Main, Sidebar, Preview, Editor, Find, Light, Dark, Narrow, Trash, and Pinned states. The real user database is not modified during visual tests.

## Delivery phases

1. Foundation: domain metadata, component boundaries, semantic design system, test fixtures.
2. Library replacement: adaptive navigation, virtualized cards, preview, search/sort/context actions.
3. Rich editor replacement: new shell, writing canvas, formatting, autosave/status, shortcuts.
4. GtkSourceView editor: Markdown/plain/code adapters, syntax, regex, lines, current line, bookmarks.
5. Multi-document/session behavior and recent items.
6. Accessibility, performance measurement, full manual verification, screenshots, installation, commit, and push.
