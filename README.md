# Noor Notes

Noor Notes is a native GTK4 sticky-note application for Linux. It works offline, imports Xpad without modifying it, supports optional per-note Always on Top, and provides end-to-end encrypted synchronization through a user-configured hosted Supabase project.

## Install on Ubuntu

Recommended installation:

```bash
git clone https://github.com/saamaamr/noor-notes.git
cd noor-notes
./scripts/install-ubuntu.sh
```

The installer adds the required Ubuntu packages, installs Rust only when it is missing, builds Noor Notes, and installs the application for your current user under `~/.local`. It will ask for your administrator password only when installing system packages.

If you already have GTK4, Libadwaita, SQLite, OpenSSL, X11 development headers, Rust stable, `pkg-config`, and `libsecret-tools`, install directly from an existing checkout:

```bash
./scripts/install-local.sh
```

Launch **Noor Notes** from the application grid or run `~/.local/bin/noor-notes`. Xpad remains installed and its files under `~/.config/xpad` remain untouched. Use the import button in the Noor Notes library to preview and confirm migration.

## Install the Snap package

After downloading a local Snap build, install it with:

```bash
sudo snap install --dangerous ./noor-notes_0.1.0_amd64.snap
```

The package is strictly confined and keeps its mutable data in Snap-managed user
directories. It requests desktop/display integration, optional network sync, and
the desktop password-manager service; it does not have filesystem-wide access.
The bundled GNOME Shell extension is not installed by the Snap. Consequently,
on GNOME Wayland the Always on Top control can remain unavailable unless the
extension is installed separately outside the Snap.

## Build and install the Flatpak package locally

Install Flatpak and its builder, then add Flathub as the runtime remote:

```bash
sudo apt install flatpak flatpak-builder
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install flathub org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable//25.08
```

From the repository root, build and install the user-scoped test package:

```bash
flatpak-builder --user --install --force-clean --install-deps-from=flathub \
  flatpak-build packaging/flatpak/io.github.saamaamr.NoorNotes.yml
flatpak run io.github.saamaamr.NoorNotes
```

The Flatpak manifest pins the released source commit and declares every Cargo
crate with a checksum, so Cargo builds offline inside the Flatpak sandbox. The
package requests only display integration, optional sync networking, and the
desktop Secret Service; it has no filesystem-wide access and does not bundle
the GNOME Shell extension.

## Window behavior

X11 uses native EWMH properties. The native checkout can use the included,
narrowly scoped GNOME Shell extension. If GNOME does not activate that separately
installed extension immediately, log out and back in once. Store-sandboxed
packages never install it automatically. Unknown Wayland compositors keep note
editing available but disable unsupported window toggles. KDE Wayland support is
planned after the first release.

## Encrypted sync

Apply `supabase/migrations/202608040001_encrypted_notes.sql` to a hosted Supabase project, then enter its project URL and anonymous key in Noor Notes. Confirm and store the displayed recovery key before sync can activate. Supabase receives ciphertext only. Credentials are stored in GNOME Keyring.

## Data and recovery

Local data is stored at `${XDG_DATA_HOME:-~/.local/share}/noor-notes/notes.db`. Back up that file while Noor Notes is closed. Keep the recovery key offline; losing both it and every unlocked device makes encrypted cloud notes unrecoverable. See [docs/security.md](docs/security.md) for the complete security model.

## Development verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
xvfb-run -a cargo test -p noor-windowing
gjs -m extensions/gnome/tests/test-policy.js
bash tests/e2e/two_device_sync.sh
```
