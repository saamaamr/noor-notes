use chrono::{Duration, TimeZone, Utc};
use noor_domain::{Note, NoteState};
use noor_storage::{DatabaseKey, PredictionModelRecord, SqliteNoteRepository};

async fn encrypted_repository() -> (tempfile::TempDir, SqliteNoteRepository) {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open_encrypted(
        &directory.path().join("notes.db"),
        &DatabaseKey::generate(),
    )
    .await
    .unwrap();
    (directory, repository)
}

#[tokio::test]
async fn corpus_contains_only_active_and_archived_bodies() {
    let (_directory, repository) = encrypted_repository().await;
    let now = Utc.with_ymd_and_hms(2026, 8, 16, 12, 0, 0).unwrap();

    let mut active = Note::new(now);
    active.title = "secret active title".into();
    active.tags = vec!["secret-active-tag".into()];
    active.content = "active body".into();
    let mut archived = Note::new(now);
    archived.content = "archived body".into();
    archived.state = NoteState::Archived;
    let mut trashed = Note::new(now);
    trashed.content = "trashed body".into();
    trashed.state = NoteState::Trashed { deleted_at: now };

    for note in [&active, &archived, &trashed] {
        repository.save_note(note).await.unwrap();
    }

    let corpus = repository.prediction_corpus().await.unwrap();
    assert_eq!(corpus.bodies.len(), 2);
    let joined = corpus.bodies.join(" ");
    assert!(joined.contains("active body"));
    assert!(joined.contains("archived body"));
    assert!(!joined.contains("trashed"));
    assert!(!joined.contains("secret active title"));
    assert!(!joined.contains("secret-active-tag"));
    assert_eq!(corpus.watermark.len(), 64);
}

#[tokio::test]
async fn prediction_record_round_trips_and_malformed_json_is_ignored() {
    let (_directory, repository) = encrypted_repository().await;
    let updated_at = Utc.with_ymd_and_hms(2026, 8, 16, 12, 30, 0).unwrap();
    let record = PredictionModelRecord {
        schema_version: 1,
        corpus_watermark: "abc123".into(),
        model_json: r#"{"bigrams":{}}"#.into(),
        updated_at,
    };

    repository.replace_prediction_model(&record).await.unwrap();
    assert_eq!(
        repository.load_prediction_model().await.unwrap(),
        Some(record)
    );

    let malformed = PredictionModelRecord {
        schema_version: 1,
        corpus_watermark: "broken".into(),
        model_json: "not json".into(),
        updated_at,
    };
    repository
        .replace_prediction_model(&malformed)
        .await
        .unwrap();
    assert_eq!(repository.load_prediction_model().await.unwrap(), None);

    let incompatible = PredictionModelRecord {
        schema_version: 99,
        corpus_watermark: "future".into(),
        model_json: "{}".into(),
        updated_at,
    };
    repository
        .replace_prediction_model(&incompatible)
        .await
        .unwrap();
    assert_eq!(repository.load_prediction_model().await.unwrap(), None);
}

#[tokio::test]
async fn lifecycle_changes_the_corpus_watermark() {
    let (_directory, repository) = encrypted_repository().await;
    let now = Utc.with_ymd_and_hms(2026, 8, 16, 13, 0, 0).unwrap();
    let mut note = Note::new(now);
    note.content = "learn this phrase".into();
    repository.save_note(&note).await.unwrap();
    let active = repository.prediction_corpus().await.unwrap();

    repository
        .trash(note.id, now + Duration::minutes(1))
        .await
        .unwrap();
    let trashed = repository.prediction_corpus().await.unwrap();
    assert!(trashed.bodies.is_empty());
    assert_ne!(active.watermark, trashed.watermark);

    repository
        .restore(note.id, now + Duration::minutes(2))
        .await
        .unwrap();
    let restored = repository.prediction_corpus().await.unwrap();
    assert_eq!(restored.bodies, vec!["learn this phrase"]);
    assert_ne!(trashed.watermark, restored.watermark);

    repository.delete_permanently(note.id).await.unwrap();
    let deleted = repository.prediction_corpus().await.unwrap();
    assert!(deleted.bodies.is_empty());
    assert_ne!(restored.watermark, deleted.watermark);
}
