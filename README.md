# Noor Notes

Noor Notes is a private, offline-first GTK4/libadwaita notes application for Linux, combining a fast sticky-note workflow with a modern three-pane library and focused writing editor.

## Features

- **Modern native library**: a focused GNOME header, private/offline status, Notes/Archived/Trash navigation, note previews, section counts, useful empty states, and keyboard-selectable rows.
- **Completely redesigned library**: adaptive sidebar, virtualized note cards, selected-note preview, seven navigation sections, responsive empty states, and a compact native header.
- **Dual editor modes**: existing rich notes remain compatible, while Markdown, plain-text, and code notes use GtkSourceView with line numbers, current-line highlighting, regex search, bookmarks, and syntax languages.
- **Premium appearance modes**: follow GNOME automatically or choose Light, Graphite, Midnight, or OLED; every palette updates open windows, editor surfaces, paper colors, and symbolic icon colors together.
- **Fast search and sorting**: Unicode-aware library search is debounced and stale results are discarded; sort by recently updated, recently created, title A–Z, or title Z–A.
- **Named and organized notes**: edit titles, add searchable tags, choose six accessible colours, and duplicate notes.
- **Reliable saving**: visible Saving/Saved state, retryable failures, and close-time flushing protect pending edits.
- **Encrypted locally**: SQLCipher protects note text, titles, tags, and history using a random key held by GNOME Keyring; existing databases migrate automatically and safely.
- **Rich text**: bold, italic, underline, strikethrough, reliable bullet and numbered lists, preset or custom positive whole-number font sizes, alignment, text and highlight colours, emoji, undo, and redo.
- **Find and replace**: navigate matches, replace one or all matches, and optionally match case or whole words while preserving Unicode character offsets.
- **Editor productivity**: word wrap, zoom from 50% to 300%, go to line, full screen, line/column position, line/word/character/selection counts, plain-text and Markdown export, and a keyboard-shortcut reference.
- **Accessible appearance**: compact consistent controls, symbolic icons, visible tooltips and focus states, semantic light/dark palettes, and optional readable paper colours.
- A searchable library with active, archived, and **Trash** notes.
- Source-install Xpad import that previews the migration and leaves the source files unchanged.
- Optional window controls, including Always on Top, all-workspaces, and opacity where the desktop supports them.

## Installation

### Release packages

Download `noor-notes_0.1.1_amd64.snap` or `noor-notes.flatpak` from the v0.1.1 release, verify it as described below, then install one package.

```bash
sudo snap install --dangerous ./noor-notes_0.1.1_amd64.snap
```

On Ubuntu or another APT-based system, install Flatpak and add a runtime remote before installing the Flatpak bundle:

```bash
sudo apt install flatpak
flatpak --user remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user ./noor-notes.flatpak
flatpak run io.github.saamaamr.NoorNotes
```

The Flatpak bundle needs Flatpak itself and a remote that can provide its GNOME 50 runtime; the command above adds Flathub only as that runtime remote, not as a Noor Notes installation source. The release Snap is strictly confined. The Flatpak exposes display integration, optional networking, and the desktop Secret Service; neither package has filesystem-wide access.

### Build from source on Ubuntu

For Ubuntu and other APT-based systems, clone the repository and run the installer:

```bash
git clone https://github.com/saamaamr/noor-notes.git
cd noor-notes
./scripts/install-ubuntu.sh
```

It installs the required system packages, installs Rust only if it is missing, builds the app, and installs it for the current user. From an existing checkout with the dependencies already available, use `./scripts/install-local.sh`. Launch Noor Notes from the application grid or run `~/.local/bin/noor-notes` (or the equivalent `XDG_BIN_HOME` location).

## Verify release artifacts

To download every published asset from a terminal:

```bash
release=https://github.com/saamaamr/noor-notes/releases/download/v0.1.1
curl -LO "$release/noor-notes_0.1.1_amd64.snap"
curl -LO "$release/noor-notes.flatpak"
curl -LO "$release/SHA256SUMS.txt"
sha256sum -c SHA256SUMS.txt
```

That command verifies both package files and each line must report `OK`. If you downloaded just one package through a browser, verify that selected artifact instead (replace the value with the Snap filename when appropriate):

```bash
artifact=noor-notes.flatpak
test "$(grep -Fc "  $artifact" SHA256SUMS.txt)" = 1
grep -F "  $artifact" SHA256SUMS.txt | sha256sum -c -
```

Choose **Editor mode** from the note's More menu to switch between Rich Text,
## Appearance and dark palettes

Use the moon button in a library or editor header to cycle quickly between
Graphite, Midnight, and OLED. For a direct choice, open the main menu's
**Appearance** submenu. **Appearance Settings** shows all five modes with
visual swatches. System follows GNOME while remembering the last preferred
dark palette.

Native symbolic icons automatically adopt the selected palette: neutral icons
follow foreground text, active icons use the accent, and destructive icons
Markdown, Plain Text, and Code. Noor Notes previews conversions first and creates
a recovery copy whenever rich styling would be lost.

The final command must report `OK` for the selected artifact before installation.

