# Noor Notes Professional Product UI Redesign

## Goal

Redesign every visible Noor Notes surface into a calm, cohesive, professional desktop notes product while preserving existing notes, encrypted persistence, autosave, note lifecycle behavior, editor commands, shortcuts, supported editor modes, themes, native window behavior, and current application identity.

The product promise is **Calm Notes + Powerful Editor**. Creating, selecting, reading, and editing an ordinary note must remain simple. Advanced formatting, source editing, search, conversion, and note-management actions remain discoverable through progressive disclosure.

## Approved Approach

Use a controlled presentation-layer component replacement. Visible shell, navigation, card, document workspace, toolbar, menu, popover, settings, dialog, and sticky-window components may be replaced or decomposed. Domain objects, storage, encryption, autosave, repository APIs, command implementations, and note payload formats remain authoritative and are not rebuilt.

The work is delivered in three production phases, but the redesign is not complete until all phases and cross-theme verification finish:

1. Design foundation, application shell, sidebar, notes pane, and document workspace.
2. Editor shell, mode-aware command surfaces, menus, formatting, search, and note actions.
3. Sticky window, settings, dialogs, themes, accessibility, responsive behavior, and final regression verification.

## Current Product Audit

Noor Notes is a native Rust desktop application using GTK4, Libadwaita, GtkSourceView, SQLite-backed encrypted note payloads, and a local autosave queue. The current managed application opens a `MainWindow` with a sidebar, virtualized note collection, and preview-based primary editor. A standalone `NoteWindow` remains available as a legacy/full-editor surface. `StickyNoteWindow` provides the read-only floating view and routes always-on-top through the existing window-controller abstraction.

The current design system lives in `apps/noor-notes/resources/design-system.css`. `AppearanceManager` applies one semantic theme class to each registered window. Supported effective themes are Snow, Warm Paper, Cool Mist, Graphite, Midnight, and OLED. Existing editor commands, `RichBuffer`, source adapters, conversion flow, undo/redo, autosave, note lifecycle services, search/sort state, writing assistance, and shortcuts already provide the functional foundation.

The 33-page `data/screenshots/noor-notes-dev-current-all-features.pdf` was reviewed as the current-state functional baseline. It confirms the library, edit/read modes, menu bar, command toolbar, typography/formatting panel, color/highlight controls, emoji, sorting, global search, note actions, formatting active states, sticky read-only window, always-on-top state, all library sections, More Actions, and editor-mode entry points. The reference also exposes the design problems this work addresses: weak pane hierarchy, cramped command surfaces, excess editor whitespace, inconsistent transient-surface organization, small low-emphasis controls, incomplete alignment between title/toolbars/body, and insufficient visual distinction between reading and editing.

## Non-negotiable Boundaries

- Do not change note IDs, encrypted payload compatibility, database schema, keyring behavior, or storage paths.
- Do not migrate or silently rewrite user notes.
- Do not bypass the existing autosave and repository command paths.
- Do not add unsupported or decorative commands.
- Do not add a second icon family or design framework.
- Do not replace native GTK/Libadwaita window controls.
- Preserve Rich Text internal canvas margins of 8 pixels horizontally and 5 pixels vertically.
- Preserve Rich Text, Markdown, Plain Text, and Code modes and their safe conversion/recovery behavior.
- Preserve Graphite, Midnight, OLED, Warm Paper, and Cool Mist while making Snow the default Light personality.
- Keep unrelated user worktree changes untouched.

## Information Architecture

The main window becomes a composition of focused presentation components under the existing coordinator:

```text
MainWindow coordinator
├── AppHeader
├── AdaptiveWorkspace
│   ├── LibraryNavigation
│   ├── NotesPane
│   │   ├── NotesPaneHeader
│   │   ├── SearchAndSort
│   │   ├── NoteCollection
│   │   └── EmptyState
│   └── DocumentWorkspace
│       ├── DocumentHeader
│       ├── EditorMenuBar
│       ├── EditorCommandBar
│       ├── ReadingOrEditingCanvas
│       └── DocumentStatus
└── AppStatusBar
```

