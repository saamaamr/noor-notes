# Noor Notes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an offline-first GTK4 sticky-note application for Linux with Xpad import, optional per-note Always on Top, end-to-end encrypted Supabase synchronization, and GNOME Wayland integration.

**Architecture:** A Rust workspace separates domain, storage, encryption, synchronization, migration, and compositor integration from the GTK application. SQLite is the local source of truth; encrypted revisions synchronize through Supabase. X11 window operations are native, while GNOME Wayland uses a narrowly scoped Shell extension over D-Bus.

**Tech Stack:** Rust 1.85+, GTK4/Libadwaita, SQLite via `sqlx`, XChaCha20-Poly1305 and Argon2id, Supabase HTTPS/realtime APIs via `reqwest` and `tokio-tungstenite`, X11RB, GNOME Shell JavaScript, GLib test utilities.

## Global Constraints

- Linux desktop only; GTK4 is the native application interface.
- X11 receives direct window controls; GNOME Wayland uses a companion extension.
- Every note autosaves and remembers geometry, appearance, workspace behavior, and optional pin state.
- SQLite remains the source of truth and editing must work without a network connection.
- Supabase must receive ciphertext only for note content and sensitive metadata.
- Existing files under `~/.config/xpad` must never be modified.
- Secrets and note content must never be written to logs.
- Missing desktop adapters must degrade gracefully without blocking note editing.
- Copy and paste use the system clipboard normally; clipboard contents are never persisted or synchronized separately.
- Initial release includes X11 and GNOME Wayland; KDE Wayland is deferred.

---

## File Map

- `Cargo.toml`: workspace members and shared dependency versions.
- `crates/domain/`: note, style, geometry, revision, and conflict types with no UI/database dependencies.
- `crates/storage/`: SQLite migrations, repositories, local change journal, trash retention.
- `crates/xpad-import/`: read-only Xpad parser, preview, receipt, and import report.
- `crates/windowing/`: protocol detection and X11/fallback `WindowController` implementations.
- `crates/crypto/`: vault creation/unlock, recovery key, authenticated encryption envelopes.
- `crates/sync/`: Supabase client, upload/download worker, merge and conflict-copy logic.
- `apps/noor-notes/`: GTK application, note windows, main window, tray/actions, settings.
- `extensions/gnome/`: GNOME Shell companion restricted to Noor Notes windows.
- `supabase/migrations/`: encrypted-record schema and row-level-security policies.
- `tests/fixtures/xpad/`: representative, malformed, and Unicode Xpad fixtures.

---

### Task 1: Bootstrap the Rust/GTK workspace

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/main.rs`
- Create: `tests/workspace_smoke.sh`
- Create: `crates/{domain,storage,xpad-import,windowing,crypto,sync}/Cargo.toml`
- Create: `crates/{domain,storage,xpad-import,windowing,crypto,sync}/src/lib.rs`

**Interfaces:**
- Produces: buildable workspace packages `noor-domain`, `noor-storage`, `noor-xpad-import`, `noor-windowing`, `noor-crypto`, `noor-sync`, and binary `noor-notes`.

- [ ] **Step 1: Install reproducible Ubuntu build prerequisites**

Run:

```bash
pkexec apt-get install -y build-essential curl pkg-config libgtk-4-dev libadwaita-1-dev libsqlite3-dev libssl-dev libx11-dev libxcb1-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
```

Expected: `pkg-config --modversion gtk4` and `cargo --version` succeed.

- [ ] **Step 2: Write the workspace smoke test before the binary exists**

Create `tests/workspace_smoke.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cargo check --workspace --all-targets
cargo test --workspace
```

Run: `bash tests/workspace_smoke.sh`

Expected: FAIL because the workspace manifests do not exist.

- [ ] **Step 3: Create the workspace and minimal GTK binary**

Use these shared dependencies in the root manifest:

```toml
[workspace]
resolver = "2"
members = ["apps/noor-notes", "crates/*"]

