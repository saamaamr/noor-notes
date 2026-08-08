# Current Interface Screenshot Refresh Design

Date: 2026-08-08
Status: Approved

## Goal

Replace the two obsolete store screenshots and expand the repository gallery with truthful images of the current Noor Notes interface. The images must present the real GTK4/libadwaita application without exposing or modifying personal notes.

## Capture Source and Data Safety

Launch the current release build against an isolated temporary XDG data directory. Populate it with polished, non-personal sample notes covering planning, writing, code, and recovery workflows. Do not open, copy, alter, or capture the user's normal notes database. Temporary databases, logs, helper files, and capture artifacts outside the approved output directory must not be committed.

All visible application surfaces must come from the real running application. Do not replace the interface with mockups or edit controls into screenshots after capture. Cropping, resizing, and neutral background padding are allowed only to produce the required store-safe canvas.

## Screenshot Set

Create seven 1248 x 702 RGB PNG files under `data/screenshots/`:

1. `noor-notes-library.png` — light appearance with the navigation sidebar, several note cards, a selected note, and its preview.
2. `noor-notes-editor.png` — a polished Rich Text note showing the current editor hierarchy, compact toolbar, writing canvas, and status bar.
3. `noor-notes-dark.png` — a representative current dark palette with readable cards, preview, icons, and metadata.
4. `noor-notes-formatting.png` — Rich Text mode with the compact formatting and professional colour controls visible.
5. `noor-notes-find-replace.png` — the editor's inline find-and-replace workflow with a visible query and result state.
6. `noor-notes-trash.png` — Trash containing recoverable sample notes without oversized destructive actions.
7. `noor-notes-responsive.png` — the real narrow adaptive layout, centered within the 1248 x 702 output without stretching.

The sample content must contain no credentials, personal identifiers, private URLs, or claims that are not supported by the current application.

## Composition and Visual Quality

- Capture application windows only; exclude the desktop panel, dock, notifications, terminal, cursor, and unrelated windows.
- Use consistent framing, neutral breathing room, and crisp native-resolution rendering.
- Keep text large enough to remain legible in README and software-store previews.
- Prefer populated, purposeful states over blank canvases while avoiding visual clutter.
- Preserve native theme colours and symbolic icons; do not apply filters or artificial shadows.
- Visually inspect each final image for clipping, transient loading states, accidental personal data, and inconsistent scale.

## Documentation and Metadata

Keep `noor-notes-editor.png` and `noor-notes-library.png` as the two AppStream source images. Add the expanded gallery to the README with accurate alt text and concise captions. Add the additional screenshots to AppStream only when each image truthfully represents a stable feature and retains the required 1248 x 702 dimensions.

## Verification

- Confirm all seven files are nonempty 1248 x 702 RGB PNGs.
- Run `tests/store_metadata.sh` and relevant AppStream validation.
- Inspect every final image visually.
- Confirm the normal notes database and keyring entries were not modified by the capture workflow.
- Confirm Git contains no temporary database, log, helper, credential, or generated build output.

## Out of Scope

This work does not redesign the application, alter storage, change packaging identity, upload a Snap, publish a release, or capture the GNOME lock screen.
