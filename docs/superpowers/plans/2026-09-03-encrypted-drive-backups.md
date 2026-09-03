# Encrypted Drive Backup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real optional encrypted backups to Google Drive App Data and OneDrive App Folder without turning either provider into a synchronization source.

**Architecture:** A provider-neutral archive builder serializes a bounded snapshot of all local note states and encrypts the complete payload with the unlocked Noor sync vault. Separate public-client OAuth PKCE adapters request only each provider's app-folder scope, store refresh tokens in GNOME Keyring, and atomically replace a single current backup plus timestamped recovery archives. Restore downloads, authenticates, previews metadata, and imports through repository APIs after explicit confirmation.

**Tech Stack:** Rust, XChaCha20-Poly1305, serde, reqwest, OAuth 2.0 PKCE, Google Drive REST, Microsoft Identity Platform/Graph, GNOME Keyring, GTK4/libadwaita, Wiremock

**Spec:** `docs/superpowers/specs/2026-09-03-cloud-account-and-backup-design.md`

## Global Constraints

- Backup providers receive no plaintext note content, titles, tags, rich formatting, or vault key.
- Google uses only `drive.appdata`; OneDrive uses only `Files.ReadWrite.AppFolder offline_access`.
- No OAuth client secret is embedded because Noor Notes is a public desktop client.
- Provider failure never affects local save, Supabase sync, or another provider.
- Restore never replaces the SQLCipher database file in place and always needs explicit confirmation.
- Provider setup is hidden or disabled when reviewed public client configuration is absent.

---

### Task 1: Authenticated encrypted archive format

**Files:**
- Modify: `crates/crypto/src/envelope.rs`
- Modify: `crates/crypto/src/vault.rs`
- Create: `crates/sync/src/backup_archive.rs`
- Modify: `crates/sync/src/lib.rs`
- Create: `crates/sync/tests/backup_archive.rs`

**Interfaces:**
- Produce `EncryptedBackup { version, created_at, nonce, ciphertext }`.
- Produce `BackupPreview { created_at, note_count, device_id }`.
- Produce `BackupArchive::{create, preview, decrypt}`; the preview is returned only after AEAD authentication.
- Payload contains all `Note` values from `search_notes_sorted("", UpdatedDesc)` and is capped at 128 MiB encrypted.

- [ ] Write tests proving archive bytes omit known title/body text, tampering and wrong vault fail, metadata changes fail authentication, archived/trash notes round-trip, and oversized input is rejected before allocation growth.
- [ ] Run `cargo test -p noor-sync --test backup_archive`; verify RED on missing archive APIs.
- [ ] Add vault backup encryption with domain-separated AAD and implement deterministic versioned JSON serialization plus strict size/count limits.
- [ ] Re-run crypto vectors and archive tests with strict noor-sync clippy.
- [ ] Commit as `feat(backup): add authenticated encrypted archives`.

### Task 2: Public-client provider OAuth

**Files:**
- Create: `crates/sync/src/provider_oauth.rs`
- Modify: `crates/sync/src/lib.rs`
- Create: `crates/sync/tests/provider_oauth.rs`
- Modify: `apps/noor-notes/src/oauth_callback.rs`

**Interfaces:**
- Produce `ProviderOAuth::{google, onedrive, authorization, exchange, refresh, revoke}`.
- Google callback is `http://127.0.0.1:43818/backup/google` and exact scope is `https://www.googleapis.com/auth/drive.appdata`.
- OneDrive callback is `http://127.0.0.1:43819/backup/onedrive` and exact scopes are `offline_access Files.ReadWrite.AppFolder`.
- Reuse a parameterized one-shot bounded callback listener with independent random state and S256 verifier.

- [ ] Write Wiremock/query tests proving exact provider endpoints, scopes, PKCE, refresh exchange, redacted errors, state mismatch rejection, and absence of client secrets.
- [ ] Run provider OAuth and callback tests; verify RED on missing adapters/parameterized callback.
- [ ] Implement explicit provider configurations and bounded token responses; access tokens remain memory-only and refresh tokens are zeroized.
- [ ] Re-run focused tests and strict clippy.
- [ ] Commit as `feat(backup): authorize app-folder providers`.

### Task 3: Google Drive and OneDrive storage adapters

