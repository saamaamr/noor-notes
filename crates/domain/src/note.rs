use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{NoteColor, NoteStyle, RichDocument, WindowGeometry};

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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub color: NoteColor,
    pub tags: Vec<String>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rich_content: Option<RichDocument>,
    pub style: NoteStyle,
    pub geometry: WindowGeometry,
    pub always_on_top: bool,
    pub all_workspaces: bool,
    pub state: NoteState,
    pub revision: Revision,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct NoteWire {
    id: NoteId,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    color: NoteColor,
    #[serde(default)]
    tags: Vec<String>,
    content: String,
    #[serde(default)]
    rich_content: Option<RichDocument>,
    style: NoteStyle,
    geometry: WindowGeometry,
    always_on_top: bool,
    all_workspaces: bool,
    state: NoteState,
    revision: Revision,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for Note {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = NoteWire::deserialize(deserializer)?;
        let title = wire
            .title
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| Note::derive_title(&wire.content));
        Ok(Self {
            id: wire.id,
            title,
            color: wire.color,
            tags: wire.tags,
            content: wire.content,
            rich_content: wire.rich_content,
            style: wire.style,
            geometry: wire.geometry,
            always_on_top: wire.always_on_top,
            all_workspaces: wire.all_workspaces,
            state: wire.state,
            revision: wire.revision,
            created_at: wire.created_at,
            updated_at: wire.updated_at,
        })
    }
}

impl Note {
    pub fn derive_title(content: &str) -> String {
        content
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(|line| line.chars().take(80).collect())
            .unwrap_or_else(|| "Untitled note".into())
    }

    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            "Untitled note"
        } else {
            self.title.trim()
        }
    }

    pub fn set_tags(&mut self, values: Vec<String>) {
        let mut keys = std::collections::HashSet::new();
        self.tags = values
            .into_iter()
            .filter_map(|value| {
                let value = value.trim().to_string();
                if value.is_empty() || !keys.insert(value.to_lowercase()) {
                    None
                } else {
                    Some(value)
                }
            })
            .collect();
    }

    pub fn duplicate(&self, now: DateTime<Utc>) -> Self {
        let mut copy = Self::new(now);
        copy.title = format!("{} copy", self.display_title());
        copy.color = self.color;
        copy.tags = self.tags.clone();
        copy.content = self.content.clone();
        copy.rich_content = self.rich_content.clone();
        copy.style = self.style.clone();
        copy
    }

    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            id: NoteId::new(),
            title: "Untitled note".into(),
            color: NoteColor::default(),
            tags: Vec::new(),
            content: String::new(),
            rich_content: None,
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