`MainWindow` continues to own repository access, note collection state, section projection, autosave scheduling, sticky-window ownership, and refresh generations. Child presentation components communicate through typed callbacks or existing commands; they never write to the database directly.

The new document components are reusable by the main preview workspace and the standalone editor. Compatibility adapters preserve current public integration points during replacement. The sticky window reuses the read-only document canvas but not the full editing chrome.

## Semantic Design System

The existing stylesheet is consolidated around semantic roles rather than per-screen literal colors. Required roles include:

- Background: app, sidebar, list, editor, surface, raised surface, popover, modal, and input.
- Text: primary, secondary, muted, disabled, and inverse.
- Border: default, subtle, strong, and focus.
- Action: accent, accent hover, accent soft, accent strong, success, warning, danger, danger soft, and info.
- Interaction: hover, active, selected, pressed, disabled, focus ring, and drag target where a real drag interaction exists.
- Typography: UI, editor, and code families; display, section, body, metadata, and caption scales.
- Geometry: spacing, control heights, icon sizes, radii, shadows, transitions, pane ratios, and readable line length.

Snow uses a neutral professional base: app `#F7F8FA`, sidebar `#F5F7F9`, list `#F8FAFC`, editor/surface `#FFFFFF`, primary text `#1F2937`, secondary text `#475467`, muted text `#667085`, border `#E4E7EC`, accent `#4F6FE8`, accent hover `#425FCC`, accent soft `#EEF2FF`, danger `#DC2626`, success `#16A34A`, and warning `#D97706`. Exact GTK color declarations remain semantic and centralized.

Spacing follows 4, 8, 12, 16, 20, 24, 32, 40, and 48 units. Compact icon controls use a 32-pixel interaction box and primary/header controls use 36 pixels. Sidebar rows target 40 pixels. Icon sizes are 16, 18, and 20 pixels. Radius roles are 4, 6, 8, 10, and 12 pixels. Shadows are limited to raised transient surfaces and dialogs. Transitions remain 140–180 milliseconds and respect reduced-motion preferences.

Typography targets a 30-pixel document title, 20-pixel section title, 15–16-pixel note title, 16-pixel editor/body with approximately 1.6 line height, 13-pixel metadata, and 12-pixel captions/status. System UI fonts remain native; code uses the existing monospace/source configuration.

Warm Paper, Cool Mist, Graphite, Midnight, and OLED override semantic values rather than component structure. Shared component rules never embed Snow literals.

## Adaptive Application Shell

Pane allocation is ratio-driven and recalculated from the actual window allocation:

- Wide target: navigation about 10%, notes pane about 20%, document workspace receives the remainder.
- Readability guards prevent labels, counts, card content, and editor controls from collapsing. Navigation has a 160-pixel safety floor and 220-pixel upper guard; the notes pane has a 280-pixel safety floor and 360-pixel upper guard. These are usability limits, not fixed layout widths.
- Medium layouts collapse navigation to an icon-only rail or hide it according to available space, while notes and document remain visible.
- Narrow layouts show either the notes destination or document destination with explicit Back navigation. Three panes are never squeezed into one narrow allocation.
- A safe user-adjusted ratio may be retained for the current session if GTK allocation and tests prove stable. Stored pane geometry must never override a new window's safety guards.

The editor always receives the visual majority. Ultrawide windows do not inflate navigation or note cards indefinitely. The document canvas targets a readable 58–78 characters per line, expands on smaller panes, and uses comfortable dynamic horizontal padding.

## Application Header

The native Libadwaita header remains. Product title and development/private subtitle retain their hierarchy. New Note is the only persistent primary action. Main menu, appearance, sort, search, and native window controls use a shared quiet control language with 36-pixel targets, 18-pixel symbolic icons, neutral hover, soft-accent active state, and visible focus.