**Files:**
- Create: `crates/sync/src/backup_provider.rs`
- Create: `crates/sync/src/google_drive.rs`
- Create: `crates/sync/src/onedrive.rs`
- Modify: `crates/sync/src/lib.rs`
- Create: `crates/sync/tests/backup_providers.rs`

**Interfaces:**
- Produce async `BackupProvider::{upload, list, download, delete}` and `BackupObject { id, name, modified_at, size }`.
- Google targets `appDataFolder`, discovers by exact appProperties/name, uploads temporary content, then updates the current object.
- OneDrive targets `/me/drive/special/approot:/Noor Notes/<name>:/content` with conflict behavior replace.
- Reject redirects to foreign origins, objects over 128 MiB, unsafe names, malformed lists, and non-TLS production endpoints.

- [ ] Write provider-contract tests against Wiremock for exact Google/Graph paths, auth headers, atomic/current naming, pagination bounds, download limits, and provider-specific failures.
- [ ] Run `cargo test -p noor-sync --test backup_providers`; verify RED on missing adapters.
- [ ] Implement the narrow trait and two REST adapters using existing reqwest; add no SDK dependency.
- [ ] Re-run all noor-sync tests and strict clippy.
- [ ] Commit as `feat(backup): add Drive storage adapters`.

### Task 4: Backup controller and secure provider sessions

**Files:**
- Create: `apps/noor-notes/src/cloud_backup.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Modify: `apps/noor-notes/src/key_store.rs`
- Create: `apps/noor-notes/tests/cloud_backup.rs`

**Interfaces:**
- Add `SecretKind::{GoogleDriveSession, OneDriveSession}`.
- Produce `CloudBackupController::{connect, restore_connections, backup_now, list_backups, preview_restore, restore, disconnect}`.
- `restore` saves each authenticated note through `save_remote_note`, preserves newer/conflicting local content through the existing merge policy, and never swaps database files.

- [ ] Write tests proving each provider token is isolated, one provider failure does not cancel another, archive plaintext never reaches HTTP, disconnect removes only its provider token, and restore requires a previously authenticated preview token.
- [ ] Run `cargo test -p noor-notes --test cloud_backup`; verify RED on missing controller.
- [ ] Implement configuration loading from `NOOR_GOOGLE_DRIVE_CLIENT_ID` and `NOOR_ONEDRIVE_CLIENT_ID`, session refresh, archive upload, preview nonce, confirmed restore, and per-provider result reporting.
- [ ] Re-run cloud backup, key-store, storage lifecycle, and sync conflict tests with strict app clippy.
- [ ] Commit as `feat(backup): coordinate encrypted provider backups`.

### Task 5: Backup settings UI

**Files:**
- Modify: `apps/noor-notes/src/ui/account_settings.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Create: `apps/noor-notes/tests/cloud_backup_ui.rs`

**Interfaces:**
- Account & Sync shows independent Google Drive and OneDrive connection state, Backup Now, last result, available encrypted archives, Preview, confirmed Restore, and Disconnect.
- Controls require a signed-in/unlocked Noor vault; provider authorization is separate from Google account sign-in.

- [ ] Write GTK tests proving real buttons invoke injected controllers, narrow windows remain usable, unavailable configuration is explicit, destructive restore confirmation is required, and provider errors remain isolated.
- [ ] Run the GTK test and verify RED on missing backup controls.
- [ ] Implement responsive rows/dialogs with asynchronous operations and accessible labels; do not display or log tokens.
- [ ] Re-run account/sync/backup UI tests under the available display.
- [ ] Commit as `feat(ui): add encrypted backup controls`.

### Task 6: Provider documentation and complete verification

**Files:**
- Modify: `README.md`
- Modify: `docs/security.md`
- Modify: `snapcraft.yaml`
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Modify: relevant manifest tests

**Interfaces:**
- Document exact redirect URIs, least-privilege scopes, public client IDs, archive location, recovery/restore behavior, and configuration status without credentials.

- [ ] Update docs only after both providers work against contract tests; distinguish implemented code from live production configuration.
- [ ] Run formatting, strict workspace clippy, workspace tests, RLS tests, manifest tests, `git diff --check`, and `git status --short`.
- [ ] Rebuild/install Noor Notes Dev and verify its version and desktop launch contract.
- [ ] Commit as `docs: document encrypted Drive backups`.

