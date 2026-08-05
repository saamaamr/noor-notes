use std::path::Path;

use sqlx::Row;

use crate::{DatabaseKey, SqliteNoteRepository, StorageError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseFormat {
    Missing,
    Plaintext,
    Encrypted,
}

pub fn detect_database_format(path: &Path) -> Result<DatabaseFormat, StorageError> {
    if !path.exists() {
        return Ok(DatabaseFormat::Missing);
    }
    let bytes = std::fs::read(path).map_err(|source| StorageError::MigrationIo {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.starts_with(b"SQLite format 3\0") {
        Ok(DatabaseFormat::Plaintext)
    } else {
        Ok(DatabaseFormat::Encrypted)
    }
}

pub async fn migrate_or_open(
    path: &Path,
    key: &DatabaseKey,
) -> Result<SqliteNoteRepository, StorageError> {
    match detect_database_format(path)? {
        DatabaseFormat::Missing | DatabaseFormat::Encrypted => {
            SqliteNoteRepository::open_encrypted(path, key).await
        }
        DatabaseFormat::Plaintext => migrate_plaintext(path, key).await,
    }
}

async fn migrate_plaintext(
    path: &Path,
    key: &DatabaseKey,
) -> Result<SqliteNoteRepository, StorageError> {
    let temporary = path.with_extension("db.encrypted-new");
    let guard = path.with_extension("db.plaintext-migration");
    if temporary.exists() || guard.exists() {
        return Err(StorageError::MigrationVerification);
    }

    let legacy = SqliteNoteRepository::open(path).await?;
    sqlx::raw_sql("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&legacy.pool)
        .await?;
    let expected: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes")
        .fetch_one(&legacy.pool)
        .await?;
    let escaped_path = temporary.to_string_lossy().replace('\'', "''");
    let export = format!(
        "ATTACH DATABASE '{escaped_path}' AS encrypted KEY \"x'{}'\"; SELECT sqlcipher_export('encrypted'); DETACH DATABASE encrypted;",
        key.hex()
    );
    sqlx::raw_sql(&export).execute(&legacy.pool).await?;
    legacy.close().await;

    let candidate = SqliteNoteRepository::open_encrypted(&temporary, key).await?;
    let actual: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes")
        .fetch_one(&candidate.pool)
        .await?;
    let integrity = sqlx::query("PRAGMA cipher_integrity_check")
        .fetch_all(&candidate.pool)
        .await?;
    if actual != expected
        || integrity
            .iter()
            .any(|row| row.try_get::<String, _>(0).is_ok_and(|value| value != "ok"))
    {
        candidate.close().await;
        return Err(StorageError::MigrationVerification);
    }
    candidate.close().await;

    rename(path, &guard)?;
    if let Err(error) = rename(&temporary, path) {
        let _ = std::fs::rename(&guard, path);
        return Err(error);
    }
    match SqliteNoteRepository::open_encrypted(path, key).await {
        Ok(repository) => {
            std::fs::remove_file(&guard).map_err(|source| StorageError::MigrationIo {
                path: guard,
                source,
            })?;
            Ok(repository)
        }
        Err(error) => {
            let _ = std::fs::rename(path, &temporary);
            let _ = std::fs::rename(&guard, path);
            Err(error)
        }
    }
}

fn rename(from: &Path, to: &Path) -> Result<(), StorageError> {
    std::fs::rename(from, to).map_err(|source| StorageError::MigrationIo {
        path: from.to_path_buf(),
        source,
    })
}
