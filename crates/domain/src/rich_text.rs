use serde::{Deserialize, Serialize};

pub const RICH_DOCUMENT_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListKind {
    Bullet,
    Numbered,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextMarks {
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underline: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub strikethrough: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foreground: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,
}

const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichSpan {
    pub text: String,
    #[serde(default, skip_serializing_if = "TextMarks::is_default")]
    pub marks: TextMarks,
}

impl TextMarks {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichBlock {
    #[serde(default)]
    pub alignment: Alignment,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<ListKind>,
    #[serde(default)]
    pub spans: Vec<RichSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichDocument {
    pub version: u8,
    #[serde(default)]
    pub blocks: Vec<RichBlock>,
}

impl RichDocument {
    pub fn from_plain_text(text: &str) -> Self {
        Self {
            version: RICH_DOCUMENT_VERSION,
            blocks: text
                .split('\n')
                .map(|line| RichBlock {
                    spans: vec![RichSpan {
                        text: line.to_string(),
                        marks: TextMarks::default(),
                    }],
                    ..RichBlock::default()
                })
                .collect(),
        }
    }

    pub fn plain_text(&self) -> String {
        self.blocks
            .iter()
            .map(|block| {
                block
                    .spans
                    .iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn is_supported(&self) -> bool {
        self.version == RICH_DOCUMENT_VERSION
    }
}
