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

## Window behavior

X11 uses native EWMH properties. GNOME Wayland uses the included, narrowly scoped Shell extension. If GNOME does not activate the extension immediately, log out and back in once. Unknown Wayland compositors keep note editing available but disable unsupported window toggles. KDE Wayland support is planned after the first release.

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
