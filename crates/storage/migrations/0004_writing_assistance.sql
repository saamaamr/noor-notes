CREATE TABLE IF NOT EXISTS writing_prediction_model (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    schema_version INTEGER NOT NULL,
    corpus_watermark TEXT NOT NULL,
    model_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
