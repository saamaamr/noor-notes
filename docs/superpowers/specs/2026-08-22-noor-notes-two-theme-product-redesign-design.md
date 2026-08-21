# Noor Notes Two-Theme Professional Product Redesign

## Goal

Turn the current Noor Notes application into a polished, reliable desktop notes product that combines a calm everyday writing experience, a lightweight sticky-note workflow, and capable Markdown, plain-text, and code editing. The redesign preserves the current data model, encrypted persistence, note lifecycle, autosave, editor commands, keyboard shortcuts, and the primary `MainWindow -> integrated NotePreview` editor path.

The updated current-state references are authoritative:

- `data/screenshots/noor-notes-dev-current-all-features.pdf`
- `data/screenshots/noor-notes-dev-current-all-features.json`
- `data/screenshots/noor-notes-dev-current-all-features.coverage.json`

The reference contains 81 current feature/theme states and explicitly excludes the legacy standalone `NoteWindow` from the integrated editor surface.

## Approved Approach

Use targeted consolidation rather than a rewrite. Existing domain, storage, encryption, repository, autosave, editor adapters, rich-document serialization, note lifecycle, writing assistance, and window-controller APIs remain authoritative. Presentation components and semantic styling may be simplified or decomposed where that directly improves consistency, accessibility, responsiveness, performance, or reliability.

The primary editor remains:

```text
MainWindow -> integrated NotePreview
```

The legacy standalone `NoteWindow` is not restored as the primary editor and is not used to claim integrated functionality. Its internal compatibility code may remain while real consumers are audited, but new product behavior must not depend on it.

## Current Audit Summary

Noor Notes is a Rust 2024 GTK4/Libadwaita desktop application with GtkSourceView, encrypted SQLite persistence through `SqliteNoteRepository`, a Tokio-backed `AutosaveQueue`, a native `RichDocument` model, source editor adapters, writing assistance, and platform window controllers.

The current code already provides:

- Dynamic wide pane allocation around 10% navigation, 20% note collection, and the remaining document width, protected by readability clamps.
- Adaptive medium and narrow layouts with destination visibility and Back navigation.
- Shared editor commands, command capability checks, cursor/selection synchronization, rich formatting, source modes, and autosave.
- Semantic CSS foundations and six historical appearance values.
- Current integrated editing, lifecycle menus, sticky read-only windows, settings, search, sorting, and accessibility tests.

The major remaining product changes are therefore consolidation, not reinvention:

- Reduce the user-facing theme system to Snow and Midnight.
- Safely canonicalize historical appearance preferences.
- Remove duplicated theme-specific CSS and widget choices that no longer serve the product.
- Refine hierarchy, compact controls, document alignment, sticky chrome, and mode-aware editor presentation without duplicating commands.
- Preserve or improve reliability and performance with evidence-backed changes only.

Existing uncommitted source/test edits and existing screenshot deletions in the main checkout are user-owned state and must not be overwritten or accidentally committed.

## Non-Negotiable Data and Behavior Boundaries

- Do not change note IDs, database schema, encrypted payload format, keyring behavior, storage paths, or recovery behavior.
- Do not rewrite or silently convert existing notes.
- Do not bypass `AutosaveQueue`, `SqliteNoteRepository`, or existing lifecycle services.
- Preserve `RichDocument` persistence and all four modes: Rich Text, Markdown, Plain Text, and Code.
- Preserve undo/redo, formatting, lists, colors, highlights, font sizes, clear formatting, emoji, syntax highlighting, line numbers, current-line highlighting, and auto-indent where currently supported.
- Preserve All Notes, Pinned, Favorites, Tags, Archived, Trash, Recent, search, sorting, import, shortcuts, writing assistance, archive/trash/restore/delete, read-only sticky, and Always on Top.
- Do not expose legacy-only File/View/Tools/More/Find actions as integrated functionality unless a real integrated command is implemented and tested.
- Every visible interactive control must call a real command or open a functional surface.
- Do not add a new icon library, UI framework, editor engine, or persistence mechanism.
- Do not modify production code without a failing test that proves the intended behavior.

## Information Architecture

```text
MainWindow coordinator
├── AppHeader
├── Adaptive workspace
│   ├── LibrarySidebar
│   ├── NoteCollection
│   │   └── NoteCard
│   └── NotePreview
│       ├── EditorHeader
│       ├── Mode-aware EditorMenuBar
│       ├── Mode-aware EditorToolbar
│       ├── Reading or editing canvas
│       └── Save/status feedback
└── Library status bar

StickyNoteWindow
├── Compact native title bar
├── Always-on-top toggle
└── Read-only document body
```

