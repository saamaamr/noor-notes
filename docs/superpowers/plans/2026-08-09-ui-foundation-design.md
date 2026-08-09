# Noor Notes UI Foundation Design

Date: 2026-08-09
Status: Approved
Scope: Design foundation, library, and editor shell

## Product intent

Noor Notes should present a calm writing surface by default and reveal advanced editing tools only when useful. This milestone reconstructs the visible library and editor composition while preserving the note model, SQLite schema, autosave queue, recovery behavior, rich-document format, and GtkSourceView-based source editors.

The redesign is architectural rather than cosmetic. Existing storage and editor engines remain authoritative, while oversized view controllers and duplicated styling are divided into reusable presentation components.

## Constraints

- Existing notes and settings remain compatible.
- No database migration or rich-document schema change is included.
- No cloud service, analytics, telemetry, or remote assets are added.
- Existing transactional save, close-time flush, recovery-copy, archive, trash, and mode-conversion behavior is retained.
- GtkSourceView remains the source editor for Markdown, plain-text, and code modes.
- Expanded rich-text elements, tabs, and the command palette are deferred.

## Design system

`design-system.css` becomes the single visual foundation for all four themes. Components consume semantic tokens instead of theme-specific colors or scattered dimensions.

Token groups cover application, surface, editor, sidebar, text, borders, accent, semantic status, focus, hover and selection colors; the 4–48px spacing scale; typography; compact/control/card/dialog radii; layout dimensions; and a short transition respecting reduced motion where available.

Theme classes assign semantic token values but do not reimplement components. Conflicting legacy selectors are removed after their call sites are migrated.

## Library shell

The library uses an adaptive three-pane composition: navigation sidebar, notes collection, and selected-note content. Large windows show all three; medium windows collapse the sidebar; small windows show either the collection or selected note with explicit Back navigation. Panes fill available height with subtle separators. Light mode must never expose unexplained black regions.

### Sidebar

Rows display a symbolic icon, label, and secondary count. Selection uses a soft accent surface and narrow accent indicator instead of saturated fill. Keyboard navigation and an icon-only collapsed state with tooltips are supported. Existing filters continue through `LibraryState`.

### Notes collection

Cards contain title, one-to-three-line preview, modified time, up to two tags, relevant pin/favorite/type indicators, restrained color stripe, and consistent context menu. Hover, focus, selection, archived, and trash states share CSS classes. Selection is not communicated using color alone. Destructive actions are not permanently visible.

### Preview and empty states

The preview uses a readable constrained column with document typography. Unsupported or absent content produces an intentional empty state. Every library section has a concise contextual message and relevant primary action.

## Editor shell

The editor is separated into header, compact toolbar, find/replace panel, writing/source canvas, and status bar. `note_window.rs` remains the lifecycle coordinator initially, while widget construction and presentation move into focused modules. Storage calls remain outside view components.

### Header and toolbar

The header presents a prominent borderless title, subtle textual save state, pin/favorite actions, note appearance, overflow menu, and native controls. The title receives expansion priority to avoid unnecessary truncation. Tags are secondary metadata.

The toolbar is a non-wrapping horizontal group exposing Undo, Redo, Find, Style, core emphasis/list actions, supported Link/Code actions, Color, Emoji, and More. Lower-priority actions move into More at constrained widths. Unsupported operations are disabled rather than simulated. Icon controls have accessible labels and shortcut-bearing tooltips.

### Formatting and canvas

Formatting uses labeled groups for typography, formatting, alignment, text color, highlight, and clear formatting. Current states are visible. Only existing persisted rich marks are included in this milestone.

Rich Text uses a neutral centered canvas with comfortable padding and a 760–860px reading width where possible. Optional note colors are restrained paper tints, not application chrome. Markdown, Plain Text, and Code retain GtkSourceView and derive foreground, background, selection, current-line, gutter, and syntax colors from semantic themes.

### Find/replace and status

The existing body-scoped find/replace engine is retained and integrated into the hierarchy. Next, previous, replace, replace all, match case, whole word, result count, keyboard focus, and Escape remain available. Regex remains source-mode-only until a consistent rich-text contract exists.

The status bar displays only relevant save, cursor, selection, word, character, mode, encoding/EOL/indentation, and zoom values.

## Accessibility

Title, tags, body, searches, panes, and icon buttons receive explicit accessible names and descriptions. Focus follows the visible hierarchy and remains visible in every theme. Save and selected states use text or iconography in addition to color. Narrow layouts expose Back navigation instead of off-screen panes.

## Performance and data safety

Filtering and storage continue through existing boundaries. Presentation state never directly saves note content. Autosave stays debounced, and close/archive/trash continue flushing pending edits. Existing payloads are not rewritten merely to display the new UI. Signal ownership is localized, and redundant full-buffer statistics work is reduced where practical.

## Component boundaries

Likely extracted modules:

- `ui/app_shell.rs`
- `ui/editor_header.rs`
- `ui/editor_shell.rs`
- `ui/find_replace_panel.rs`
- `ui/editor_status_bar.rs`
- `ui/formatting_popover.rs`
- `ui/responsive_layout.rs`

Existing modules revised include the library UI modules, `ui/editor_toolbar.rs`, `ui/editor_presentation.rs`, `note_window.rs`, and `resources/design-system.css`. Exact extraction boundaries may adjust after characterization tests, but responsibilities will not collapse back into one oversized module.

## Verification and acceptance

Automated verification includes formatting, Clippy with warnings denied, workspace tests, release build, and targeted tests for responsive presentation, accessible labels, toolbar priority, theme classes, autosave preservation, and existing-note compatibility.

Manual verification uses isolated XDG data across four themes, empty/search/library sections, rich/source modes, formatting, find/replace, narrow/medium/wide windows, view-only, sticky behavior, keyboard-only navigation, and save failure presentation where safely reproducible.

The milestone succeeds when the library and editor have visibly different hierarchy, Light theme has no unexplained dark panels, the toolbar never wraps, the writing surface is calm, advanced actions remain discoverable, existing notes remain readable, and baseline verification remains green.

## Deferred milestones

- Versioned rich-document additions such as headings, checklists, links, blockquotes, and code elements.
- Multiple open-note tabs and session restoration.
- Searchable command palette.
- Full advanced source-editing command set.
- Broader sticky-note and settings redesign beyond foundation fixes.