Search may expand inline without obscuring the document. Sort communicates the selected order and remains keyboard accessible. Header controls move into safe overflow or collapse by priority at constrained widths; they do not wrap into a second header row.

## Library Navigation

All Notes, Pinned, Favorites, Tags, Archived, Trash, and Recent remain. Each row contains one symbolic icon, label, and right-aligned muted count. Hover is neutral. Selection uses accent-soft background, accent icon/text, and a non-color cue such as an inset indicator. Focus remains independently visible.

Collapsed navigation exposes icons with accurate tooltips and accessible names. The local-only/privacy status remains quiet and does not compete with navigation.

## Notes Pane and Cards

The notes pane includes a compact section header, result count, search/sort state where appropriate, the existing virtualized model, and section-specific empty states. Search matching continues to use the existing searchable fields and debounce/generation rules.

Each note card includes title, maximum two-line preview, edited metadata, optional compact tags, pin/favorite indicators, a four-pixel note-color rail, and a 32-pixel overflow action. Note colors become identity markers with only an extremely subtle tint. Whole-card saturated fills are prohibited.

Card states are normal, hover, selected, focused, archived, and trashed. Dragging styling appears only if a real supported drag interaction exists. Selected cards use accent-soft surface, a restrained accent border, readable dark text in Snow, and explicit dark-theme overrides. Focus is not represented only by color. Secondary actions may appear on hover without causing layout shift.

The action menu is lifecycle-aware:

- Active: Archive, then a separated Move to Trash danger action.
- Archived: Restore to All Notes, then separated Move to Trash.
- Trashed: Restore, then separated Permanently Delete with confirmation.

Empty states provide a meaningful title and one-sentence next step for every library section and search results.

## Document Reading and Editing

Reading mode presents only title, metadata, content, a compact Edit action, and a secondary Open Read-only action. The title is a semantic label in reading mode rather than an always-visible editable entry. Body text is selectable, uses a comfortable line height, and safely wraps long URLs, identifiers, and unbroken tokens.

Edit mode transitions within the same workspace. The title becomes an entry, Done becomes a compact completion control, and editor chrome appears without moving the document onto an unrelated alignment grid. Title, metadata, menu bar, toolbar, and body share one content container and one left edge.

Rich Text retains 8-pixel left/right and 5-pixel top/bottom internal margins. Source modes retain their existing source palette, monospace behavior, word-wrap configuration, line behavior, and mode capabilities.

The main workspace must edit the selected note through the existing session/adapter boundary instead of forcing a note to Rich mode. Rich, Markdown, Plain Text, and Code notes reopen in their saved mode. Safe conversion dialogs and recovery behavior remain authoritative.

## Two-level Editor Chrome

Level one contains File, Edit, View, Insert, Format, and Tools. Level two contains the commands applicable to the current mode. Both align with the document content and use content-fit sizing. The toolbar does not create a large empty bordered rectangle, and it never wraps into an unusable multi-row cluster. Secondary actions move into More at constrained widths.

Rich Text exposes Undo, Redo, supported typography/size, Bold, Italic, Underline, Strikethrough, Formatting, supported lists, Emoji, and More. Markdown, Plain Text, and Code expose only commands actually provided by the current adapters. No Link, block style, indentation, comment, language, or other command appears unless the adapter supplies a real implementation.

The Style control exposes only real typography capabilities. Current Rich content has no block-style model, so Heading, Quote, and Code Block are not invented. If the control only opens supported typography/formatting, its label and accessible description must communicate that truthfully.

## Command Architecture and Control Contracts

Toolbar buttons, menu proxies, keyboard shortcuts, and More entries consume the same underlying `EditorCommand`/adapter operation. A visible control must have a real execute path, capability predicate, enabled state, checked state where applicable, accessible name, tooltip for icon-only controls, and failure behavior.

Undo and Redo reflect real history availability. Formatting active states follow cursor/selection changes. Selection and cursor are preserved centrally while focus moves to a toolbar or popover. Mutations re-enter the existing buffer-change and autosave path. Read-only state disables both pointer and shortcut mutation paths.