`MainWindow` remains the coordinator for repository access, projected library state, selected note, autosave, lifecycle refresh, and sticky ownership. Presentation components communicate through typed callbacks and existing command paths; they do not write directly to storage.

## Two-Theme Appearance Model

The user-facing theme system contains only two intentional themes:

### Snow

Snow is the definitive day theme:

- App background: `#F6F7F9`
- Sidebar: `#F4F6F8`
- Notes list: `#F8F9FB`
- Editor and surfaces: `#FFFFFF`
- Hover: `#F1F3F5`
- Primary text: `#1F2937`
- Secondary text: `#475467`
- Muted text: `#667085`
- Border: `#E4E7EC`
- Subtle border: `#EEF0F2`
- Accent: `#4F6FE8`
- Accent hover: `#425FCC`
- Accent soft: `#EEF2FF`
- Focus ring: accent at approximately 24% opacity
- Success: `#16A34A`
- Warning: `#D97706`
- Danger: `#DC2626`

### Midnight

Midnight is the definitive night theme and avoids pure-black application surfaces:

- App/editor background: `#0F1724`
- Sidebar: `#111A2A`
- Notes list: `#121C2D`
- Surface: `#172235`
- Hover: `#1D2A40`
- Primary text: `#F1F5F9`
- Secondary text: `#CBD5E1`
- Muted text: `#94A3B8`
- Border: `#26364D`
- Accent: `#6D8BFF`
- Accent soft: `#1D2A4A`
- Focus ring: accent at approximately 28% opacity
- Danger: `#F87171`

### Compatibility Migration

Historical values remain deserializable and are canonicalized without breaking startup:

```text
light       -> Snow
warm-paper  -> Snow
cool-mist   -> Snow
graphite    -> Midnight
midnight    -> Midnight
oled        -> Midnight
system      -> Snow or Midnight from the current system appearance
```

The migration does not rewrite note data. Preference loading accepts old values, resolves them safely, and persists only the new canonical representation after an intentional appearance change or the existing safe preference-save boundary. Invalid preference files continue to fail closed without being destroyed.

Appearance settings expose two polished choices only:

```text
Snow
Clean daytime theme

Midnight
Comfortable dark theme
```

The compact header appearance action toggles Snow/Midnight and has an accurate icon, accessible name, tooltip, active state, and persistence. Historical `system` behavior remains compatible at load time but is not a third user-facing theme card.

## Semantic Design System

The stylesheet uses one shared component language with semantic roles for app, sidebar, notes list, editor, surface, elevated surface, input, text states, borders, accent states, feedback states, selection, focus, and disabled controls. Snow defines the default values; Midnight overrides the same roles.

The implementation uses a GTK-compatible Atomic CSS architecture. Small reusable utility classes own single, stable presentation responsibilities such as padding/margin, control height, radius, typography, surface, muted text, icon-button geometry, and focus treatment. Representative utilities include `.nn-p-8`, `.nn-p-12`, `.nn-m-4`, `.nn-h-32`, `.nn-h-36`, `.nn-radius-6`, `.nn-radius-8`, `.nn-text-body`, `.nn-text-meta`, `.nn-text-muted`, `.nn-surface`, `.nn-icon-button`, and `.nn-focus-ring`. Inter-widget spacing remains an explicit GTK layout property such as `Box::set_spacing(8)` because GTK CSS does not provide a portable web-style `gap` utility.

Atomic utilities do not replace semantic state or structural component selectors. Complex behavior such as selected note contrast, destructive actions, sticky pin state, mode-aware editor chrome, list-row state, and Midnight overrides remains in small focused component/state classes. Widgets attach the required utilities explicitly in Rust so repeated padding, radius, height, and typography declarations are not copied across component selectors.

This is not a web Tailwind integration. No CSS generator, JavaScript pipeline, runtime class composer, new dependency, or unbounded utility matrix is introduced. The utility set is curated from the approved compact scales, and tests enforce required utilities, valid GTK parsing, absence of obsolete theme layers, and a compact stylesheet line budget.

The compact scale is:

- Spacing: 4, 8, 12, 16, 20, 24, 32, 40, 48.
- Radius: 4, 6, 8, 10, 12.
- Control heights: 28, 32, 36, 40.
- Icons: existing GTK symbolic icons at 16, 18, or 20 pixels.
- Motion: 120-180 milliseconds for hover/focus/selection only, disabled by reduced-motion preference.
- Shadows: restrained and limited to raised transient surfaces and dialogs.

Shared component selectors must not contain theme-specific literal colors. Historical theme selectors and redundant theme swatches are removed after migration tests prove compatibility.

Typography uses native UI fonts and the existing source-editor monospace configuration:

