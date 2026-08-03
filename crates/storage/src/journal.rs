use noor_domain::{NoteId, Revision};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingChange {
    pub id: Uuid,
    pub note_id: NoteId,
    pub revision: Revision,
    pub operation: String,
    pub payload_json: String,
}
