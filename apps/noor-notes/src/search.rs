use noor_domain::Note;
use noor_storage::{SqliteNoteRepository, StorageError};

pub async fn search_notes(
    repository: &SqliteNoteRepository,
    query: &str,
) -> Result<Vec<Note>, StorageError> {
    repository.search_notes(query.trim()).await
}