- App/section title: 18-22 pixels according to context.
- Note card title: 15-16 pixels.
- Reading and editor body: 16 pixels with comfortable approximately 1.5-1.6 line height.
- Metadata: 13 pixels.
- Caption/status: 12 pixels.

The 16-pixel body size is intentional for desktop readability; 12 pixels is reserved for secondary information.

## Adaptive Application Shell

The existing ratio-driven allocation remains authoritative and dynamic:

- Wide target: sidebar about 10%, notes list about 18-20%, editor receives the remainder.
- Sidebar safety range: approximately 160-220 pixels.
- Notes-list safety range: approximately 280-360 pixels.
- Medium: sidebar collapses or hides while notes and editor remain useful.
- Narrow list: notes destination fills the window.
- Narrow document: document fills the window with explicit Back navigation.

The limits are readability guards, not fixed pane widths. Pane recalculation follows the actual window allocation. The editor always receives the majority and navigation/card widths do not grow indefinitely on large displays.

The AppHeader preserves New Note, product identity, menu, appearance toggle, search, sort, and native window controls. Frequent secondary actions use symbolic icons; the New Note action remains discoverable. Every icon-only control has a tooltip, accessible label, hover, focus, and active state.

## Library and Note Collection

All Notes, Pinned, Favorites, Tags, Archived, Trash, and Recent remain. Rows use aligned 18-pixel symbolic icons, label, muted count, neutral hover, visible focus, and a selected state with soft accent surface plus a non-color indicator.

Note cards retain title, a short preview, metadata, tags, pin/favorite indicators, overflow, and note color. Note color remains a four-pixel identity rail with at most a subtle tint. Selected cards use accent-soft background and restrained border while preserving explicit readable title, preview, metadata, and action colors in Snow and Midnight.

Lifecycle-aware actions remain:

- Active: Archive; separated Move to Trash.
- Archived: Restore to All Notes; separated Move to Trash.
- Trashed: Restore; separated Permanently Delete with confirmation.

Every section and search state has a useful empty-state title and one-sentence guidance. Search continues to cover title, body, and tags. Sorting remains Recently updated, Recently created, Title A-Z, and Title Z-A.

## Reading and Integrated Editing

Reading preview displays title, metadata, tags, body, Edit, and the real read-only sticky action. Editing controls are absent until edit mode. Long URLs, API keys, identifiers, and unbroken strings wrap without escaping the document pane; source modes retain their appropriate scrolling/wrapping behavior.

Edit mode stays in the same `NotePreview` surface. Title becomes editable, Done becomes the compact completion action, and mode-aware editor chrome appears without changing the document alignment grid. Title, metadata, editor menu, toolbar, and body share one content container.

The editor body remains a writing surface rather than a form field. It uses comfortable dynamic padding and a readable line length while allowing source modes to use the width they need. Existing Rich Text internal margins and document serialization are preserved.

## Editor Modes and Commands

The four existing modes remain explicit and safely switchable:

- Rich Text: undo, redo, font size, bold, italic, underline, strikethrough, advanced formatting, bullet list, numbered list, and emoji.
- Markdown: only source-relevant real controls and existing syntax behavior.
- Plain Text: a minimal source-editing surface without Rich Text controls.
- Code: existing syntax highlighting, line numbers, current-line highlight, auto-indent, and only additional controls already backed by the source adapter.

Toolbar, menu proxy, and shortcut actions consume the same `EditorCommand` execution path. Availability and active state come from the current mode, adapter capabilities, read-only state, history, and cursor/selection. Controls that cannot execute are omitted or correctly disabled; placeholder actions are prohibited.

The advanced formatting popover preserves typography/font size, underline, strikethrough, alignment, text color, highlight, list actions, and clear formatting. Selection preservation, cursor placement, undo history, autosave, and persisted RichDocument output remain part of every mutation contract.

## Sticky Window

The sticky window remains read-only and lightweight:

- Native compact title bar with one title.
- Body content only; no duplicated title or metadata inside the body.
- Always-on-top represented by the existing symbolic pin control with tooltip, accessible checked state, and clear active appearance.
- Unsupported platform capability is disabled with explanatory feedback.
- Closing the sticky window updates the integrated read-only action without closing the main application or leaving stale state.

Window-controller operations remain asynchronous. Latest-intent/generation protection ensures rapid Always-on-Top toggles cannot persist an older completion over the current UI state.

## Settings, Menus, and Dialogs

Application menu retains Import notes, Keyboard Shortcuts, Appearance, Appearance Settings, Writing Assistance, and Quit. Appearance has only Snow and Midnight choices. Sorting, lifecycle popovers, editor menus, formatting surfaces, emoji, settings, and confirmations share one spacing, focus, selected, destructive, and elevation language.

