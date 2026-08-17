# Light Mode UI Refresh Design

## Goal

Refine Noor Notes Light Mode into a calm, compact, professional desktop productivity interface while preserving every existing feature, note, editor mode, theme, shortcut, window behavior, and persistence contract.

The work is an incremental visual redesign of the existing Rust/GTK4 and Libadwaita application. It does not introduce a second design system, a new dependency, or a component rewrite.

## Current Architecture

Noor Notes uses Rust 1.85, GTK4, Libadwaita, GtkSourceView, and one application CSS file at `apps/noor-notes/resources/design-system.css`. `AppearanceManager` adds one effective theme class to every registered window: Light, Graphite, Midnight, or OLED. Shared semantic GTK colors provide the base palette, while dark theme selectors override component surfaces and states.

The requested interface is owned by these existing units:

- `library_window.rs`: application header, search, sort, three-pane shell, footer, and responsive orchestration;
- `library_sidebar.rs`: navigation rows, icons, labels, counts, and collapsed state;
- `note_collection.rs` and `note_card.rs`: virtualized note list, selection, note identity, previews, and actions;
- `note_preview.rs`: library reading preview;
- `adaptive_layout.rs`: wide, medium, and narrow visibility rules;
- `editor_header.rs`, `editor_toolbar.rs`, `editor_canvas.rs`, and `editor_status_bar.rs`: separate note editor chrome and writing surface;
- `design-system.css`: semantic palette, spacing, typography, surfaces, icon states, and dark theme overrides.

## Design Direction

The interface will retain the Noor Notes identity of a calm notes library with a powerful editor when needed. Visual hierarchy will come from quiet neutral surfaces, typography, spacing, borders, and state changes rather than saturated fills, large shadows, gradients, or decorative effects.

The implementation will reuse Libadwaita widgets and the current symbolic icon theme. Native window controls remain managed by the desktop framework.

## Semantic Light Mode Foundation

The existing semantic GTK token layer will be refined near these values:

- application background: `#F7F8FA`;
- sidebar background: `#F6F7F9`;
- note-list background: `#FAFAFB`;
- surface and editor preview: `#FFFFFF`;
- neutral hover: `#F1F3F6`;
- primary text: `#1F2937`;
- secondary text: `#667085`;
- muted text: `#6B7280`, which maintains approximately `4.83:1` contrast on white for small metadata;
- border: `#E5E7EB`;
- subtle border: `#EEF0F2`;
- accent: `#4F6FE8`;
- accent hover: `#425FCC`;
- accent-soft selection: `#EEF2FF`;
- danger: `#DC2626`;
- success: `#16A34A`;
- focus ring: a translucent form of the accent.

Component rules will consume semantic tokens instead of introducing unrelated literal colors. The existing Graphite, Midnight, and OLED component overrides remain authoritative for Dark Mode. Shared structural changes must be checked in every theme.

## Pane Hierarchy

Wide library windows will present three distinct layers:

- a 220–240 pixel sidebar;
- a note-list pane targeting 330–340 pixels;
- a white preview surface that receives the remaining width.

Pane boundaries use one-pixel semantic borders with no heavy shadows. The app background remains visible only where it communicates structure. The preview stays intentionally spacious rather than being filled with unrelated panels.

## Sidebar

All Notes, Pinned, Favorites, Tags, Archived, Trash, and Recent remain in their existing order and retain the current symbolic icon system.

Rows will be approximately 42 pixels tall with an 8-pixel radius and consistent horizontal padding. Default icons and labels use neutral dark text; counts use the muted token and remain right aligned. Hover uses a neutral surface.

The active row uses the accent-soft surface, accent icon and label, semibold text, and a three-pixel inset accent rail. Selection is therefore communicated through background, position rail, color, and weight. The Trash icon becomes destructive only in an explicitly destructive context, not as a permanent unrelated icon color.

Collapsed navigation retains tooltips and the current accessible names. Expanded width targets 232 pixels while the existing compact collapsed behavior remains functionally unchanged.

## Note List and Cards

The note list uses the dedicated note-list surface with 14–16 pixels of outer spacing and 10–12 pixels between cards.

Each note card uses a white base, subtle border, approximately 10-pixel radius, and a minimal one-pixel elevation cue. The existing `NoteColor` classes remain the source of note identity. Yellow, cream, blue, green, rose, and lavender render as a four-pixel left rail and may apply only a very faint theme-safe tint. A note color never becomes a fully saturated card background.

Selected cards use a pale accent surface, restrained accent border, and subtle one-pixel selection ring while preserving the note-color rail. Hover adjusts the neutral border and elevation without turning the card blue.

Card typography uses a 15–16 pixel semibold title, a 13–14 pixel two-line preview with comfortable line height, 12-pixel metadata, and compact tag treatment. Titles, previews, tags, and metadata ellipsize or wrap within the card and never widen the list.

Pin and favorite remain symbolic status indicators. The archive action continues to appear for the selected active note. Archive and overflow controls use compact 32-pixel icon targets with transparent default surfaces and neutral hover states. The current tall pale action block is removed; context-menu and right-click behavior remain unchanged.

## Library Preview

The preview retains its existing 860-pixel reading-width clamp. Its document column stays aligned toward the left within the available surface instead of awkwardly centering short content. Responsive horizontal padding ranges from approximately 32 to 48 pixels, with efficient vertical spacing.

