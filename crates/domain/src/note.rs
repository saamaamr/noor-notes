use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{NoteStyle, WindowGeometry};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoteId(Uuid);

impl NoteId {
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn value(self) -> Uuid {
        self.0
    }
}

impl Default for NoteId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Revision(u64);

impl Revision {
    pub const fn from_value(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteState {
    Active,
    Archived,
    Trashed { deleted_at: DateTime<Utc> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub content: String,
    pub style: NoteStyle,
    pub geometry: WindowGeometry,
    pub always_on_top: bool,
    pub all_workspaces: bool,
    pub state: NoteState,
    pub revision: Revision,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            id: NoteId::new(),
            content: String::new(),
            style: NoteStyle::default(),
            geometry: WindowGeometry::default(),
            always_on_top: false,
            all_workspaces: false,
            state: NoteState::Active,
            revision: Revision::default(),
            created_at: now,
            updated_at: now,
        }
    }
}
