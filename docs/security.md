# Noor Notes Security Model

Noor Notes keeps SQLite as the local source of truth. The database is created with owner-only permissions, WAL transactions, an idempotent change journal, and a timestamped safety copy when corruption is detected. Existing Xpad files are read-only import sources and are never changed.

Cloud records contain only versioned XChaCha20-Poly1305 ciphertext, random nonces, revision identifiers, and synchronization timestamps. Note identifiers and revisions are authenticated as associated data. A random vault key is wrapped with an Argon2id-derived passphrase key and separately with a checksum-protected recovery key. Temporary key buffers are zeroized.

Supabase row-level-security policies restrict every select, insert, update, and delete to `owner_id = auth.uid()`. Refresh tokens and wrapped vault material are stored through GNOME Secret Service, not in SQLite or logs.

The GNOME Shell extension owns only two methods: setting Above and all-workspaces state. It accepts calls only from the Noor Notes application bus owner and only for windows whose GTK application ID is `io.github.saamaamr.NoorNotes`. It exposes no note text and restores normal window state when disabled.

The recovery key is the only recovery path if the passphrase and all enrolled devices are lost. Store it offline. Noor Notes cannot recover it from Supabase because the service never receives plaintext keys.
