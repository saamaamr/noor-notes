# Security Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Encrypt local notes at rest and harden secrets, networking, imports, exports, desktop integration, packaging, and release verification without losing existing data.

**Architecture:** A `KeyStore` bootstrap obtains an independent random database key before `EncryptedSqliteRepository` opens SQLCipher. A transactional, idempotent migrator converts existing plaintext databases and fails closed. Boundary-specific validators protect network, import/export, D-Bus, and package surfaces.

**Tech Stack:** Rust 1.85+, GTK4/libadwaita, Tokio, SQLx 0.8.6, libsqlite3-sys 0.30.1 with `bundled-sqlcipher-vendored-openssl`, oo7 0.6, XChaCha20-Poly1305, Rustls, Snap, Flatpak.

## Global Constraints

- Preserve the database path, application ID, note schema, visible note behavior, and every existing note, revision, tag, style, geometry value, and journal entry.
- Never fall back to plaintext storage when keyring, encryption, verification, or migration fails.
- Never derive data encryption from the desktop four-digit lock password.
- Keep local database encryption and cloud envelope encryption cryptographically independent.
- Never log note content, credentials, tokens, passphrases, database keys, or decrypted key material.
- Keep the untracked release artifact `noor-notes_0.1.0_amd64.snap` untouched.
- Use TDD for every behavior change and commit only after its focused tests pass.

---

### Task 1: Typed secret storage and zeroizing database keys

**Files:**
- Modify: `Cargo.toml`
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/key_store.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Replace: `apps/noor-notes/src/account.rs`
- Test: `apps/noor-notes/tests/key_store.rs`

**Interfaces:**
- Produce `#[async_trait] pub trait KeyStore: Send + Sync { async fn get(&self, kind: SecretKind, account: &str) -> Result<Option<Zeroizing<Vec<u8>>>, KeyStoreError>; async fn put(&self, kind: SecretKind, account: &str, value: &[u8]) -> Result<(), KeyStoreError>; async fn delete(&self, kind: SecretKind, account: &str) -> Result<(), KeyStoreError>; }`.
- Produce `pub enum SecretKind { DatabaseKey, RefreshToken, WrappedVault }` and `pub struct Oo7KeyStore` backed by `oo7::Keyring` with application, kind, and account attributes.
- Consume `KeyStore` from account and bootstrap code; no caller invokes `secret-tool`.

- [ ] **Step 1: Write failing key-store tests**

Use an in-memory fake to assert separate attributes cannot collide, duplicate matches are rejected, missing items return `None`, delete removes the exact item, and formatted errors contain neither the secret nor account password.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p noor-notes --test key_store`

- [ ] **Step 3: Add dependencies and implementation**

Add `oo7 = { version = "0.6", default-features = false, features = ["tokio", "native_crypto"] }` and expose the exact interfaces above. Replace subprocess storage in `account.rs`; store only refresh tokens and wrapped vaults, and add sign-out deletion.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-notes --test key_store --test vault_onboarding && cargo clippy -p noor-notes --all-targets -- -D warnings`

Commit: `security: use typed secret service storage`

### Task 2: SQLCipher repository and file-permission policy

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/encrypted_open.rs`
- Create: `crates/storage/src/permissions.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/repository.rs`
- Modify: `crates/storage/src/backup.rs`
- Test: `crates/storage/tests/encrypted_repository.rs`
- Test: `crates/storage/tests/permissions.rs`

**Interfaces:**
- Produce `pub struct DatabaseKey(Zeroizing<[u8; 32]>)` with `generate()`, `try_from_slice(&[u8])`, and redacted `Debug`.
- Change `SqliteNoteRepository::open(path)` to `SqliteNoteRepository::open_encrypted(path: &Path, key: &DatabaseKey) -> Result<Self, StorageError>`.
- Produce `secure_data_tree(path: &Path) -> Result<(), StorageError>` enforcing directory `0700` and database/WAL/SHM/backup files `0600`.

- [ ] **Step 1: Write failing ciphertext and permission tests**

Save a note containing marker `NOOR-PLAINTEXT-SENTINEL`; assert raw database, WAL, and SHM bytes do not contain the marker or `SQLite format 3`; assert reopening with the correct key succeeds, a wrong key fails, tampering fails, and every data artifact has the required Unix mode.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p noor-storage --test encrypted_repository --test permissions`

- [ ] **Step 3: Link SQLCipher and implement encrypted open**

Add direct `libsqlite3-sys = { version = "0.30.1", features = ["bundled-sqlcipher-vendored-openssl"] }` so SQLx shares the SQLCipher build. Apply a hex key using `PRAGMA key = "x..."` through a connection setup hook before all other SQL; set `cipher_memory_security = ON`, foreign keys, WAL, and a busy timeout; verify `PRAGMA cipher_integrity_check` before migrations.

- [ ] **Step 4: Apply permissions to all artifacts**

Secure the parent before creation and again after WAL initialization and backup creation. Reject symlink database paths and non-regular existing database files.