## First use and Xpad import

Create a note from the library. With a native/source installation, use the Xpad import control, review the preview, and confirm the import. Noor Notes does not modify Xpad or its files under `~/.config/xpad`.

The strict Snap and Flatpak packages cannot read the host `~/.config/xpad`, so their import control cannot migrate host Xpad notes in v0.1.1. No portal or file-selection import path is provided in those packages; use a native/source installation for Xpad migration.

## Rich text and Trash

Name a note from its title field or Rename action. Add comma-separated tags below the title and choose a note colour from Window Settings. Use the compact formatting toolbar to style selected text or insert an emoji. Repeated list-button clicks toggle the list instead of duplicating markers, and Enter continues or exits lists naturally. Preset sizes and a custom positive whole-number pixel size are available. Formatting is saved with the note; if a stored rich-text format is unsupported, Noor Notes safely displays its plain text instead.

Use the grouped editor controls or these shortcuts:

- **Ctrl+F** — find in the current note
- **Ctrl+H** — open find and replace
- **Ctrl+G** — go to line
- **Ctrl++**, **Ctrl+-**, **Ctrl+0** — zoom in, out, or reset
- **Ctrl+Z**, **Ctrl+Shift+Z**, or **Ctrl+Y** — undo or redo
- **Ctrl+B**, **Ctrl+I**, **Ctrl+U** — bold, italic, or underline
- **F11** — enter or leave full screen
- **Escape** — close the active find panel

Export from the More menu as UTF-8 plain text or Markdown. Open the keyboard-shortcuts reference from the main-window application menu.

Archive notes to hide them from the active list, or move them to Trash. In Trash, restore a note to the active list or choose **Permanently Delete** and confirm the destructive action. Permanent deletion removes the note from the local database.

## Window and sandbox limitations

On X11, Noor Notes uses native window-manager support. A source checkout can also install the included, narrowly scoped GNOME Shell extension. Sandboxed Snap and Flatpak packages do not install that extension or receive host Xpad-directory access. On GNOME Wayland, Always on Top can therefore remain unavailable unless it is installed separately outside the sandbox. Unsupported Wayland compositors keep note editing available while disabling unsupported window controls.

## Encrypted sync

Encrypted synchronization is not available to v0.1.1 users: the released app has no account, vault, or Supabase-project configuration flow, and its Sync action reports that cloud sync is not configured. Notes remain encrypted locally until a future release integrates that workflow.

## Data and recovery

Back up the database only while Noor Notes is closed. Its location depends on how the app is installed:

- **Source install:** `${XDG_DATA_HOME:-~/.local/share}/noor-notes/notes.db`.
- **Flatpak:** `~/.var/app/io.github.saamaamr.NoorNotes/data/noor-notes/notes.db`.
- **Snap:** normally `~/snap/noor-notes/current/.local/share/noor-notes/notes.db`; the app's snap-scoped `HOME` maps to the revision-specific `SNAP_USER_DATA` directory.

Back up the encrypted database together with a working GNOME Keyring backup. If the local database key is lost, the ciphertext cannot be recovered. Plain-text and Markdown exports are intentionally unencrypted; protect or delete them separately.

## Troubleshooting

- If a release package will not install, re-download it and `SHA256SUMS.txt`, then use the selected-artifact checksum commands above. Confirm that the exact package reports `OK`.
- If Always on Top is disabled on GNOME Wayland, use a source installation with the separately installed GNOME Shell extension, or use a supported window environment.
- If Xpad import cannot find your existing notes from a Snap or Flatpak install, use a native/source installation: those sandboxes cannot read the host `~/.config/xpad` in v0.1.1.
- If Sync says it is not configured, that is the current v0.1.1 limitation; there is no supported account or Supabase setup path yet.
- If source installation fails, run `./scripts/install-ubuntu.sh` on an APT-based system so the GTK4, Libadwaita, SQLite, OpenSSL, X11, and Secret Service dependencies are installed.
- If an Xpad note is skipped, inspect the import preview; it identifies entries that cannot be parsed before any import is committed.

## Development and build verification

Build a release binary with:

```bash
cargo build --release --package noor-notes
```

Before contributing a change, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
xvfb-run -a cargo test -p noor-windowing
gjs -m extensions/gnome/tests/test-policy.js
bash tests/e2e/two_device_sync.sh
```

## Release automation and Store status

Version tags build Snap and Flatpak artifacts in GitHub Actions. The `v0.1.1` tag creates the GitHub release with `noor-notes_0.1.1_amd64.snap`, `noor-notes.flatpak`, and `SHA256SUMS.txt` after the security gate passes.

Store publication is not automated: Snap Store upload remains a manual owner action, and a Flathub submission is not created by this project’s workflow. Use the published release artifacts above rather than assuming Snap Store or Flathub availability.

## Contributing

Contributions and bug reports are welcome. Open an [issue](https://github.com/saamaamr/noor-notes/issues) with reproduction details, and include relevant formatting, package, or workspace checks with a pull request.

## License

Noor Notes is licensed under [GPL-3.0-or-later](LICENSE).
