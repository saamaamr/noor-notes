# Noor Notes Design

## Purpose

Noor Notes is a modern Linux sticky-note application replacing Xpad. It preserves the low-friction behavior of desktop notes while adding optional per-note Always on Top, dependable resizing, rich customization, offline-first storage, encrypted cloud synchronization, and automatic Xpad import.

## Supported environment

- Linux desktop only.
- GTK4 provides the native application interface.
- X11 receives direct native window controls.
- GNOME Wayland and KDE Wayland receive small desktop-specific adapters where compositor security prevents ordinary applications from controlling stacking and placement.
- Other Wayland desktops retain core note functionality and expose unsupported window controls as unavailable rather than pretending they succeeded.

The application detects the active display protocol and desktop environment at startup and chooses the appropriate window-control backend.

## User experience

### Notes

Each note is an independent resizable window with remembered position, size, color, opacity, font, text size, workspace behavior, and pin state. Notes autosave as the user types. Closing a note hides its window without deleting its content.

The compact note toolbar provides:

- Pin or unpin Always on Top.
- Show on the current workspace or all workspaces where supported.
- Change color, opacity, font, and text size.
- Duplicate, archive, or move to trash.
- Access formatting and checklist controls.

The pin state is optional and stored per note. A note is never forced above other windows unless its pin toggle is enabled.

### Main window

The main window provides search, active notes, archived notes, trash, synchronization status, account settings, and customization defaults. Search works locally and remains available offline.

### Tray and shortcuts

A tray indicator provides New Note, Show/Hide Notes, Search, Sync Now, and Quit. Configurable global shortcuts create a note and show or hide all notes. Desktop-specific shortcut registration is used where required.

## Window-control architecture

A small `WindowController` interface isolates compositor-specific behavior:

- `X11WindowController`: uses native X11/EWMH operations for Above, workspace stickiness, positioning, opacity, and resizing.
- `GnomeWaylandController`: communicates with a narrowly scoped GNOME Shell companion extension using D-Bus. The extension only recognizes Noor Notes windows and applies requested stacking/workspace behavior.
- `KdeWaylandController`: integrates with supported KWin scripting/window-rule APIs.
- `FallbackWindowController`: supports ordinary GTK resizing and reports unsupported compositor-controlled operations clearly.

The application remains usable if an adapter is missing or disabled. Notes and synchronization must never depend on an adapter.

## Local data

SQLite is the source of truth on each device. The database stores note identifiers, encrypted or plaintext local content as selected by the encryption design, formatting, window geometry, timestamps, revision metadata, archive/trash state, and synchronization state.

Writes use transactions. Text changes are debounced for efficiency, while window geometry is saved after movement or resizing settles. A local change journal makes interrupted synchronization retryable and idempotent.

## End-to-end encryption

All note content and sensitive metadata are encrypted locally before upload. Supabase receives ciphertext, non-secret synchronization identifiers, revision numbers, timestamps, and deletion markers.

Encryption uses a well-reviewed authenticated-encryption library rather than custom cryptography. A key derived from the user's encryption passphrase unlocks a randomly generated vault key. The vault key encrypts note records. Device enrollment transfers the vault key through a recovery-key flow; Supabase never receives the plaintext vault key or passphrase.

Losing both the passphrase and recovery key makes encrypted notes unrecoverable. The onboarding flow must state this explicitly and require the recovery key to be saved before enabling synchronization.

## Cloud synchronization

Supabase provides hosted authentication, encrypted record storage, and change notifications. Email/password authentication is sufficient for the first release.

Synchronization is offline-first:

1. A local edit commits to SQLite immediately.
2. The change journal records a pending encrypted revision.
3. The sync worker uploads pending revisions when online.
4. Remote revisions are downloaded, authenticated, decrypted, and merged locally.
5. UI updates are emitted only after a successful local transaction.

