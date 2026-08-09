# Comprehensive Screenshot Gallery Design

Date: 2026-08-09
Status: Approved

## Goal

Create a truthful, comprehensive visual inventory of the current Noor Notes GTK4/libadwaita application. The gallery must cover major features, themes, menus, dialogs, editor modes, card actions, View-Only Mode, and responsive window states. It must include both reusable individual screenshots and categorized contact sheets.

The final assets belong under `data/screenshots/`. The existing seven README/AppStream filenames remain stable and are refreshed from the current application.

## Safety and Capture Source

All visible application surfaces must come from the real running Noor Notes widgets. Capture the application through a dedicated temporary harness with a distinct non-production application identity and a temporary encrypted database containing deterministic sample notes.

The workflow must not:

- open, copy, alter, or capture the normal Noor Notes database;
- read or modify personal notes, settings, or keyring entries;
- activate or stop the installed Noor Notes process;
- expose private names, URLs, credentials, or local filesystem details;
- add mock controls or edit application content into a screenshot afterward;
- rebuild, upload, release, or modify any Snap Store revision.

Before and after capture, record the normal database path, size, timestamp, and hash when it is safe to do so. If the installed application is running, leave it untouched and use process/path checks instead of hashing active WAL state.

## Output Structure

Keep the established root assets:

- `noor-notes-library.png`
- `noor-notes-editor.png`
- `noor-notes-dark.png`
- `noor-notes-formatting.png`
- `noor-notes-find-replace.png`
- `noor-notes-trash.png`
- `noor-notes-responsive.png`

Add organized subdirectories:

- `library/`
- `editor/`
- `formatting/`
- `search/`
- `modes/`
- `view-only/`
- `menus/`
- `themes/`
- `responsive/`
- `settings/`
- `contact-sheets/`

Add `data/screenshots/INDEX.md` as the human-readable filename-to-feature inventory. Each individual screenshot uses a descriptive kebab-case filename and a 1248 x 702 RGB PNG canvas. Contact sheets use a larger canvas sized for legible labeled thumbnails and retain PNG RGB output.

## Window State Interpretation

Capture these distinct window states:

- maximized desktop window;
- normal restored window;
- compact non-maximized window;
- narrow adaptive window;
- short window that forces the editor More menu into multiple columns.

A genuinely minimized window has no visible application surface, so “minimized” is represented by the compact non-maximized state. This distinction is documented in the index.

## Deterministic Sample Data

Seed the temporary encrypted repository with polished non-personal notes that exercise:

- rich text with headings, emphasis, lists, alignment, colors, and highlights;
- Markdown with headings, lists, links, and code fencing;
- plain Unicode text, including Bengali and Arabic samples where useful for typography;
- source code with syntax highlighting;
- pinned and favorite states;
- multiple tags and note colors;
- recent, archived, and trashed lifecycle states;
- content suitable for search, replace, whole-word, case-sensitive, and regex examples;
- a long note that demonstrates scrolling, word wrap, line numbers, and statistics.

Use fictional product-planning and writing content. Do not use real credentials, private URLs, personal identifiers, or unsupported product claims.

## Capture Inventory

### Library

Capture the maximized library with sidebar, note list, selected card, and preview; normal and compact library states; each populated navigation view; tags; search results; no-results state; sort menu; selected-card quick actions; active, archived, and trash card context menus; restore; permanent-delete confirmation; and meaningful empty states.

### Editor

Capture maximized, normal, compact, and narrow editors; title and tags hierarchy; compact toolbar; status bar; enabled and disabled undo/redo states; visible save-state variants where they can be produced truthfully; note color selection; archive and trash actions; go-to-line; word-wrap and zoom states; export-related menus; and the multi-column More menu in a short window.

### Rich Formatting

Capture the formatting popover overview and focused states for text style, paragraph controls, lists, alignment, font-size presets, custom font size, text-color presets, custom text color, highlight presets, custom highlight color, reset controls, and emoji insertion. Screenshots must display real selected formatting states and never imply unsupported formatting in source modes.

### Find and Replace

Capture find-only, replace, nonzero result count, no-results, match-case, whole-word, regex where supported, and replacement-result states. The panel must operate on the note body.

