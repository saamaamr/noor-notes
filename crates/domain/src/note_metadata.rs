use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteColor {
    #[default]
    Yellow,
    Cream,
    Blue,
    Green,
    Rose,
    Lavender,
}

impl NoteColor {
    pub const fn css_class(self) -> &'static str {
        match self {
            Self::Yellow => "note-yellow",
            Self::Cream => "note-cream",
            Self::Blue => "note-blue",
            Self::Green => "note-green",
            Self::Rose => "note-rose",
            Self::Lavender => "note-lavender",
        }
    }
}
