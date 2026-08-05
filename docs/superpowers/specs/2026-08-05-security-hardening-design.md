# Noor Notes Security Hardening Design

**Date:** 2026-08-05

## Goal

Protect note content, credentials, synchronization traffic, imported data, exported files, and desktop integration without losing existing notes or requiring a password after every Linux login.

## Threat model

The design protects against a copied application data directory, malicious or malformed imports and cloud responses, accidental credential disclosure, insecure endpoint configuration, unauthorized D-Bus callers, permissive local files, and vulnerable dependencies. It does not claim to protect plaintext visible to the logged-in desktop session, a compromised kernel, screen capture by an already-authorized process, or forensic recovery from unencrypted storage after the former plaintext database has been deleted. Full-disk encryption remains recommended.

## Architecture

A bootstrap security layer obtains a random 256-bit database key from GNOME Secret Service before storage opens. The storage crate opens SQLite through SQLCipher and verifies the cipher and schema before exposing a repository. Secret Service failure is fail-closed: Noor Notes displays a recovery-oriented error and never creates or opens a plaintext replacement.

The existing XChaCha20-Poly1305 vault remains the cloud end-to-end-encryption boundary. Local database encryption and cloud envelope encryption use independent keys and nonces. Sensitive byte buffers are zeroized when dropped, and secrets are never included in errors or logs.

## Existing database migration

On first hardened launch, bootstrap identifies the existing database format before opening it. For a plaintext database it checkpoints WAL state, creates a new encrypted database beside the original, copies all schema and rows through a transaction, runs SQLCipher integrity checks plus application-level note counts and deserialization checks, fsyncs the new file and parent directory, then atomically replaces the original. The old plaintext file is removed only after verification and replacement succeed. If any step fails, the untouched original remains authoritative and the application stops with actionable guidance.

Successful migration also restricts the database directory, database, WAL, SHM, temporary migration files, and corruption backups to the owner. Future corruption backups are encrypted because they copy only an encrypted database. Tests use temporary key providers and never the real desktop keyring.

## Secret management

Replace the external `secret-tool` subprocess with a typed Secret Service adapter. Store the database key, refresh token, and wrapped cloud vault as separate schema-qualified items bound to the application ID and account. Reads reject ambiguous duplicate items. Passwords, access tokens, refresh tokens, database keys, passphrases, and decrypted key material use zeroizing containers where ownership permits. Sign-out deletes account credentials and in-memory sessions.

The database key is generated once with the operating-system CSPRNG and is never derived from the four-digit lock-screen password. Losing both the GNOME Keyring item and every backup makes the encrypted local database unrecoverable; the UI and documentation state this clearly.

## Network and sync hardening

Production Supabase endpoints must use HTTPS with a valid host. Plain HTTP is accepted only for loopback addresses in tests. The HTTP client has connection and total-request timeouts, redirects are disabled, response bodies are size-limited, and revision batches, ciphertext, nonce, timestamps, identifiers, and versions are validated before allocation, decryption, or persistence. Authentication and transport errors stay redacted.

Cloud decryption continues authenticating note ID and revision as associated data. Replay or downgrade revisions are rejected, malformed records are quarantined without replacing valid local notes, rate limits retain bounded exponential backoff, and Supabase row-level-security tests continue proving owner isolation.

## Local input and output hardening

Xpad import rejects symlinks, non-regular files, paths outside the selected root, oversized metadata, oversized note bodies, excessive file counts, invalid UTF-8, and unsafe geometry values. Import remains preview-first and read-only.

Plaintext export is explicitly labeled as unencrypted. Local exports are created with owner-only permissions where the portal permits it, filenames are sanitized and length-limited, and overwrite behavior remains user-confirmed by the system file chooser. No note content is written to logs, crash messages, clipboard, or temporary files by Noor Notes.

## Desktop and packaging boundaries

The GNOME extension keeps its narrow allowlist: only Above and all-workspaces operations, only the session bus, only the application bus owner, and only windows matching the application ID. Invalid identifiers and stale windows are rejected. Snap remains strictly confined and Flatpak permissions remain limited to display, network, and Secret Service access; packaging tests reject new broad filesystem, device, process, or system-bus permissions.

Release builds use locked dependencies, reproducible package inputs with checksums or commit pins, automated advisory auditing, license checks, secret scanning, and generated SBOM artifacts. Security-sensitive failures must not silently fall back to weaker behavior.

## Error handling and recovery

Security failures are categorized as keyring unavailable, key missing, migration failed, encrypted database invalid, network policy violation, malformed remote data, and unsafe import. UI messages explain the next safe action without exposing paths unnecessarily or printing secrets. Migration is retryable and idempotent. Normal autosave never proceeds against a partially migrated database.

## Testing and acceptance criteria

- A copied database does not contain the SQLite header, note text, titles, tags, or journal plaintext.
- Existing plaintext fixtures migrate without note, metadata, history, or revision loss.
- Wrong keys, tampering, missing keyring items, and interrupted migrations fail closed.
- Database-related files and backups are owner-only.
- Non-loopback HTTP endpoints, redirects, oversized responses, invalid envelopes, and revision downgrades are rejected.
- Malicious import paths, symlinks, special files, excessive sizes, and excessive file counts are rejected.
- Secrets and note bodies do not appear in errors or captured logs.
- D-Bus and package permission-policy tests pass.
- Formatting, strict Clippy, all workspace tests, dependency audit, package tests, and installation smoke tests pass before release.

## Compatibility and rollout

The migration preserves the existing database path and application ID. No note schema or visible note behavior changes. The release is staged: keyring and encrypted-storage migration first; network and input validation second; desktop/package and supply-chain policy third. Each stage is independently tested and committed, and no stage is released if it would strand an existing database.
