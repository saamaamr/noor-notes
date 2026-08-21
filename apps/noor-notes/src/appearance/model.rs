use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceMode {
    #[default]
    System,
    #[serde(alias = "light", alias = "warm-paper", alias = "cool-mist")]
    Snow,
    #[serde(alias = "graphite", alias = "oled")]
    Midnight,
}

impl AppearanceMode {
    pub const fn action_name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Snow => "snow",
            Self::Midnight => "midnight",
        }
    }

    pub fn from_action_name(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "snow" | "light" | "warm-paper" | "cool-mist" => Some(Self::Snow),
            "midnight" | "graphite" | "oled" => Some(Self::Midnight),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveTheme {
    Snow,
    Midnight,
}

impl EffectiveTheme {
    pub const ALL: [Self; 2] = [Self::Snow, Self::Midnight];

    pub const ALL_CLASSES: [&'static str; 2] = ["nn-theme-snow", "nn-theme-midnight"];

    pub const fn css_class(self) -> &'static str {
        match self {
            Self::Snow => "nn-theme-snow",
            Self::Midnight => "nn-theme-midnight",
        }
    }

    pub const fn palette_prefix(self) -> &'static str {
        match self {
            Self::Snow => "snow",
            Self::Midnight => "midnight",
        }
    }

    pub const fn is_light(self) -> bool {
        matches!(self, Self::Snow)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppearancePreferences {
    pub mode: AppearanceMode,
}

impl AppearancePreferences {
    pub const fn resolve(&self, system: SystemScheme) -> EffectiveTheme {
        match self.mode {
            AppearanceMode::System => match system {
                SystemScheme::Light => EffectiveTheme::Snow,
                SystemScheme::Dark => EffectiveTheme::Midnight,
            },
            AppearanceMode::Snow => EffectiveTheme::Snow,
            AppearanceMode::Midnight => EffectiveTheme::Midnight,
        }
    }
}
