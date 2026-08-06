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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
    #[default]
    Rich,
    Markdown,
    PlainText,
    Code,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    #[default]
    Markdown,
    Named(String),
}

impl SourceLanguage {
    pub fn new(value: &str) -> Option<Self> {
        let value = value.trim();
        (!value.is_empty()
            && value.len() <= 64
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            }))
        .then(|| Self::Named(value.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Markdown => "markdown",
            Self::Named(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorPreferences {
    #[serde(default = "default_zoom_percent")]
    pub zoom_percent: u16,
    #[serde(default = "default_word_wrap")]
    pub word_wrap: bool,
    #[serde(default)]
    pub cursor_offset: usize,
    #[serde(default)]
    pub scroll_offset: i32,
    #[serde(default)]
    pub bookmarks: Vec<u32>,
    #[serde(default)]
    pub view_only: bool,
}

const fn default_zoom_percent() -> u16 {
    100
}

const fn default_word_wrap() -> bool {
    true
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            zoom_percent: default_zoom_percent(),
            word_wrap: default_word_wrap(),
            cursor_offset: 0,
            scroll_offset: 0,
            bookmarks: Vec::new(),
            view_only: false,
        }
    }
}

impl EditorPreferences {
    pub fn set_zoom_percent(&mut self, value: u16) {
        self.zoom_percent = value.clamp(50, 300);
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
    pub pinned: bool,
    pub favorite: bool,
    pub editor_preferences: EditorPreferences,
    pub editor_mode: EditorMode,
    pub source_language: SourceLanguage,
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
    editor_mode: EditorMode,
    #[serde(default)]
    source_language: SourceLanguage,
    #[serde(default)]
    tags: Vec<String>,
    content: String,
    #[serde(default)]
    pinned: bool,
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    editor_preferences: EditorPreferences,
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
            pinned: wire.pinned,
            favorite: wire.favorite,
            editor_preferences: wire.editor_preferences,
            editor_mode: wire.editor_mode,
            source_language: wire.source_language,
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
        copy.editor_preferences = self.editor_preferences.clone();
        copy.editor_preferences.view_only = false;
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
            pinned: false,
            favorite: false,
            editor_preferences: EditorPreferences::default(),
            editor_mode: EditorMode::default(),
            source_language: SourceLanguage::default(),
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