- [ ] **Step 5: Verify and commit**

Run: `cargo test -p noor-storage --test encrypted_repository --test permissions && cargo clippy -p noor-storage --all-targets -- -D warnings`

Commit: `security: encrypt local repository with SQLCipher`

### Task 3: Fail-safe plaintext-to-encrypted migration

**Files:**
- Create: `crates/storage/src/migration.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/error.rs`
- Test: `crates/storage/tests/encryption_migration.rs`

**Interfaces:**
- Produce `pub async fn migrate_or_open(path: &Path, key: &DatabaseKey) -> Result<SqliteNoteRepository, StorageError>`.
- Produce internal `DatabaseFormat::{Missing, Plaintext, Encrypted}` detected from bounded header reads without opening unknown data.
- Produce migration checkpoints through an injectable `MigrationHooks` test interface so interruption is tested after copy, verification, fsync, and rename.

- [ ] **Step 1: Write failing migration tests**

Build a legacy plaintext fixture containing active, archived, and trashed notes plus styles, geometry, and pending/acknowledged journal rows. Assert exact semantic equality after migration, ciphertext at rest, idempotent second open, untouched plaintext authority on every injected failure, and no plaintext temporary file after success.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p noor-storage --test encryption_migration`

- [ ] **Step 3: Implement transactional migration**

Checkpoint the legacy WAL, create `notes.db.encrypted-new` with `0600`, copy tables inside one transaction, verify SQLCipher integrity, row counts, note deserialization, revision totals, and foreign keys, fsync the new file and parent, rename the original to a unique migration guard, atomically place the encrypted file, fsync again, then unlink the guard. Restore the guard on any post-rename failure.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-storage --test encryption_migration --test corruption --test repository --test lifecycle --test metadata`

Commit: `security: migrate existing databases without data loss`

### Task 4: Secure application bootstrap and recovery UI

**Files:**
- Create: `apps/noor-notes/src/security_bootstrap.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Modify: `apps/noor-notes/resources/modern.css`
- Test: `apps/noor-notes/tests/security_bootstrap.rs`

**Interfaces:**
- Produce `pub async fn open_repository(path: &Path, keys: Arc<dyn KeyStore>) -> Result<SqliteNoteRepository, BootstrapError>`.
- Database-key account is the fixed value `local-default`; missing key on a missing/plaintext database generates 32 CSPRNG bytes and stores them before migration; missing key on an encrypted database returns `BootstrapError::DatabaseKeyMissing`.

- [ ] **Step 1: Write failing bootstrap tests**

Assert first run creates exactly one key, restart reuses it, encrypted database plus missing key fails closed, locked/unavailable keyring does not create plaintext files, and all errors have safe user-facing messages.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p noor-notes --test security_bootstrap`

- [ ] **Step 3: Wire bootstrap before GTK repository use**

Replace direct `SqliteNoteRepository::open` in `managed_app.rs`. Present a blocking error window for unavailable/missing keys or migration failure with Retry and Quit only; never launch note windows with an invalid repository.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-notes --test security_bootstrap --test vault_onboarding`

Commit: `security: fail closed during secure startup`

### Task 5: Network and remote-record validation

**Files:**
- Modify: `crates/sync/src/client.rs`
- Modify: `crates/sync/src/types.rs`
- Modify: `crates/sync/src/remote_worker.rs`
- Modify: `crates/sync/src/merge.rs`
- Test: `crates/sync/tests/client.rs`
- Create: `crates/sync/tests/security_policy.rs`

**Interfaces:**
- Add `EndpointPolicy::Production` and `EndpointPolicy::AllowLoopbackHttpForTests`; change constructor to `SupabaseClient::new(base_url, anon_key, policy)`.
- Add constants: 10-second connect timeout, 30-second request timeout, 1 MiB maximum response body, 4 MiB maximum ciphertext, and 500 revisions per response.
- Produce `RemoteRevision::validate(&self) -> Result<(), SyncClientError>` checking version, UUID, nonce length, ciphertext size, timestamp range, and monotonic revision.

- [ ] **Step 1: Write failing boundary tests**

Test rejection of production HTTP, credentials in URLs, redirects, oversized bodies, excessive revision arrays, wrong nonce sizes, future timestamps beyond five minutes, revision downgrade/replay, and malformed records without local replacement.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p noor-sync --test security_policy --test client --test remote_download`

- [ ] **Step 3: Implement hardened client and merge policy**