### Editor Modes

Capture Rich Text, Markdown, Plain Text, and Code. Markdown and Code must show their real syntax palettes; Plain Text must show its intended uniform body color. Include source-editor line numbers, current-line highlight, bookmarks, word wrap, and mode-specific menu states where available.

### View-Only and Card Reading

Capture the selected library card and preview, full View-Only Mode, compact View-Only Mode, and the discoverable More-menu action that enters View-Only Mode. The reading window must contain only native window controls and the formatted note body, as implemented.

### Themes

Capture Light, Graphite, Midnight, and OLED for both the library and editor. Include the application Appearance menu and Appearance Settings. Verify that symbolic icon colors, text contrast, note paper colors, selections, and editor source palettes visibly adapt.

### Menus, Dialogs, and Settings

Capture the application menu, Appearance submenu, sort menu, card context menus, editor More menu, short-window multi-column More menu, formatting popover, insert/emoji surfaces, note-color menu, keyboard-shortcuts reference, confirmation dialogs, and every currently available settings surface that presents meaningful UI.

### Responsive Use Cases

Capture maximized, restored, compact, narrow, and short layouts. Demonstrate preview visibility on wide windows, preview removal/adaptation on narrow windows, toolbar wrapping, accessible More-menu behavior, and compact View-Only Mode.

## Capture and Composition Rules

- Capture only the active Noor Notes application window or transient surface.
- Exclude the desktop panel, dock, terminal, notifications, cursor, and unrelated windows.
- Preserve native GTK theme rendering and symbolic icons.
- Normalize individual assets to 1248 x 702 without stretching.
- Scale down proportionally when necessary; do not upscale compact windows.
- Center compact captures on a neutral color sampled from the active application theme.
- Do not apply filters, artificial shadows, or post-capture UI reconstruction.
- Keep controls and text legible at README preview size.
- Reject any image with clipping, loading states, broken menus, inaccessible contrast, personal information, or inconsistent framing.

## Contact Sheets

Generate one labeled contact sheet per category and one master overview sheet. Build sheets only from the approved individual screenshots. Labels may be added outside screenshot boundaries for navigation, but the screenshots themselves must remain unedited.

Contact sheets are documentation assets, not AppStream screenshots. Their dimensions may exceed 1248 x 702 to keep thumbnails and labels readable.

## Automation and Reproducibility

Prefer a temporary screenshot harness plus bounded accessibility automation. Locate controls by accessible role/name rather than brittle desktop coordinates. Every lookup and transition must have a timeout and report the missing surface clearly.

Temporary harness source, raw captures, sample databases, logs, and automation files must be removed before completion unless a small repository-owned capture script is intentionally retained, documented, tested, and contains no machine-specific paths or secrets.

## Documentation and Metadata

Refresh the seven established root images without renaming them so README and AppStream links remain valid. Do not add every detailed screenshot to AppStream. Use `INDEX.md` and contact sheets to expose the complete gallery without making the main README unwieldy.

README changes are limited to accurate gallery presentation if the refreshed images or gallery organization require it. Packaging identity and store revision metadata remain unchanged.

## Verification

- Visually inspect every individual image and every contact sheet.
- Confirm each individual image is a nonempty 1248 x 702 RGB PNG.
- Confirm every index entry resolves to an existing image.
- Confirm contact sheets contain the expected category images and readable labels.
- Run `tests/store_metadata.sh` for the established root gallery.
- Run AppStream validation when metadata is touched.
- Run `git diff --check` and verify that no temporary databases, logs, raw captures, credentials, build artifacts, or local paths are staged.
- Confirm the normal Noor Notes data remained untouched.
- Confirm no Snap Store, release, analytics, network-service, or package-install action occurred.

## Completion Report

Report the exact output directories, total individual screenshot count, contact-sheet count, capture method, isolated-data location, visual inspection result, validation commands, repository status, and any state that could not be captured truthfully.

## Out of Scope

This task does not redesign application UI, change storage behavior, modify the application ID, alter package metadata, upload a Snap, publish a release, add analytics or cloud services, or use personal notes as demonstration content.
