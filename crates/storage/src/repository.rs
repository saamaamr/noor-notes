use std::path::Path;
use std::str::FromStr;

use chrono::Utc;
use noor_domain::{Note, NoteId, Revision};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::{PendingChange, StorageError, backup::preserve_corrupt_database};

#[derive(Clone)]
pub struct SqliteNoteRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteNoteRepository {
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| StorageError::PrepareDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = match SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
        {
            Ok(pool) => pool,
            Err(error) => {
                preserve_corrupt_database(path)?;
                return Err(error.into());
            }
        };
        if let Err(error) = sqlx::raw_sql(include_str!("../migrations/0001_initial.sql"))
            .execute(&pool)
            .await
        {
            pool.close().await;
            preserve_corrupt_database(path)?;
            return Err(error.into());
        }
        secure_database_file(path)?;
        Ok(Self { pool })
    }

    pub async fn save_note(&self, note: &Note) -> Result<(), StorageError> {
        let mut tx = self.pool.begin().await?;
        let id = note.id.value().to_string();
        let payload = serde_json::to_string(note)?;
        let state = serde_json::to_string(&note.state)?;

        sqlx::query(
            "INSERT INTO notes (id, payload_json, content, state_json, revision, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET payload_json=excluded.payload_json,
             content=excluded.content, state_json=excluded.state_json,
             revision=excluded.revision, updated_at=excluded.updated_at",
        )
        .bind(&id)
        .bind(&payload)
        .bind(&note.content)
        .bind(state)
        .bind(note.revision.value() as i64)
        .bind(note.created_at.to_rfc3339())
        .bind(note.updated_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO note_styles (note_id, background, foreground, font, opacity)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(note_id) DO UPDATE SET background=excluded.background,
             foreground=excluded.foreground, font=excluded.font, opacity=excluded.opacity",
        )
        .bind(&id)
        .bind(&note.style.background)
        .bind(&note.style.foreground)
        .bind(&note.style.font)
        .bind(note.style.opacity)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO window_geometry
             (note_id, x, y, width, height, always_on_top, all_workspaces)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(note_id) DO UPDATE SET x=excluded.x, y=excluded.y,
             width=excluded.width, height=excluded.height,
             always_on_top=excluded.always_on_top, all_workspaces=excluded.all_workspaces",
        )
        .bind(&id)
        .bind(note.geometry.x)
        .bind(note.geometry.y)
        .bind(note.geometry.width)
        .bind(note.geometry.height)
        .bind(note.always_on_top)
        .bind(note.all_workspaces)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO change_journal
             (id, note_id, revision, operation, payload_json, created_at)
             VALUES (?, ?, ?, 'upsert', ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&id)
        .bind(note.revision.value() as i64)
        .bind(payload)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_note(&self, id: NoteId) -> Result<Option<Note>, StorageError> {
        let payload =
            sqlx::query_scalar::<_, String>("SELECT payload_json FROM notes WHERE id = ?")
                .bind(id.value().to_string())
                .fetch_optional(&self.pool)
                .await?;
        payload
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(StorageError::from)
    }

    pub async fn pending_changes(&self, limit: u32) -> Result<Vec<PendingChange>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, note_id, revision, operation, payload_json
             FROM change_journal WHERE acknowledged_at IS NULL
             ORDER BY created_at ASC LIMIT ?",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let id = Uuid::parse_str(row.try_get("id")?)?;
                let note_id = NoteId::from_uuid(Uuid::parse_str(row.try_get("note_id")?)?);
                let revision = Revision::from_value(row.try_get::<i64, _>("revision")? as u64);
                Ok(PendingChange {
                    id,
                    note_id,
                    revision,
                    operation: row.try_get("operation")?,
                    payload_json: row.try_get("payload_json")?,
                })
            })
            .collect()
    }

    pub async fn ack_change(&self, id: Uuid) -> Result<(), StorageError> {
        sqlx::query("UPDATE change_journal SET acknowledged_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(unix)]
fn secure_database_file(path: &Path) -> Result<(), StorageError> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|source| {
        StorageError::SecureFile {
            path: path.to_path_buf(),
            source,
        }
    })
}

#[cfg(not(unix))]
fn secure_database_file(_path: &Path) -> Result<(), StorageError> {
    Ok(())
}