Build Reqwest with Rustls, no redirects, bounded timeouts, and bounded byte reads before JSON parsing. Preserve redacted errors and existing bounded backoff. Validate before decrypting or persisting.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-sync && cargo clippy -p noor-sync --all-targets -- -D warnings`

Commit: `security: validate sync endpoints and remote data`

### Task 6: Import and export boundary hardening

**Files:**
- Modify: `crates/xpad-import/src/parser.rs`
- Modify: `crates/xpad-import/src/error.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Create: `apps/noor-notes/src/safe_export.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Test: `crates/xpad-import/tests/security.rs`
- Create: `apps/noor-notes/tests/safe_export.rs`

**Interfaces:**
- Import limits: 10,000 info files, 64 KiB per info file, 16 MiB per content file, 64 MiB total preview bytes, width/height 100..8192, and coordinates -32768..32767.
- Produce `sanitize_export_name(title: &str, extension: ExportExtension) -> String` capped at 120 Unicode scalar values and excluding separators/control characters.
- Produce `set_owner_only(path: &Path)` for non-portal local exports; UI labels exports as unencrypted.

- [ ] **Step 1: Write failing malicious-input tests**

Test symlink info/content files, FIFO/special files, traversal, oversized files/counts/total, invalid UTF-8, extreme geometry, control-character filenames, and local export mode `0600`.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p noor-xpad-import --test security && cargo test -p noor-notes --test safe_export`

- [ ] **Step 3: Implement bounded reads and safe exports**

Use `symlink_metadata`, require regular files, canonicalize root and candidates, verify containment, inspect sizes before bounded `take(limit + 1)` reads, and stop once aggregate limits are reached. Sanitize suggested names and explain that exported files are plaintext.

- [ ] **Step 4: Verify and commit**

Run: `cargo test -p noor-xpad-import && cargo test -p noor-notes --test import_flow --test export --test safe_export`

Commit: `security: harden import and export boundaries`

### Task 7: D-Bus and sandbox permission policy

**Files:**
- Modify: `extensions/gnome/policy.js`
- Modify: `extensions/gnome/extension.js`
- Modify: `extensions/gnome/tests/test-policy.js`
- Modify: `snap/snapcraft.yaml`
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Modify: `tests/snap_manifest.sh`
- Modify: `tests/flatpak_manifest.sh`

**Interfaces:**
- Policy accepts only the two existing methods, exact application bus owner, exact GTK application ID, valid live window identifiers, and boolean values.
- Manifest tests deny home/host filesystem, devices, process control, system bus, broad D-Bus names, and classic confinement.

- [ ] **Step 1: Add failing authorization and manifest tests**

Cover spoofed titles, stale windows, wrong sender, malformed X11 IDs, unexpected method names, non-boolean inputs, and forbidden permission strings.

- [ ] **Step 2: Verify tests fail**

Run: `gjs -m extensions/gnome/tests/test-policy.js && bash tests/snap_manifest.sh && bash tests/flatpak_manifest.sh`

- [ ] **Step 3: Harden policy and retain minimal manifests**

Validate every boundary before mutating a window. Keep Snap strict confinement and only desktop, display, network, password manager, and the exact session D-Bus slot. Keep Flatpak permissions limited to display, network, and Secret Service.

- [ ] **Step 4: Verify and commit**

Run: `gjs -m extensions/gnome/tests/test-policy.js && bash tests/snap_manifest.sh && bash tests/flatpak_manifest.sh`

Commit: `security: enforce desktop integration boundaries`

### Task 8: Supply-chain policy, documentation, and release gate

**Files:**
- Create: `deny.toml`
- Create: `.github/workflows/security.yml`
- Create: `scripts/security-check.sh`
- Modify: `docs/security.md`
- Modify: `README.md`
- Modify: `snap/snapcraft.yaml`
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Test: `tests/release_workflow.sh`
- Test: `tests/workspace_smoke.sh`

**Interfaces:**
- `scripts/security-check.sh` runs formatting, strict Clippy, workspace tests, `cargo audit`, `cargo deny check`, secret-pattern scanning, package manifest tests, and SBOM generation.
- CI uploads a CycloneDX/SPDX-compatible SBOM artifact and fails on unacknowledged vulnerabilities, yanked crates, forbidden licenses/sources, or leaked secret patterns.

- [ ] **Step 1: Write failing release-policy tests**

Assert locked builds, pinned/checksummed package inputs, invocation of audit/deny/secret scan/SBOM steps, and documentation of local-encryption recovery limitations.

- [ ] **Step 2: Verify tests fail**

Run: `bash tests/release_workflow.sh && bash tests/workspace_smoke.sh`

- [ ] **Step 3: Add policy and update documentation**

Allow GPL-3.0-or-later-compatible dependency licenses explicitly, deny unknown registries/git sources, and require documented comments for any advisory exception. Update the security model to distinguish encrypted local DB, encrypted cloud envelopes, plaintext exports, and the unrecoverable missing-key case.

- [ ] **Step 4: Run complete release verification**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && bash scripts/security-check.sh && bash tests/install_ubuntu.sh && git diff --check`

- [ ] **Step 5: Install and smoke-test**

Run: `PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh`; close the old process, reopen Noor Notes, verify existing notes, create/edit/close/reopen a test note, and confirm the raw database lacks the test marker.

- [ ] **Step 6: Commit**

Commit: `security: add release security gate and documentation`
