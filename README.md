# Noor Notes

Noor Notes is a private, offline-first GTK4 sticky-note application for Linux. [Version 0.1.0](https://github.com/saamaamr/noor-notes/releases/tag/v0.1.0) is available as a locally installable Snap and Flatpak bundle.

## Features

- **Named and organized notes**: edit titles, add searchable tags, choose six accessible colours, sort the library, and duplicate notes.
- **Reliable saving**: visible Saving/Saved state, retryable failures, and close-time flushing protect pending edits.
- **Rich text**: bold, italic, underline, strikethrough, reliable bullet and numbered lists, preset or custom positive whole-number font sizes, alignment, text and highlight colours, emoji, undo, and redo.
- **Productivity**: Unicode-aware find-in-note, plain-text and Markdown export, keyboard shortcuts help, and polished empty states.
- A searchable library with active, archived, and **Trash** notes.
- Source-install Xpad import that previews the migration and leaves the source files unchanged.
- Optional window controls, including Always on Top, all-workspaces, and opacity where the desktop supports them.

## Installation

### Release packages

Download `noor-notes_0.1.0_amd64.snap` or `noor-notes.flatpak` from the [v0.1.0 release](https://github.com/saamaamr/noor-notes/releases/tag/v0.1.0), verify it as described below, then install one package.

```bash
sudo snap install --dangerous ./noor-notes_0.1.0_amd64.snap
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
release=https://github.com/saamaamr/noor-notes/releases/download/v0.1.0
curl -LO "$release/noor-notes_0.1.0_amd64.snap"
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

The final command must report `OK` for the selected artifact before installation.

## First use and Xpad import

Create a note from the library. With a native/source installation, use the Xpad import control, review the preview, and confirm the import. Noor Notes does not modify Xpad or its files under `~/.config/xpad`.

The strict Snap and Flatpak packages cannot read the host `~/.config/xpad`, so their import control cannot migrate host Xpad notes in v0.1.0. No portal or file-selection import path is provided in those packages; use a native/source installation for Xpad migration.

## Rich text and Trash

Name a note from its title field or Rename action. Add comma-separated tags below the title and choose a note colour from Window Settings. Use the compact formatting toolbar to style selected text or insert an emoji. Repeated list-button clicks toggle the list instead of duplicating markers, and Enter continues or exits lists naturally. Preset sizes and a custom positive whole-number pixel size are available. Formatting is saved with the note; if a stored rich-text format is unsupported, Noor Notes safely displays its plain text instead.

Use **Ctrl+F** inside a note to find text, **Ctrl+Z** to undo, and **Ctrl+Shift+Z** to redo. Export from the More menu as UTF-8 plain text or Markdown. Open the keyboard-shortcuts reference from the main-window keyboard icon.

Archive notes to hide them from the active list, or move them to Trash. In Trash, restore a note to the active list or choose **Permanently Delete** and confirm the destructive action. Permanent deletion removes the note from the local database.

## Window and sandbox limitations

On X11, Noor Notes uses native window-manager support. A source checkout can also install the included, narrowly scoped GNOME Shell extension. Sandboxed Snap and Flatpak packages do not install that extension or receive host Xpad-directory access. On GNOME Wayland, Always on Top can therefore remain unavailable unless it is installed separately outside the sandbox. Unsupported Wayland compositors keep note editing available while disabling unsupported window controls.

## Encrypted sync

Encrypted synchronization is not available to v0.1.0 users: the released app has no account, vault, or Supabase-project configuration flow, and its Sync action reports that cloud sync is not configured. Do not apply the bundled migration expecting a supported runtime setup. Notes remain local until a future release integrates that workflow.

## Data and recovery

Back up the database only while Noor Notes is closed. Its location depends on how the app is installed:

- **Source install:** `${XDG_DATA_HOME:-~/.local/share}/noor-notes/notes.db`.
- **Flatpak:** `~/.var/app/io.github.saamaamr.NoorNotes/data/noor-notes/notes.db`.
- **Snap:** normally `~/snap/noor-notes/current/.local/share/noor-notes/notes.db`; the app's snap-scoped `HOME` maps to the revision-specific `SNAP_USER_DATA` directory.

If database corruption is detected, Noor Notes preserves a timestamped `.corrupt-*.bak` copy beside the database. v0.1.0 has no configured cloud vault or recovery-key workflow, so a closed-app copy of this local database is the available recovery measure.

## Troubleshooting

- If a release package will not install, re-download it and `SHA256SUMS.txt`, then use the selected-artifact checksum commands above. Confirm that the exact package reports `OK`.
- If Always on Top is disabled on GNOME Wayland, use a source installation with the separately installed GNOME Shell extension, or use a supported window environment.
- If Xpad import cannot find your existing notes from a Snap or Flatpak install, use a native/source installation: those sandboxes cannot read the host `~/.config/xpad` in v0.1.0.
- If Sync says it is not configured, that is the current v0.1.0 limitation; there is no supported account or Supabase setup path yet.
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
xvfb-run -a cargo test -p noor-windowing
gjs -m extensions/gnome/tests/test-policy.js
bash tests/e2e/two_device_sync.sh
```

## Release automation and Store status

Version tags build Snap and Flatpak artifacts in GitHub Actions. The final `v0.1.0` tag creates the GitHub release with `noor-notes_0.1.0_amd64.snap`, `noor-notes.flatpak`, and `SHA256SUMS.txt`.

Store publication is not automated: Snap Store upload remains a manual owner action, and a Flathub submission is not created by this project’s workflow. Use the published release artifacts above rather than assuming Snap Store or Flathub availability.

## Contributing

Contributions and bug reports are welcome. Open an [issue](https://github.com/saamaamr/noor-notes/issues) with reproduction details, and include relevant formatting, package, or workspace checks with a pull request.

## License

Noor Notes is licensed under [GPL-3.0-or-later](LICENSE).
