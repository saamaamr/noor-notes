# Encrypted Supabase Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing Noor account and encryption primitives into safe bidirectional Supabase note synchronization.

**Architecture:** SQLite remains authoritative locally. A per-account vault is wrapped by a user passphrase and recovery key, stored as ciphertext in Supabase and GNOME Keyring, while the existing change journal uploads encrypted revisions. A durable per-account cursor drives ordered downloads; GTK only invokes an asynchronous controller and refreshes after committed remote changes.

**Tech Stack:** Rust, SQLCipher/SQLx, XChaCha20-Poly1305, Argon2id, Supabase Auth/PostgREST/RLS, GNOME Keyring, GTK4/libadwaita, Wiremock

**Spec:** `docs/superpowers/specs/2026-09-03-cloud-account-and-backup-design.md`

## Global Constraints

- SQLite remains the local source of truth.
- Supabase receives ciphertext and authenticated metadata only.
- No access token, refresh token, passphrase, recovery key, or plaintext note is logged.
- Cloud work stays asynchronous and never blocks GTK.
- Existing local notes, autosave, editor modes, themes, and persistence format remain intact.
- New UI uses `MainWindow -> NotePreview`, never legacy `NoteWindow`.

---

### Task 1: Remote encrypted vault contract

**Files:**
- Modify: `crates/crypto/src/recovery.rs`
- Modify: `crates/sync/src/types.rs`
- Modify: `crates/sync/src/client.rs`
- Modify: `crates/sync/src/lib.rs`
- Create: `crates/sync/tests/vault_client.rs`
- Create: `supabase/migrations/202609030001_encrypted_vaults.sql`
- Modify: `supabase/tests/rls.sql`

**Interfaces:**
- Produce `RecoveryKey::decode(&str) -> Result<RecoveryKey, CryptoError>` with checksum validation.
- Produce `RemoteVault { wrapped_vault, recovery_wrapped_vault, updated_at }`.
- Produce `SupabaseClient::{get_vault, put_vault}` using `/rest/v1/encrypted_vaults`.

- [ ] Write tests proving malformed recovery text fails, a valid encoded key round-trips, vault JSON has no passphrase/key plaintext, and authenticated GET/UPSERT use the exact owner-scoped table.
- [ ] Run `cargo test -p noor-crypto --test vectors && cargo test -p noor-sync --test vault_client`; verify failures are caused by the missing APIs.
- [ ] Implement strict Base32/checksum decoding, bounded vault responses, idempotent PostgREST upsert, and an owner-only RLS table keyed by `auth.uid()`.
- [ ] Re-run the focused tests plus `bash tests/supabase_rls.sh`; require zero failures.
- [ ] Commit as `feat(sync): store encrypted account vaults`.

### Task 2: Ordered bidirectional sync cycle

**Files:**
- Modify: `crates/sync/src/client.rs`
- Modify: `crates/sync/src/worker.rs`
- Modify: `crates/sync/src/remote_worker.rs`
- Modify: `crates/sync/src/types.rs`
- Create: `crates/sync/tests/bidirectional.rs`

**Interfaces:**
- Produce `SyncCursor { updated_at, note_id, revision }` with a deterministic epoch default.
- Produce `SyncCycle { status, cursor, uploaded, downloaded }`.
- Produce `SyncWorker::run_cycle(cursor, device_id, now) -> SyncCycle`.
- Extend `list_changes` to order and filter by timestamp plus stable note/revision tie-breakers.

- [ ] Write tests proving upload acknowledgement happens only after HTTP success, remote revisions apply before cursor advancement, failed decrypt/storage leaves the cursor unchanged, repeated cycles create no duplicate conflict copy, and auth/offline failures retain pending work.
- [ ] Run `cargo test -p noor-sync --test bidirectional`; verify RED against the upload-only worker.
- [ ] Implement one bounded page per cycle, deterministic ordering, apply-then-advance semantics, and an applied remote identity journal so equal-timestamp replay is idempotent.
- [ ] Run every `noor-sync` test with strict clippy; require zero failures/warnings.
- [ ] Commit as `feat(sync): run encrypted bidirectional cycles`.

### Task 3: Secure application sync runtime

**Files:**
- Create: `apps/noor-notes/src/cloud_sync.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Modify: `apps/noor-notes/src/key_store.rs`
- Modify: `apps/noor-notes/src/account.rs`
- Create: `apps/noor-notes/tests/cloud_sync.rs`

**Interfaces:**
- Add `SecretKind::{SyncVault, SyncCursor}`.
- Produce `CloudSyncController::{enroll, unlock_with_passphrase, unlock_with_recovery, restore, run_once, disable}`.
- Store wrapped vault and cursor under the stable Supabase user ID; keep the unlocked `Vault` and access token memory-only.
- Return `CloudSyncState::{SignedOut, EnrollmentRequired, Locked, Ready, Running, Offline, AuthRequired, Error}`.

- [ ] Write tests proving recovery confirmation gates enrollment, restart restores only wrapped material/cursor, wrong-account material fails closed, sign-out drops memory state without deleting local notes, and `run_once` refreshes expired sessions before worker retry.
- [ ] Run `cargo test -p noor-notes --test cloud_sync`; verify missing controller APIs cause RED.
- [ ] Implement the controller with injected `KeyStore`, `SqliteNoteRepository`, and account client; zeroize passphrases and serialized secret buffers.
- [ ] Re-run account, key-store, vault-onboarding, and cloud-sync tests with strict clippy.
- [ ] Commit as `feat(account): coordinate encrypted sync sessions`.

### Task 4: Enrollment, recovery, and Sync Now UI

**Files:**
- Modify: `apps/noor-notes/src/ui/account_settings.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/sync_status.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/tests/account_settings_ui.rs`
- Create: `apps/noor-notes/tests/cloud_sync_ui.rs`

**Interfaces:**
- Signed-in users can create a vault passphrase, copy and confirm the one-time recovery key, unlock an existing vault by passphrase/recovery, run Sync Now, inspect pending/last result, and sign out.
- `app.sync-now` invokes the shared `CloudSyncController`, updates status, and refreshes `MainWindow` only after remote commits.

- [ ] Write GTK tests for every account/sync state, hidden recovery value after confirmation, compact layout, real button activation, and local-only behavior when configuration is absent.
- [ ] Run the two GTK tests and verify RED on missing enrollment controls/runtime wiring.
- [ ] Implement responsive Libadwaita rows and non-blocking action handlers; never put recovery text in logs or status history.
- [ ] Run focused GTK tests under the available display and distinguish environment failures from app failures.
- [ ] Commit as `feat(ui): add encrypted sync enrollment and status`.

### Task 5: Documentation and release gates

**Files:**
- Modify: `README.md`
- Modify: `docs/security.md`
- Modify: `tests/snap_manifest.sh`
- Modify: `tests/flatpak_manifest.sh`

**Interfaces:**
- Document setup, recovery responsibility, encrypted fields, offline behavior, callback, and package permissions without embedding credentials.

- [ ] Update human documentation only after the behavior exists; do not claim production availability without live project configuration.
- [ ] Run `cargo fmt --all -- --check`, strict workspace clippy, workspace tests, Supabase RLS tests, package manifest tests, `git diff --check`, and `git status --short`.
- [ ] Rebuild/install Noor Notes Dev and verify `noor-notes-dev --version`.
- [ ] Commit as `docs: document encrypted account sync`.