Unsupported actions are omitted rather than represented by dead controls or “coming soon” responses. Existing application-level actions continue to use the shared application action system.

## Menus and Transient Surfaces

All menus, popovers, dropdowns, and contextual surfaces share padding, row height, radius, border, elevation, hover, selected, disabled, focus, and destructive patterns. They remain anchored to the triggering control, close with Escape/outside activation, restore logical focus, and do not leave unrelated overlapping popovers open.

Supported menu contents are:

- File: New Note, Duplicate, supported Export choices, and lifecycle-appropriate Delete/Trash action.
- Edit: Undo, Redo, and Find.
- View: Word Wrap, Zoom In, Zoom Out, Reset Zoom, and View Only where supported.
- Insert: Emoji.
- Format: Bold, Italic, Underline, Strikethrough, Bullet List, Numbered List, and More Formatting where supported.
- Tools: Go to Line, Editor Mode, and More Actions where supported.

Shortcut hints align on the trailing edge. Destructive actions are separated and use danger language. Checkable state uses a check/icon and accessible state rather than color alone.

The formatting popover groups Typography, Formatting, Alignment, Text Color, Highlight, Lists, and Clear Formatting. It preserves current font sizes 12, 14, 16, 18, and 24 plus the existing validated custom-size path. Color/highlight options show selection indicators, theme-safe swatches, accessible labels, and explicit automatic/no-highlight choices.

Emoji remains the current compact supported set. Picking an emoji inserts it through the command path at the preserved cursor/selection, closes the popover, returns focus to the editor, and participates in undo/autosave.

Sort preserves Recently updated, Recently created, Title A–Z, and Title Z–A with an explicit active indicator. The application menu groups Import, Keyboard Shortcuts, Appearance, Appearance Settings, Writing Assistance, and Quit. Appearance choices remain real application actions.

## Search, Status, and Feedback

Global search retains title, content, and tag semantics. It includes a clear action, visible focus, keyboard navigation, result count, dedicated no-results state, and section-aware feedback. It never overlays the document unnecessarily.

In-note Find/Replace remains a separate contextual panel and exposes only supported options. Escape closes it and returns focus to the editor.

The library status bar shows section/result information and privacy/save state. The editor status shows real statistics, writing-assistance state, mode/encoding, and zoom without becoming visually dominant. Save failure remains visible and recoverable; no action silently claims success.

## Read-only and Sticky Window

Read-only is an explicit action/state, never plain ambiguous text. Entering it disables editing controls and mutation shortcuts while preserving the selected note and theme. Exiting returns to the prior workspace state.

The sticky window contains native compact window chrome, note title in the top bar, body content only, and a clear Always on Top toggle. It does not duplicate title, metadata, or full editor toolbars inside the body. Note color may appear as a restrained accent. Content padding and typography adapt to the window allocation.

Always on Top displays active state, accessible checked state, and tooltip. Unsupported desktops disable it with an explanatory tooltip. Closing the sticky window updates the main read-only control through the existing lifecycle callback and does not close Noor Notes or leave a stale Exit read-only label.

## Settings, Dialogs, and Secondary Windows

Appearance Settings, Writing Assistance, Keyboard Shortcuts, import flow, rename, Go to Line, mode conversion, export, and permanent-delete confirmation use the same spacing, typography, control, focus, and feedback language. Existing functionality and privacy copy remain unchanged unless wording is clarified without altering meaning.

Dialog buttons follow cancel/secondary/primary order. Destructive confirmation uses explicit danger appearance and irreversible copy. Settings rows use consistent titles, subtitles, suffix controls, validation state, and keyboard navigation. Theme registration applies to every secondary window.

The legacy standalone editor remains functional and consumes shared header, command bar, menus, transient surfaces, canvas, and status components rather than maintaining a divergent visual system.

## Responsive Behavior

