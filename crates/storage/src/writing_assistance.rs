use chrono::{DateTime, Utc};
use noor_domain::NoteState;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::{SqliteNoteRepository, StorageError};

pub const PREDICTION_MODEL_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredictionCorpus {
    pub bodies: Vec<String>,
    pub watermark: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredictionModelRecord {
    pub schema_version: u32,
    pub corpus_watermark: String,
    pub model_json: String,
    pub updated_at: DateTime<Utc>,
}

impl SqliteNoteRepository {
    pub async fn prediction_corpus(&self) -> Result<PredictionCorpus, StorageError> {
        let rows =
            sqlx::query("SELECT id, content, state_json, revision FROM notes ORDER BY id ASC")
                .fetch_all(&self.pool)
                .await?;
        let mut bodies = Vec::new();
        let mut hasher = Sha256::new();
        for row in rows {
            let state_json: String = row.try_get("state_json")?;
            let state: NoteState = serde_json::from_str(&state_json)?;
            if !matches!(state, NoteState::Active | NoteState::Archived) {
                continue;
            }
            let id: String = row.try_get("id")?;
            let revision: i64 = row.try_get("revision")?;
            let state_name = match state {
                NoteState::Active => "active",
                NoteState::Archived => "archived",
                NoteState::Trashed { .. } => unreachable!(),
            };
            hasher.update(id.as_bytes());
            hasher.update([0]);
            hasher.update(revision.to_string().as_bytes());
            hasher.update([0]);
            hasher.update(state_name.as_bytes());
            hasher.update([0xff]);
            bodies.push(row.try_get("content")?);
        }
        Ok(PredictionCorpus {
            bodies,
            watermark: format!("{:x}", hasher.finalize()),
        })
    }

    pub async fn load_prediction_model(
        &self,
    ) -> Result<Option<PredictionModelRecord>, StorageError> {
        let row = sqlx::query(
            "SELECT schema_version, corpus_watermark, model_json, updated_at
             FROM writing_prediction_model WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let schema_version: i64 = row.try_get("schema_version")?;
        if schema_version != i64::from(PREDICTION_MODEL_SCHEMA_VERSION) {
            return Ok(None);
        }
        let model_json: String = row.try_get("model_json")?;
        if serde_json::from_str::<Value>(&model_json).is_err() {
            return Ok(None);
        }
        let updated_at: String = row.try_get("updated_at")?;
        let Ok(updated_at) = DateTime::parse_from_rfc3339(&updated_at) else {
            return Ok(None);
        };
        Ok(Some(PredictionModelRecord {
            schema_version: schema_version as u32,
            corpus_watermark: row.try_get("corpus_watermark")?,
            model_json,
            updated_at: updated_at.with_timezone(&Utc),
        }))
    }

    pub async fn replace_prediction_model(
        &self,
        record: &PredictionModelRecord,
    ) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO writing_prediction_model
             (id, schema_version, corpus_watermark, model_json, updated_at)
             VALUES (1, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
             schema_version=excluded.schema_version,
             corpus_watermark=excluded.corpus_watermark,
             model_json=excluded.model_json,
             updated_at=excluded.updated_at",
        )
        .bind(i64::from(record.schema_version))
        .bind(&record.corpus_watermark)
        .bind(&record.model_json)
        .bind(record.updated_at.to_rfc3339())
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(())
    }
}
