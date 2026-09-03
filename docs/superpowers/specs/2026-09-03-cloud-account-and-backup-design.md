# Cloud Account and Encrypted Backup Design

## Purpose

Noor Notes will add an optional account system without weakening its offline-first behavior. A user can create or access a Noor account with email and password, or continue with Google. Signed-in users can synchronize end-to-end encrypted notes through managed Supabase. Google Drive and OneDrive are separate, optional encrypted backup destinations rather than competing sources of truth.

## Scope and delivery order

The work is split into independently testable releases:

1. Supabase account foundation: email sign-up, email sign-in, Google OAuth sign-in, secure session refresh, sign-out, and account status UI.
2. Supabase encrypted synchronization: vault enrollment, recovery-key confirmation, queued upload/download, retry, and conflict-copy integration with the current `MainWindow -> NotePreview` path.
3. Encrypted backup providers: manual and scheduled backup through Google Drive App Data and OneDrive App Folder.

Each phase must leave local-only Noor Notes fully usable. A later phase must not be advertised or exposed as an enabled action before its real provider flow is complete.

## User experience

The main menu gains an `Account & Sync…` action. Its Libadwaita preferences window shows one of four states:

- Cloud service unavailable: the production Supabase public configuration is absent or invalid; local notes remain available.
- Signed out: email/password sign-up and sign-in plus `Continue with Google` are available.
- Signed in, sync not enrolled: the verified account identity is shown and the user can create or recover an encrypted vault.
- Sync ready: account identity, last sync result, pending-change count, `Sync Now`, backup-provider controls, and `Sign Out` are available.

Google authentication opens the system browser. Noor Notes listens only on loopback for one short-lived callback, verifies a cryptographically random OAuth state value, and uses PKCE with SHA-256. A login attempt expires after five minutes and a second simultaneous attempt is rejected. Closing or cancelling the flow does not affect local data.

Email sign-up explains when address confirmation is required. Authentication errors use actionable, non-sensitive messages and never print passwords, authorization codes, or tokens.

## Cloud configuration

Managed Supabase hosts Auth, PostgreSQL, and HTTPS APIs. Noor Notes does not require a self-managed server.

The Supabase project URL and publishable key are public application configuration, not user secrets. Development builds may receive them from `NOOR_SUPABASE_URL` and `NOOR_SUPABASE_PUBLISHABLE_KEY`. Release packages embed the same reviewed public values at build time. The application must reject non-HTTPS production endpoints and must not ship a `service_role` key.

Google must be enabled as a Supabase Auth provider. The Supabase redirect allow-list must contain the exact Noor Notes loopback callback used by the packaged application. Google Drive access is requested later and separately so signing in does not grant file access.

Microsoft OneDrive uses its own Microsoft Entra public-client registration. It is not required for Noor account authentication.

## Authentication and secret storage

`noor-sync` owns protocol-level Supabase operations:

- email sign-up;
- password sign-in;
- Google OAuth authorization URL creation;
- PKCE authorization-code exchange;
- refresh-token exchange;
- authenticated user lookup;
- remote encrypted-revision access.

The application account controller owns session lifecycle and keyring persistence. Refresh tokens, wrapped vault material, Google Drive provider tokens, and OneDrive provider tokens are stored as distinct typed entries in GNOME Keyring. Access tokens remain memory-only. Sign-out revokes the Supabase refresh token when reachable and removes only cloud-account secrets; it never deletes the local database, local encryption key, notes, or independent backup files.

Noor Notes treats a Google-authenticated Supabase session and an email/password Supabase session identically after authentication. Supabase's stable user identifier owns remote rows; an email address is display metadata and is not used as a database authorization boundary.

## Encryption and synchronization

SQLite remains the local source of truth. Note content, titles, tags, rich formatting, and sensitive metadata are serialized and encrypted locally with the existing vault before upload. Supabase receives ciphertext, nonce, note identifier, revision, timestamps, deletion marker, and the authenticated owner identifier enforced by row-level security.

The existing change journal drives upload. Remote changes are validated, authenticated, decrypted, and committed locally before UI refresh. Network or authentication failure retains pending changes. Concurrent content edits preserve a conflict copy. Sync work runs asynchronously and never blocks GTK's UI thread.

Vault enrollment displays the recovery key once and requires confirmation before synchronization becomes active. A second device must unlock the same wrapped vault with the user's passphrase or recovery key. Losing both makes cloud ciphertext unrecoverable; the UI states this before enrollment.

## Encrypted backup providers

Backups reuse one provider-neutral encrypted archive format. The archive contains a format version, creation timestamp, device identifier, encrypted note records, and authenticated metadata. Plain note text is never uploaded.

Provider implementations have one narrow interface: connect, report connection state, upload an archive atomically, list known archives, download a selected archive, and disconnect.

- Google Drive uses the hidden `appDataFolder` with the least-privilege `drive.appdata` scope.
- OneDrive uses `Apps/Noor Notes` with `Files.ReadWrite.AppFolder` delegated permission.

A user may connect either or both backup providers. Each backup target receives the same encrypted archive independently. Provider failure is reported per destination and cannot roll back a successful local save or another provider's successful backup. Restore always previews archive metadata, validates authentication tags, and requires explicit user confirmation before importing; it never replaces the existing database in place.

## Failure and privacy behavior

- Local editing, autosave, search, archive, Trash, themes, and editor modes work while signed out or offline.
- Invalid, expired, oversized, or redirected cloud responses fail closed.
- OAuth callbacks are accepted only from loopback, only for the active state, and only once.
- Secrets and note plaintext are absent from logs and error strings.
- A paused or unavailable Supabase project results in an offline status, not data deletion.
- Drive files changed or deleted outside Noor Notes produce a clear backup error; they do not mutate local notes automatically.
- Account deletion is outside this implementation until a separate destructive-flow design is approved.

## Packaging

Snap and Flatpak retain outbound `network` access. The loopback OAuth receiver adds only the minimum inbound loopback permission needed by the sandbox. The system browser is launched through GTK/GIO portal-compatible APIs. No bundled browser, analytics SDK, background server daemon, or heavyweight provider SDK is added.

## Testing and release gates

Automated coverage must include:

- email sign-up success, confirmation-required response, malformed response, and redacted errors;
- email sign-in, token refresh, user lookup, revoke/sign-out, and keyring cleanup;
- Google OAuth PKCE challenge generation, state validation, callback timeout, replay rejection, and code exchange;
- account UI signed-out, busy, error, signed-in, enrollment, offline, and compact-window states;
- ciphertext-only Supabase upload and row-level owner isolation;
- Google Drive and OneDrive least-privilege request paths, token refresh, atomic upload, provider-specific failure, and encrypted restore validation;
- existing autosave, persistence, editor, theme, and lifecycle regressions;
- Snap/Flatpak manifest permission contracts.

Before a public package is released, the real managed Supabase project, Google provider, callback allow-list, Google Drive OAuth consent, and Microsoft Entra registration must be configured and tested with non-production sample notes. Source version, package version, artifact version, Git commit, architecture, Store revision, and requested channel remain separately verified.

## Explicit non-goals

- No plaintext cloud storage.
- No simultaneous Supabase/Drive/OneDrive multi-master note synchronization.
- No collaboration or shared notebooks.
- No web editor or mobile client.
- No automatic account deletion or destructive cloud purge.
- No dependency on the legacy standalone `NoteWindow` for new account or sync behavior.
