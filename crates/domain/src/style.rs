use serde::{Deserialize, Serialize};

const MIN_OPACITY: f64 = 0.35;
const MAX_OPACITY: f64 = 1.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteStyle {
    pub background: String,
    pub foreground: String,
    pub font: String,
    pub opacity: f64,
}

impl NoteStyle {
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(MIN_OPACITY, MAX_OPACITY);
    }
}

impl Default for NoteStyle {
    fn default() -> Self {
        Self {
            background: "#F6D365".into(),
            foreground: "#261F0F".into(),
            font: "Sans 12".into(),
            opacity: MAX_OPACITY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: i32,
    pub height: i32,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 360,
            height: 320,
        }
    }
}