The title uses 28–30 pixel type, 650–700 weight, and a 1.2 line height. Metadata uses 13–14 pixel muted text followed by one subtle divider. Body text uses 16 pixels, approximately 1.6 line height, readable paragraph spacing, and selectable content.

Labels wrap safely at Unicode boundaries. Long uninterrupted strings use GTK/Pango wrapping behavior equivalent to breaking anywhere, so they do not extend underneath the window edge or controls. Source and code editor modes retain their controlled horizontal behavior where applicable.

## Rich Text and Source Editors

The previously approved Rich Text canvas margins remain exactly five pixels at the top and bottom and eight pixels at the left and right. This redesign must not replace those values with the preview's larger document padding.

The editor retains every mode, formatting control, writing-assistance service, search feature, status item, clamp behavior, and persistence path. Shared typography, surfaces, icon contrast, focus states, and toolbar consistency may be refined when those changes are theme-safe. GtkSourceView palette colors remain owned by its existing source style schemes.

## Application Header and Controls

The native Libadwaita header and window controls remain. The title and `Private notebook` subtitle stay centered. The New Note, application menu, appearance, sort, search, and native minimize/maximize/close controls preserve their behavior and accessible labels.

Top-bar controls target 34–36 pixel click areas, approximately 18-pixel symbolic icons, neutral dark Light Mode icon color, neutral hover, soft-accent active state, consistent eight-pixel radius, and visible focus. New Note remains the primary action but uses a refined compact treatment. The sort dropdown remains readable and compact with a neutral surface and explicit focus/open state. Search uses the same control language and the existing expandable `GtkSearchBar`.

One subtle semantic divider separates the header from content. Native close hover may use a destructive treatment only where supported safely by Libadwaita; the other window controls remain neutral.

## Footer and Status Bars

The library footer keeps section/result information on the left and local-save/privacy information on the right. Editor statistics, writing-assistance state, mode, and encoding also remain. Status bars target 28–30 pixels, 12-pixel text, the note-list neutral surface, and a subtle top border. They remain informational and visually quiet.

## Responsive Behavior

The existing responsive model remains the foundation:

- wide windows show sidebar, list, and preview;
- medium windows hide the sidebar first while preserving list and preview;
- narrow windows switch between list and preview through the existing Back control.

Wide pane sizing changes to reflect the 232-pixel sidebar and roughly 336-pixel list. Breakpoints may be adjusted only when tests and real screenshots demonstrate crowding. Header controls remain one row; lower-priority features stay in the existing overflow rather than wrapping into a second toolbar. No new pane-resizing architecture is added.

## Interaction and Accessibility

Hover feedback uses neutral surfaces by default. Accent is reserved for selection, active states, and focus. Transitions remain 120–180 milliseconds and follow GTK's desktop reduced-motion behavior.

Keyboard focus is visible on buttons, rows, cards, search, sort, and editor controls. Icon-only controls retain accessible labels and tooltips. Selected sidebar rows and cards use multiple visual cues rather than color alone. Practical click targets stay at least 32 pixels, with primary actions closer to 36 pixels. Tab order and the existing navigation model remain unchanged.

Secondary and muted text will be checked for readable Light Mode contrast. Dark theme overrides will be reviewed to prevent Light Mode literals leaking into shared components.

## Data and Error Boundaries

This work does not change note data, database schemas, storage, encryption, autosave, synchronization, action dispatch, search semantics, sorting values, or persistence. Existing failure and empty states remain functional; only their semantic typography and surface presentation may change.

No new runtime error path is introduced. CSS parsing failures remain covered by the existing GTK CSS validation test.

## Implementation Boundaries

Expected production changes are limited to the centralized design system and targeted UI files, principally:

- `apps/noor-notes/resources/design-system.css`;
- `apps/noor-notes/src/ui/library_window.rs`;
- `apps/noor-notes/src/ui/library_sidebar.rs`;
- `apps/noor-notes/src/ui/note_card.rs`;
- `apps/noor-notes/src/ui/note_collection.rs` when spacing or selection semantics require it;
- `apps/noor-notes/src/ui/note_preview.rs`;
- `apps/noor-notes/src/ui/adaptive_layout.rs` only if verified breakpoint changes are needed;
- editor header, toolbar, or status files only for shared visual consistency.

No storage, domain, sync, encryption, import, or writing-assistance implementation is in scope.

## Verification

Implementation proceeds test-first in small phases. Existing tests will be extended for semantic tokens, valid GTK CSS, calm selected states, note-color rails, compact actions, component accessibility, stable responsive visibility, source palette isolation, and Dark Mode overrides.

Verification includes:

- `cargo fmt --all -- --check`;
- Rust 1.85 workspace check;
- strict workspace Clippy;
- the focused GTK UI tests under Xvfb;
- the full workspace test suite under Xvfb;
- a locked optimized production build;
- `git diff --check` and an unrelated-change review.

The existing screenshot tooling will capture representative Light Mode library and editor states, selected and colored cards, long content, search, sort, archive, Trash, narrow, and maximized layouts. Representative Graphite, Midnight, and OLED views plus Light-to-Dark-to-Light switching will be inspected for regressions.

## Acceptance Criteria

The redesign is accepted when Light Mode has clear surface hierarchy, calm selected states, preserved note-color identity, compact card actions, readable typography, consistent symbolic icons, stronger top-bar controls, integrated status bars, intentional preview whitespace, safe long-content wrapping, visible focus, stable responsive behavior, and no Dark Mode or functional regression.