Conflicts are handled per note. Non-overlapping metadata changes merge automatically. Concurrent content edits preserve both versions: the latest becomes current and the other becomes a recoverable conflict copy. Deletions use tombstones so offline devices cannot silently resurrect deleted notes. Trash retention defaults to 30 days.

## Xpad migration

On first run, Noor Notes detects `~/.config/xpad`. Import is explicit and previewed before execution.

- Xpad content becomes Noor Notes content.
- Available Xpad geometry and style information is mapped when valid.
- Every imported note receives a new stable Noor Notes identifier.
- Original Xpad files remain unchanged.
- An import receipt prevents accidental duplicate imports while allowing the user to rerun migration deliberately.

Malformed or unreadable notes are skipped and listed in a human-readable import report.

## Security and privacy

- Secrets use the desktop keyring when available and are never written to logs.
- Database and configuration files use owner-only permissions.
- Supabase row-level security restricts every record to its authenticated owner.
- Desktop adapters accept requests only from the Noor Notes application and operate only on Noor Notes windows.
- Logs exclude note text, encryption keys, access tokens, and passwords.
- Clipboard operations follow normal desktop behavior and are not synchronized.

## Failure behavior

- Network loss: edits continue locally and synchronization retries with bounded exponential backoff.
- Authentication expiry: local editing continues; the UI requests sign-in without discarding pending changes.
- Decryption failure: the affected remote record is quarantined and never overwrites a valid local note.
- Database failure: the app stops writes, preserves the original database, and offers backup/recovery guidance.
- Missing desktop adapter: unsupported pin/workspace controls are disabled with a concise explanation.
- Supabase outage: pending operations remain locally queued.

## Technology boundaries

- GTK4 application shell and note windows.
- Rust for the core application, SQLite access, encryption, synchronization, migration, and window-controller abstraction.
- Libadwaita may be used for the main management window, while note windows retain a lightweight custom GTK presentation.
- Supabase is accessed through documented HTTPS and realtime interfaces.
- The GNOME companion is a minimal JavaScript Shell extension; the KDE companion is a minimal KWin integration.

Rust is preferred over Python for a long-lived desktop process handling encryption, concurrency, and native window integration. Desktop-specific code remains isolated from note and sync logic.

## Testing

- Unit tests: encryption envelopes, key derivation parameters, note model, merge rules, tombstones, import parsing, and window-controller selection.
- Database tests: migrations, transactions, change journal recovery, trash retention, and corrupted-record handling.
- Sync integration tests: offline edits, retries, duplicate delivery, concurrent edits, expired sessions, and conflict copies against a test Supabase project.
- Xpad fixture tests: valid notes, missing metadata, malformed files, duplicate imports, and Unicode content.
- Desktop tests: X11 Above toggling and geometry; GNOME/KDE adapter authorization and unsupported-backend behavior.
- UI tests: note creation, autosave, resizing, pin toggle, customization, search, archive, trash, accessibility names, and keyboard navigation.
- Security tests: row-level access isolation, secret-free logs, tamper detection, and recovery-key enrollment.

## Initial release scope

The first release includes local notes, Xpad import, resizing and geometry persistence, per-note customization, search, archive/trash, X11 window controls, GNOME Wayland Always on Top adapter, Supabase authentication, encrypted offline-first synchronization, and conflict copies.

KDE Wayland integration follows after the core and GNOME path are stable. Collaboration, shared notes, mobile clients, attachments, drawing, OCR, and web access are intentionally outside the initial scope.

## Success criteria

- Existing Xpad notes import without altering the source files.
- Notes remain editable and searchable with no network connection.
- Enabling or disabling Always on Top affects only the selected note.
- X11 and GNOME Wayland provide reliable pin behavior through their selected backends.
- Two Linux devices converge after offline and concurrent edits without silent content loss.
- Supabase cannot read note content.
- Restarting the app preserves note text, geometry, style, archive/trash state, and pending synchronization work.
