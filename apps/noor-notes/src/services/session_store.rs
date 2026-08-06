use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

use noor_domain::NoteId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorSessionRecord {
    #[serde(default)]
    pub open_notes: Vec<NoteId>,
    #[serde(default)]
    pub active_note: Option<NoteId>,
}

#[derive(Clone, Debug)]
pub struct SessionStore {
    path: PathBuf,
}

impl SessionStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn save(&self, record: &EditorSessionRecord) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = self.path.with_extension("json.tmp");
        let bytes = serde_json::to_vec(record).map_err(io::Error::other)?;
        std::fs::write(&temporary, bytes)?;
        std::fs::rename(temporary, &self.path)
    }

    pub fn load_valid(&self, available: &[NoteId]) -> io::Result<EditorSessionRecord> {
        if !self.path.exists() {
            return Ok(EditorSessionRecord::default());
        }
        let bytes = std::fs::read(&self.path)?;
        let stored: EditorSessionRecord =
            serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        let available: HashSet<NoteId> = available.iter().copied().collect();
        let mut seen = HashSet::new();
        let open_notes: Vec<NoteId> = stored
            .open_notes
            .into_iter()
            .filter(|id| available.contains(id) && seen.insert(*id))
            .collect();
        let active_note = stored
            .active_note
            .filter(|id| open_notes.contains(id))
            .or_else(|| open_notes.first().copied());
        Ok(EditorSessionRecord {
            open_notes,
            active_note,
        })
    }
}
