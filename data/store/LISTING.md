# Noor Notes Snap Store listing

This file is the repository source of truth for the public Snap Store listing. Keep it aligned with `snapcraft.yaml` and the AppStream metadata before each stable release.

## Title

Noor Notes

## Summary

Private, encrypted notes with focused editing for Linux

## Description

Noor Notes is a calm, native GTK4/libadwaita notebook for private, offline-first writing on Linux.

Keep encrypted notes on your device and organize them with fast search, tags, pinning, favorites, Archive, and recoverable Trash. Choose a focused Rich Text, Markdown, Plain Text, or Code editor, with reliable autosave and recovery-aware saving.

Work comfortably with responsive layouts, accessible controls, and refined Light, Graphite, Midnight, and OLED appearances. Offline spelling, English grammar, and learned local predictions are enabled by default and can be controlled globally or per note. Optional online assistance stays disabled until you configure and test a provider.

Noor Notes contains no advertising, analytics, or tracking.

## Additional information

- License: `GPL-3.0-or-later`
- Website: <https://github.com/saamaamr/noor-notes>
- Source code: <https://github.com/saamaamr/noor-notes>
- Support and issues: <https://github.com/saamaamr/noor-notes/issues>
- Icon: `data/io.github.saamaamr.NoorNotes.svg`
- Featured banner: `data/store/noor-notes-featured-banner.png` (1920 × 640)

## Screenshot order

Upload no more than these five 1248 × 702 PNG files, in this order:

1. `data/screenshots/noor-notes-library.png`
2. `data/screenshots/noor-notes-editor.png`
3. `data/screenshots/noor-notes-writing-assistance.png`
4. `data/screenshots/noor-notes-dark.png`
5. `data/screenshots/noor-notes-responsive.png`

All screenshots show real Noor Notes widgets with synthetic data. They do not contain personal notes or credentials.

## Release cadence

- Version tags: publish the exact validated Snap to `latest/edge`, verify the Store-installed revision, then promote it unchanged to `latest/stable`.
- `latest/edge`: every Monday at 12:00 Bangladesh time, only when `main` changed.
- `latest/stable`: the first Monday of each month, only after the installed edge revision passes the Store smoke gate.
- Manual stable release: publish the current commit to edge, run the Store smoke gate, then promote that same tested revision to stable.