Dialogs preserve existing behavior and data paths. Destructive confirmation uses explicit danger styling and irreversible wording. Settings rows use consistent title, subtitle, suffix control, validation, and keyboard navigation.

## Performance and Reliability

Performance changes must be evidence-backed and preserve behavior:

- Database and network operations remain asynchronous and outside blocking GTK callbacks.
- Autosave remains debounced, per-note serialized, flushable, and retryable.
- Selection changes and buffer updates must not form re-entrant save or formatting loops.
- Search and library refresh keep generation guards so stale results cannot replace newer intent.
- Always-on-top persistence uses latest-intent protection.
- Widget construction is reused where the current architecture allows; expensive menus or parsers are not recreated per keystroke.
- Rich snapshots and source parsing are not repeated when no relevant state changed.
- CSS is reduced by removing five obsolete user-facing theme layers and duplicate selectors after test coverage is updated.
- Dependencies are removed only when `cargo tree` and source usage prove they are unused; no speculative dependency churn is allowed.
- Release-size and startup checks use the repository's existing scripts and release profile.

Errors do not crash the application, silently discard edits, or falsely report success. Save failures remain visible and retryable. Invalid appearance preferences fail closed. Lifecycle failures leave the note recoverable and refresh the authoritative repository state.

## Accessibility

- Every icon-only control has an accessible name and tooltip.
- Focus order is logical across header, navigation, cards, editor, popovers, settings, and sticky windows.
- Focus rings are visible in Snow and Midnight.
- Selection, active, checked, destructive, and error states are not communicated by color alone.
- Text and icon contrast remain readable in both themes.
- Menus/popovers/dialogs close with Escape where GTK primitives support it and return logical focus.
- Read-only and Always-on-Top expose semantic state.
- Reduced-motion settings disable nonessential transitions.
- Wide, medium, narrow, large-text, and high-scale layouts do not clip essential controls.

## Testing Strategy

All production behavior changes use Red-Green-Refactor. Required automated coverage includes:

1. Historical appearance values load and resolve to Snow/Midnight without destroying malformed files.
2. Appearance settings and header control expose only Snow and Midnight.
3. Snow and Midnight define every semantic surface/text/action/focus role.
4. No obsolete theme selector remains in the final consolidated component CSS.
5. Dynamic wide/medium/narrow allocation keeps the editor dominant and all destinations reachable.
6. Selected notes are readable in both themes.
7. Integrated `NotePreview` preserves saved editor modes and hides unsupported controls.
8. Rich formatting, active state, history, emoji, lists, clear formatting, autosave, and reopen persistence continue to pass.
9. Read-only blocks mutations from pointer and keyboard paths.
10. Sticky latest-intent, close synchronization, and lifecycle behavior remain correct.
11. Search, sorting, archive, trash, restore, permanent delete, import, shortcuts, writing assistance, and settings regressions remain green.
12. GTK presentation, accessibility, and adaptive-layout tests run under Xvfb.
13. Full workspace formatting, clippy, tests, security/release checks, and `git diff --check` pass before integration.

Manual verification uses only the rebuilt current Noor Notes Dev binary and synthetic data. It covers Snow and Midnight, wide/medium/narrow layouts, every library section, search/sort, reading/editing, all four editor modes, Rich Text formatting, lifecycle actions, sticky Always-on-Top, settings, keyboard focus, and restart persistence.

## Delivery and Git Safety

Implementation occurs in an isolated worktree/feature branch created from the current verified `main` commit. The existing dirty main checkout is not cleaned, reset, or overwritten. Only intentional files are staged.

After each TDD task, focused tests pass. Before integration:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
GTK-focused tests under Xvfb
repository release/security checks relevant to the changed surface
git diff --check
```

After verified integration, the feature branch is merged into local `main` without destructive reset, pushed to `origin/main`, and remote/local commit identity is confirmed. Noor Notes Dev is rebuilt and installed through the existing local development installer. Startup is verified without opening or altering the user's personal note database.

## Success Criteria

- The current 81-state functionality baseline remains real and reachable, except historical theme choices intentionally consolidate to Snow and Midnight.
- Snow and Midnight are the only user-facing appearance options and old saved values remain safe.
- `MainWindow -> integrated NotePreview` is the primary editor; no legacy screenshot or control is presented as integrated behavior.
- The wide application feels proportionally 10/18-20/remainder and remains usable at medium and narrow widths.
- Reading is calm, editing is discoverable, and technical modes expose only real capabilities.
- Every visible action works, has correct state, and preserves persistence/history/read-only rules.
- Sticky notes remain lightweight and Always-on-Top persists only the latest intent.
- CSS and component styling are materially simpler without sacrificing theme safety.
- Full verification passes, the result is pushed to GitHub `main`, and the rebuilt Noor Notes Dev starts successfully.