[workspace.dependencies]
anyhow = "1"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
gtk = { package = "gtk4", version = "0.10", features = ["v4_14"] }
libadwaita = { version = "0.8", features = ["v1_5"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync", "time"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

`main.rs` must construct `adw::Application`, connect `activate`, create one `adw::ApplicationWindow`, and call `run()`.

- [ ] **Step 4: Verify bootstrap**

Run: `bash tests/workspace_smoke.sh`

Expected: PASS with zero test failures.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml .gitignore apps crates tests/workspace_smoke.sh
git commit -m "build: bootstrap Noor Notes workspace"
```

### Task 2: Define the note domain model

**Files:**
- Create: `crates/domain/src/note.rs`
- Create: `crates/domain/src/style.rs`
- Modify: `crates/domain/src/lib.rs`
- Test: `crates/domain/tests/note_model.rs`

**Interfaces:**
- Produces: `Note`, `NoteId`, `NoteStyle`, `WindowGeometry`, `Revision`, `NoteState`, and `Note::new(now)`.

- [ ] **Step 1: Write failing model tests**

```rust
#[test]
fn new_note_has_safe_defaults() {
    let now = chrono::Utc::now();
    let note = Note::new(now);
    assert!(note.content.is_empty());
    assert_eq!(note.style.opacity, 1.0);
    assert!(!note.always_on_top);
    assert_eq!(note.state, NoteState::Active);
}

#[test]
fn opacity_is_clamped() {
    let mut style = NoteStyle::default();
    style.set_opacity(1.7);
    assert_eq!(style.opacity, 1.0);
}
```

Run: `cargo test -p noor-domain --test note_model`

Expected: FAIL because the types are undefined.

- [ ] **Step 2: Implement serializable domain types**

Use `Uuid` newtypes, UTC timestamps, geometry defaults `360x320`, opacity range `0.35..=1.0`, and states `Active`, `Archived`, `Trashed { deleted_at }`.

- [ ] **Step 3: Verify and commit**

Run: `cargo test -p noor-domain`

```bash
git add crates/domain
git commit -m "feat: define note domain model"
```

### Task 3: Implement SQLite storage and change journal

**Files:**
- Create: `crates/storage/migrations/0001_initial.sql`
- Create: `crates/storage/src/repository.rs`
- Create: `crates/storage/src/journal.rs`
- Create: `crates/storage/src/error.rs`
- Modify: `crates/storage/src/lib.rs`
- Test: `crates/storage/tests/repository.rs`

**Interfaces:**
- Consumes: `noor_domain::Note`, `NoteId`, `Revision`.
- Produces: `SqliteNoteRepository::open(path)`, `save_note`, `get_note`, `search_notes`, `archive`, `trash`, `restore`, `pending_changes`, `ack_change`.

- [ ] **Step 1: Write a failing transaction/restart test**

```rust
#[tokio::test]
async fn save_survives_reopen_and_creates_one_pending_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    let repo = SqliteNoteRepository::open(&path).await.unwrap();
    let mut note = Note::new(Utc::now());
    note.content = "offline text".into();
    repo.save_note(&note).await.unwrap();
    drop(repo);
    let reopened = SqliteNoteRepository::open(&path).await.unwrap();
    assert_eq!(reopened.get_note(note.id).await.unwrap().unwrap().content, "offline text");
    assert_eq!(reopened.pending_changes(10).await.unwrap().len(), 1);
}
```

Run: `cargo test -p noor-storage --test repository`

Expected: FAIL because `SqliteNoteRepository` is undefined.

- [ ] **Step 2: Add the normalized schema**

Create tables `notes`, `note_styles`, `window_geometry`, `change_journal`, `sync_state`, `import_receipts`; enable WAL, foreign keys, and owner-only file permissions.

- [ ] **Step 3: Implement transactional repository methods**

Every user-visible mutation must update its note revision and insert one idempotent journal row in the same transaction.

- [ ] **Step 4: Test recovery, search, trash, and commit**

Add tests for case-insensitive search, archive/trash/restore, malformed-database errors, and creation of a timestamped database backup before any destructive recovery attempt. The application must report a database failure without silently replacing the user’s data.

Run: `cargo test -p noor-storage`

```bash
git add crates/storage
git commit -m "feat: add offline SQLite note storage"
```

### Task 4: Import Xpad notes without modifying them

**Files:**
- Create: `crates/xpad-import/src/parser.rs`
- Create: `crates/xpad-import/src/importer.rs`
- Create: `crates/xpad-import/src/report.rs`
- Modify: `crates/xpad-import/src/lib.rs`
- Create: `tests/fixtures/xpad/{default-style,info-VALID1,content-VALID1,info-BROKEN1,content-UNICODE1}`
- Test: `crates/xpad-import/tests/import.rs`

**Interfaces:**
- Produces: `scan_xpad(path) -> ImportPreview`, `import_xpad(preview, repository) -> ImportReport`.

- [ ] **Step 1: Copy sanitized fixtures and record source hashes**

Use copies of the Xpad file shapes, replacing personal text with `Test note`, `বাংলা পরীক্ষা`, and `اختبار`. Record SHA-256 hashes before import.

- [ ] **Step 2: Write failing preview/import tests**

```rust
#[tokio::test]
async fn import_is_read_only_and_idempotent() {
    let before = hash_fixture_tree(fixture_path());
    let preview = scan_xpad(fixture_path()).unwrap();
    assert_eq!(preview.importable.len(), 2);
    let first = import_xpad(&preview, &repo).await.unwrap();
    let second = import_xpad(&preview, &repo).await.unwrap();
    assert_eq!(first.imported, 2);
    assert_eq!(second.imported, 0);
    assert_eq!(before, hash_fixture_tree(fixture_path()));
}
```

Run: `cargo test -p noor-xpad-import`

Expected: FAIL because scanning/import is undefined.

- [ ] **Step 3: Implement tolerant parsing and receipts**

Map content, geometry, colors, font, and timestamps when valid. Skip malformed files, include their paths and errors in `ImportReport`, and key receipts by source directory plus source-file hash.

- [ ] **Step 4: Verify against a copy of the real Xpad directory**

Run:

```bash
cp -a ~/.config/xpad /tmp/noor-xpad-fixture
cargo test -p noor-xpad-import
diff -qr ~/.config/xpad /tmp/noor-xpad-fixture
```

Expected: tests pass and `diff` reports no differences.

- [ ] **Step 5: Commit**

```bash
git add crates/xpad-import tests/fixtures/xpad
git commit -m "feat: add read-only Xpad migration"
```

### Task 5: Add protocol detection and X11 window controls

**Files:**
- Create: `crates/windowing/src/controller.rs`
- Create: `crates/windowing/src/detect.rs`
- Create: `crates/windowing/src/x11.rs`
- Create: `crates/windowing/src/fallback.rs`
- Modify: `crates/windowing/src/lib.rs`
- Test: `crates/windowing/tests/detect.rs`
- Test: `crates/windowing/tests/x11_integration.rs`

**Interfaces:**
- Produces:

```rust
#[async_trait]
pub trait WindowController {
    async fn set_above(&self, window: NativeWindowId, enabled: bool) -> Result<()>;
    async fn set_all_workspaces(&self, window: NativeWindowId, enabled: bool) -> Result<()>;
    async fn set_opacity(&self, window: NativeWindowId, value: f64) -> Result<()>;
    fn capabilities(&self) -> WindowCapabilities;
}
pub fn detect_backend(env: &Environment) -> BackendKind;
```

- [ ] **Step 1: Test deterministic backend selection**

Assert X11 for `XDG_SESSION_TYPE=x11`, GNOME adapter for `wayland` plus `GNOME`, and fallback for unknown Wayland.

- [ ] **Step 2: Implement fallback first and verify tests pass**

Fallback must return `UnsupportedOperation` for stacking, workspace, and native opacity requests and expose those capabilities as false.

- [ ] **Step 3: Write failing X11 Above test under Xvfb**

Create a test window, call `set_above(true)`, and assert `_NET_WM_STATE_ABOVE`; then disable and assert removal. Set opacity and assert the expected `_NET_WM_WINDOW_OPACITY` value.

- [ ] **Step 4: Implement X11RB/EWMH controller and commit**

Run: `xvfb-run -a cargo test -p noor-windowing`

```bash
git add crates/windowing
git commit -m "feat: add X11 note window controls"
```

### Task 6: Build note windows and autosave

**Files:**
- Create: `apps/noor-notes/src/app.rs`
- Create: `apps/noor-notes/src/note_window.rs`
- Create: `apps/noor-notes/src/note_toolbar.rs`
- Create: `apps/noor-notes/src/autosave.rs`
- Create: `apps/noor-notes/resources/noor-notes.gresource.xml`
- Create: `apps/noor-notes/resources/note-window.ui`
- Create: `apps/noor-notes/resources/style.css`
- Test: `apps/noor-notes/tests/autosave.rs`

**Interfaces:**
- Consumes: repository and `WindowController`.
- Produces: `NoteWindow::new(note, repository, controller)` and `AutosaveQueue::schedule(NoteDraft)`.

- [ ] **Step 1: Test debounced autosave with paused Tokio time**

Assert multiple edits within 400 ms produce one save, while closing flushes immediately.

- [ ] **Step 2: Implement autosave independently of GTK**

Use a cancellable per-note timer and a `flush(note_id)` method. Never block the GTK main thread.

- [ ] **Step 3: Build the GTK note window**

Include editable text, drag-resizable native window, compact toolbar, pin toggle, workspace toggle, color/opacity/font controls, duplicate/archive/trash actions, and persisted geometry.

- [ ] **Step 4: Wire pin state through `WindowController`**

If unsupported, disable the toggle and attach the tooltip `Always on Top is unavailable on this Wayland desktop`.

- [ ] **Step 5: Run tests and manual smoke test**

Run:

```bash
cargo test -p noor-notes
cargo run -p noor-notes
```

Verify create, edit, resize, restart, pin/unpin, archive, and trash.

- [ ] **Step 6: Commit**

```bash
git add apps/noor-notes
git commit -m "feat: add native sticky note windows"
```

### Task 7: Add main window, search, tray actions, and import UI

**Files:**
- Create: `apps/noor-notes/src/main_window.rs`
- Create: `apps/noor-notes/src/search.rs`
- Create: `apps/noor-notes/src/import_dialog.rs`
- Create: `apps/noor-notes/src/actions.rs`
- Modify: `apps/noor-notes/src/app.rs`
- Test: `apps/noor-notes/tests/search.rs`
- Test: `apps/noor-notes/tests/import_flow.rs`

**Interfaces:**
- Produces application actions `app.new-note`, `app.show-notes`, `app.search`, `app.import-xpad`, `app.sync-now`, and `app.quit`.

- [ ] **Step 1: Write failing search and import-preview tests**

Search must be case-insensitive for Latin text and Unicode-safe for Arabic/Bangla. Import must require confirmation and show skipped-file errors.

- [ ] **Step 2: Implement main management window**

Provide Active, Archived, and Trash views, local search, a sync-status area, and settings. Restore from trash and permanent deletion require explicit confirmation.

- [ ] **Step 3: Add application actions and desktop entry**

Create `data/io.github.saamaamr.NoorNotes.desktop` and `data/io.github.saamaamr.NoorNotes.metainfo.xml`. Use AppIndicator only when available; all actions must remain reachable from the main window.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-notes`

```bash
git add apps/noor-notes data
git commit -m "feat: add note management and Xpad import UI"
```

### Task 8: Implement the encrypted vault

**Files:**
- Create: `crates/crypto/src/vault.rs`
- Create: `crates/crypto/src/envelope.rs`
- Create: `crates/crypto/src/recovery.rs`
- Create: `crates/crypto/src/error.rs`
- Modify: `crates/crypto/src/lib.rs`
- Test: `crates/crypto/tests/vectors.rs`

**Interfaces:**
- Produces `Vault::create(passphrase)`, `Vault::unlock(passphrase, wrapped)`, `encrypt_note`, `decrypt_note`, `RecoveryKey::generate`, `Vault::unlock_with_recovery`.

- [ ] **Step 1: Write tamper, wrong-passphrase, and round-trip tests**

Use fixed test vectors. Assert ciphertext does not contain plaintext, wrong keys return `AuthenticationFailed`, and one-bit changes fail authentication.

- [ ] **Step 2: Implement versioned envelopes**

Use Argon2id with random 16-byte salt to wrap a random 32-byte vault key; use XChaCha20-Poly1305 with random 24-byte nonce per note revision. Include envelope version and note/revision identifiers as authenticated associated data. Zeroize temporary key material.

- [ ] **Step 3: Add recovery-key encoding**

Encode the recovery key with checksum and grouped human-readable characters. Require confirmation of randomly selected groups before sync activation.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-crypto`

```bash
git add crates/crypto
git commit -m "feat: add end-to-end encrypted vault"
```

### Task 9: Create Supabase schema and authenticated client

**Files:**
- Create: `supabase/migrations/202608040001_encrypted_notes.sql`
- Create: `supabase/tests/rls.sql`
- Create: `crates/sync/src/client.rs`
- Create: `crates/sync/src/types.rs`
- Test: `crates/sync/tests/client.rs`

**Interfaces:**
- Produces `SupabaseClient::sign_in`, `upload_revision`, `list_changes`, `upload_tombstone`; server rows contain `owner_id`, `note_id`, `revision`, `ciphertext`, `nonce`, `updated_at`, `deleted_at`.

- [ ] **Step 1: Write RLS tests before policies**

Create two users; assert each can CRUD only rows whose `owner_id = auth.uid()` and cannot read the other's ciphertext.

- [ ] **Step 2: Add tables, uniqueness, indexes, and RLS**

Use primary key `(owner_id, note_id, revision)` and an index on `(owner_id, updated_at)`. Enable RLS before granting authenticated access.

- [ ] **Step 3: Write HTTP client contract tests using WireMock**

Cover successful upload, duplicate idempotency, expired token, 429 retry metadata, malformed response, and redacted errors.

- [ ] **Step 4: Implement client and commit**

Run: `cargo test -p noor-sync --test client`

```bash
git add supabase crates/sync
git commit -m "feat: add encrypted Supabase storage client"
```

### Task 10: Implement offline synchronization and conflict copies

**Files:**
- Create: `crates/sync/src/worker.rs`
- Create: `crates/sync/src/merge.rs`
- Create: `crates/sync/src/backoff.rs`
- Modify: `crates/sync/src/lib.rs`
- Test: `crates/sync/tests/offline.rs`
- Test: `crates/sync/tests/conflicts.rs`

**Interfaces:**
- Produces `SyncWorker::run_once`, `SyncWorker::run`, `merge_remote_revision`, and statuses `Idle`, `Syncing`, `Offline`, `AuthRequired`, `Error`.

- [ ] **Step 1: Write failing offline/idempotency tests**

Queue edits while the mock server is unavailable, restore it, run twice, and assert one remote revision and an empty acknowledged journal.

- [ ] **Step 2: Write concurrent-edit and tombstone tests**

Assert concurrent content creates a recoverable `Conflict copy — <device> — <timestamp>` note. Assert a newer tombstone prevents offline resurrection and trash retains it for 30 days.

- [ ] **Step 3: Implement upload/download transaction boundaries**

Decrypt and authenticate remote data before opening the local write transaction. Quarantine invalid envelopes. Apply local note, sync cursor, and journal acknowledgements atomically.

- [ ] **Step 4: Implement bounded exponential backoff**

Use 2s, 4s, 8s, 16s, 30s maximum with jitter; stop automatic retries on authentication failure until re-login.

- [ ] **Step 5: Verify and commit**

Run: `cargo test -p noor-sync`

```bash
git add crates/sync
git commit -m "feat: add offline encrypted synchronization"
```

### Task 11: Add account, recovery, and sync UI

**Files:**
- Create: `apps/noor-notes/src/account.rs`
- Create: `apps/noor-notes/src/vault_setup.rs`
- Create: `apps/noor-notes/src/sync_status.rs`
- Modify: `apps/noor-notes/src/main_window.rs`
- Test: `apps/noor-notes/tests/vault_onboarding.rs`

**Interfaces:**
- Consumes: `Vault`, `SupabaseClient`, `SyncWorker`.
- Produces onboarding states `SignedOut`, `AccountReady`, `RecoveryKeyRequired`, `Ready`.

- [ ] **Step 1: Test that sync cannot enable before recovery confirmation**

Assert closing onboarding before confirmation leaves cloud sync disabled and local notes untouched.

- [ ] **Step 2: Implement login and vault enrollment**

Store refresh tokens and wrapped vault keys in Secret Service/libsecret, never SQLite or logs. Display the recovery key once and require group confirmation.

- [ ] **Step 3: Implement sync status and recovery flows**

Show pending count, last success, Offline, Auth Required, and quarantined-record errors. Keep local editing enabled in every state.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-notes`

```bash
git add apps/noor-notes
git commit -m "feat: add encrypted sync onboarding"
```

### Task 12: Build the GNOME Wayland Always-on-Top adapter

**Files:**
- Create: `extensions/gnome/metadata.json`
- Create: `extensions/gnome/extension.js`
- Create: `extensions/gnome/dbus.xml`
- Create: `extensions/gnome/stylesheet.css`
- Create: `crates/windowing/src/gnome.rs`
- Test: `extensions/gnome/tests/test-policy.js`
- Test: `crates/windowing/tests/gnome_contract.rs`

**Interfaces:**
- Produces D-Bus interface `io.github.saamaamr.NoorNotes.Window1` with `SetAbove(s window_id, b enabled)` and `SetAllWorkspaces(s window_id, b enabled)`.

- [ ] **Step 1: Test request authorization policy**

Accept only windows whose application ID is `io.github.saamaamr.NoorNotes`; reject unknown IDs, stale windows, invalid booleans/signatures, and calls from other bus names.

- [ ] **Step 2: Implement the minimal GNOME extension**

Track Noor Notes windows only, apply compositor stacking/workspace operations, restore normal state on disable, and expose no note content or generic window-control method.

- [ ] **Step 3: Implement Rust D-Bus controller and capability probe**

If the extension is absent or incompatible, select `FallbackWindowController` and show installation guidance.

- [ ] **Step 4: Test on GNOME Wayland and commit**

Run:

```bash
gjs -m extensions/gnome/tests/test-policy.js
cargo test -p noor-windowing --test gnome_contract
```

Manually toggle Above on one of two notes and verify only the selected note changes.

```bash
git add extensions/gnome crates/windowing
git commit -m "feat: add GNOME Wayland window adapter"
```

### Task 13: Package, migrate, and run end-to-end acceptance

**Files:**
- Create: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Create: `packaging/deb/noor-notes.install`
- Create: `scripts/install-gnome-extension.sh`
- Create: `tests/e2e/two_device_sync.sh`
- Create: `README.md`
- Create: `docs/security.md`

**Interfaces:**
- Produces installable application, companion-extension installer, and documented recovery/export paths.

- [ ] **Step 1: Write the two-device acceptance script**

The script creates two isolated data directories, edits offline on both, reconnects, and asserts convergence plus one preserved conflict copy. It must also assert Supabase fixture rows do not contain known plaintext.

- [ ] **Step 2: Build Flatpak and Debian packages**

Request only network, Secret Service, notifications, and required display permissions. Keep GNOME extension installation explicit and separately reviewable.

- [ ] **Step 3: Run full verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
xvfb-run -a cargo test -p noor-windowing
bash tests/e2e/two_device_sync.sh
flatpak-builder --force-clean /tmp/noor-notes-build packaging/flatpak/io.github.saamaamr.NoorNotes.yml
```

Expected: every command exits 0; no test output or logs contain fixture plaintext, tokens, passphrases, or keys.

- [ ] **Step 4: Perform real Xpad migration rehearsal**

Back up `~/.config/xpad`, import through the UI, compare source hashes, verify content/geometry/style, restart, and confirm no duplicate import.

- [ ] **Step 5: Document install, recovery, limitations, and commit**

README must explain X11 vs Wayland behavior, Supabase setup, recovery-key responsibility, Xpad import, backup/export, and GNOME companion installation.

```bash
git add packaging scripts tests/e2e README.md docs/security.md
git commit -m "release: package Noor Notes first release"
```

---

## Execution Checkpoints

1. After Task 4: review domain, database integrity, and Xpad import before UI work.
2. After Task 7: run and review the complete offline X11 application.
3. After Task 10: security review encryption and sync conflict behavior before connecting the UI.
4. After Task 12: review GNOME extension permissions and window scoping.
5. After Task 13: run final acceptance on two Linux user profiles before release.
