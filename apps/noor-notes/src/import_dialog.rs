use std::path::Path;

use noor_storage::SqliteNoteRepository;
use noor_xpad_import::{ImportError, ImportPreview, ImportReport, import_xpad, scan_xpad};

pub struct ImportFlow {
    preview: ImportPreview,
}

impl ImportFlow {
    pub fn from_path(path: &Path) -> Result<Self, ImportError> {
        Ok(Self {
            preview: scan_xpad(path)?,
        })
    }

    pub fn preview(&self) -> &ImportPreview {
        &self.preview
    }

    pub async fn confirm(
        self,
        repository: &SqliteNoteRepository,
    ) -> Result<ImportReport, ImportError> {
        import_xpad(&self.preview, repository).await
    }
}
