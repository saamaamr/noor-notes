use noor_storage::SqliteNoteRepository;

use crate::{ImportError, ImportPreview, ImportReport};

pub async fn import_xpad(
    preview: &ImportPreview,
    repository: &SqliteNoteRepository,
) -> Result<ImportReport, ImportError> {
    let mut report = ImportReport {
        skipped: preview.skipped.clone(),
        ..ImportReport::default()
    };
    for candidate in &preview.importable {
        if repository.has_import_receipt(&candidate.source_key).await? {
            report.already_imported += 1;
            continue;
        }
        repository.save_note(&candidate.note).await?;
        repository
            .record_import_receipt(&candidate.source_key, candidate.note.id)
            .await?;
        report.imported += 1;
    }
    Ok(report)
}