- Wide: approximately 10% navigation, 20% notes, remainder document, subject to readability guards.
- Medium: compact/icon navigation or hidden navigation, notes plus document, priority-based header/toolbar overflow.
- Narrow list state: notes destination fills the window.
- Narrow document state: document fills the window with a visible Back action.
- Sticky: body and compact top bar scale without editor chrome.

No breakpoint may leave a 40-pixel preview sliver, clip title/actions, overlap native window controls, or wrap toolbar groups into an unusable layout. Allocation behavior is covered by real GTK widget tests under Xvfb.

## Accessibility

- Every icon-only control has an accessible name and tooltip.
- Every menu, popover, dialog, list, card, search field, and editor surface has a logical tab order.
- Focus rings remain visible across Snow, Warm Paper, Cool Mist, Graphite, Midnight, and OLED.
- Selection, checked state, error, and destructive meaning are not communicated by color alone.
- Text and icon contrast meet practical desktop readability in all states.
- Menu/popover/dialog Escape behavior and focus restoration are verified.
- Selected notes expose semantic selected state; Read-only and Always on Top expose semantic state.
- UI remains usable with large text and 200% interface scaling.
- Reduced-motion preferences disable nonessential transitions.

## Data Flow and Error Handling

```text
UI action
  → shared application/editor command
  → editor adapter or lifecycle service
  → in-memory note/session update
  → existing buffer-change/autosave/repository path
  → collection cache and status update
```

UI components do not directly mutate storage. Archive, Trash, Restore, and Permanent Delete use existing lifecycle services, refresh the current projection, choose the next valid selection or empty state, and keep the main window open.

Command failure never crashes the window or corrupts note state. Recoverable failures remain in the current editor/session and surface through save status, application status, or a focused dialog according to severity. Mode conversion commits only after confirmation and preserves current recovery behavior.

## Testing Strategy

Implementation follows test-first replacement in reviewable vertical slices. Tests cover:

- semantic token completeness and valid GTK CSS;
- theme-safe shared component rules and selected-note contrast;
- dynamic pane ratios, safety guards, user ratio handling, and narrow navigation;
- sidebar counts, collapsed accessibility, focus, and section selection;
- note-card content, color rail, hover/selected/focused/lifecycle states, and compact action menu;
- every section/search empty state;
- reading/edit transitions, shared alignment, title behavior, long-string wrapping, and Rich Text 8/5 margins;
- command capability, enabled/active synchronization, selection preservation, undo/redo, lists, emoji, colors, highlight, and persistence;
- mode-aware toolbar/menu visibility across Rich, Markdown, Plain Text, and Code;
- menu/popover mutual exclusion, Escape behavior, focus restoration, and destructive grouping;
- read-only shortcut blocking, sticky close synchronization, and always-on-top routing;
- settings/dialog theme registration, accessible names, focus order, and destructive confirmations;
- autosave and lifecycle regressions without changing the real user database.

Verification runs formatter, workspace check, strict Clippy, focused non-GTK tests, GTK tests under the supported Xvfb setup, full workspace tests, locked release build, and `git diff --check`. Manual verification uses a disposable encrypted profile and captures Snow plus representative Warm Paper, Cool Mist, Graphite, Midnight, and OLED states at wide, medium, narrow, and sticky sizes. Light-to-dark-to-light switching, search, sort, archive, restore, trash, editor modes, formatting, long title/content, Read-only, and Always on Top are inspected.

## Acceptance Criteria

The redesign is complete when every current PDF feature remains real and reachable; the shell gives the editor dominant space; navigation and cards are compact and readable; note colors are restrained identity rails; reading and editing form one coherent workspace; every visible command has real capability-aware behavior; menus and popovers share one professional interaction language; sticky/read-only state is clear and synchronized; all themes remain readable; responsive layouts never squeeze three panes into unusable widths; accessibility contracts are visible and semantic; existing notes, storage, autosave, shortcuts, lifecycle operations, and editor conversions remain compatible; and all required automated and manual verification passes.
