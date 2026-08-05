# Noor Notes Security Model

Noor Notes keeps SQLCipher-encrypted SQLite as the local source of truth. A random 256-bit database key is stored in GNOME Keyring and never derived from the desktop lock password. Existing plaintext databases are transactionally copied, verified, and atomically replaced. Missing keys and migration failures stop startup rather than falling back to plaintext. The data directory is owner-only (`0700`), and database-related files are owner-only (`0600`).

Cloud records contain versioned XChaCha20-Poly1305 ciphertext, random nonces, revision identifiers, and synchronization timestamps. Note identifiers and revisions are authenticated as associated data. Vault wrapping uses Argon2id and independent recovery material. Temporary key buffers are zeroized.

Database keys, refresh tokens, and wrapped vault material use a typed GNOME Secret Service client, not subprocesses, SQLite, or logs. Production cloud endpoints require HTTPS; redirects, oversized responses, malformed nonces, excessive ciphertext, and future-dated records are rejected. Supabase row-level-security policies restrict operations to the authenticated owner.

Xpad import accepts only bounded regular files inside the selected root and rejects symlinks, special files, traversal, unsafe geometry, and excessive inputs. Exports are deliberately plaintext, labeled unencrypted, use sanitized names, and receive owner-only permissions when written to a directly accessible local path.

The GNOME extension validates the method, argument type, application bus owner, GTK application ID, live compositor window, and UUID-based window title. Snap is strictly confined and Flatpak permissions remain narrowly enumerated.

The local SQLCipher key lives in the user keyring. If that item and all usable backups are lost, the encrypted local database cannot be recovered. Full-disk encryption remains recommended because a compromised logged-in session, kernel, or screen-capture process is outside this threat model.

Dependency auditing has one explicit lockfile-only exception: `RUSTSEC-2023-0071` applies to RSA in SQLx’s optional MySQL backend. Noor Notes disables SQLx default features, enables SQLite only, and verifies with `cargo tree --target all -i rsa@0.9.10` that RSA is absent from every compiled target graph.
