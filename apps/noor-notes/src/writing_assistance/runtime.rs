use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use chrono::Utc;
use noor_domain::Note;
use noor_storage::{PredictionModelRecord, SqliteNoteRepository, StorageError};

use crate::key_store::{KeyStore, SecretKind};

use super::{
    CloudAssistanceClient, GrammarService, PredictionModel, ResolvedWritingAssistance,
    WritingAssistancePreferences, WritingAssistanceStore,
};

const MODEL_SCHEMA_VERSION: u32 = 1;
const MODEL_ENTRY_LIMIT: usize = 50_000;

#[derive(Clone)]
pub struct WritingAssistanceRuntime {
    inner: Arc<RuntimeInner>,
}

struct RuntimeInner {
    repository: SqliteNoteRepository,
    preferences: RwLock<WritingAssistancePreferences>,
    grammar: Arc<GrammarService>,
    prediction: Arc<RwLock<PredictionModel>>,
    cloud: RwLock<Option<CloudAssistanceClient>>,
    pending_rebuild: Mutex<Option<tokio::task::JoinHandle<()>>>,
    rebuild_count: AtomicUsize,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("the local prediction model could not be serialized")]
    Serialization,
    #[error("the local prediction worker stopped unexpectedly")]
    Worker,
}

impl WritingAssistanceRuntime {
    pub async fn new(
        repository: SqliteNoteRepository,
        store: WritingAssistanceStore,
        keys: Arc<dyn KeyStore>,
    ) -> Self {
        let preferences = store.load();
        let key = keys
            .get(SecretKind::WritingAssistanceApiKey, "provider")
            .await
            .ok()
            .flatten()
            .map(|value| value.to_vec());
        let cloud = preferences
            .provider
            .is_validated()
            .then(|| CloudAssistanceClient::new(preferences.provider.clone(), key))
            .and_then(Result::ok);
        Self {
            inner: Arc::new(RuntimeInner {
                repository,
                preferences: RwLock::new(preferences),
                grammar: Arc::new(GrammarService::default()),
                prediction: Arc::new(RwLock::new(PredictionModel::default())),
                cloud: RwLock::new(cloud),
                pending_rebuild: Mutex::new(None),
                rebuild_count: AtomicUsize::new(0),
            }),
        }
    }

    pub fn preferences(&self) -> WritingAssistancePreferences {
        self.inner
            .preferences
            .read()
            .expect("writing preferences lock poisoned")
            .clone()
    }

    pub fn resolved_for(&self, note: &Note) -> ResolvedWritingAssistance {
        self.preferences()
            .resolve(&note.editor_preferences.writing_assistance)
    }

    pub fn grammar_service(&self) -> Arc<GrammarService> {
        self.inner.grammar.clone()
    }

    pub fn prediction_model(&self) -> Arc<RwLock<PredictionModel>> {
        self.inner.prediction.clone()
    }

    pub fn cloud_client(&self) -> Option<CloudAssistanceClient> {
        self.inner
            .cloud
            .read()
            .expect("cloud client lock poisoned")
            .clone()
    }

    pub fn suggest(&self, context: &str, partial: &str, limit: usize) -> Vec<String> {
        self.inner
            .prediction
            .read()
            .expect("prediction model lock poisoned")
            .suggest(context, partial, limit)
    }

    pub fn rebuild_count(&self) -> usize {
        self.inner.rebuild_count.load(Ordering::Relaxed)
    }

    pub async fn rebuild_if_stale(&self) -> Result<(), RuntimeError> {
        let corpus = self.inner.repository.prediction_corpus().await?;
        if let Some(record) = self.inner.repository.load_prediction_model().await? {
            if record.schema_version == MODEL_SCHEMA_VERSION
                && record.corpus_watermark == corpus.watermark
            {
                if let Ok(model) = serde_json::from_str::<PredictionModel>(&record.model_json) {
                    *self
                        .inner
                        .prediction
                        .write()
                        .expect("prediction model lock poisoned") = model;
                    return Ok(());
                }
            }
        }
        self.rebuild_from_corpus(corpus.bodies, corpus.watermark)
            .await
    }

    pub fn schedule_model_rebuild(&self, delay: Duration) {
        let mut pending = self
            .inner
            .pending_rebuild
            .lock()
            .expect("prediction rebuild lock poisoned");
        if let Some(previous) = pending.take() {
            previous.abort();
        }
        let runtime = self.clone();
        *pending = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            if let Ok(corpus) = runtime.inner.repository.prediction_corpus().await {
                let _ = runtime
                    .rebuild_from_corpus(corpus.bodies, corpus.watermark)
                    .await;
            }
        }));
    }

    async fn rebuild_from_corpus(
        &self,
        bodies: Vec<String>,
        watermark: String,
    ) -> Result<(), RuntimeError> {
        let model = tokio::task::spawn_blocking(move || {
            let mut model = PredictionModel::default();
            model.train(&bodies);
            model.prune(MODEL_ENTRY_LIMIT);
            model
        })
        .await
        .map_err(|_| RuntimeError::Worker)?;
        let model_json = serde_json::to_string(&model).map_err(|_| RuntimeError::Serialization)?;
        let record = PredictionModelRecord {
            schema_version: MODEL_SCHEMA_VERSION,
            corpus_watermark: watermark,
            model_json,
            updated_at: Utc::now(),
        };
        self.inner
            .repository
            .replace_prediction_model(&record)
            .await?;
        *self
            .inner
            .prediction
            .write()
            .expect("prediction model lock poisoned") = model;
        self.inner.rebuild_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
