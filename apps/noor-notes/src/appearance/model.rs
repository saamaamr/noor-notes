use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceMode {
    #[default]
    System,
    Light,
    Graphite,
    Midnight,
    Oled,
}

impl AppearanceMode {
    pub fn action_name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Graphite => "graphite",
            Self::Midnight => "midnight",
            Self::Oled => "oled",
        }
    }

    pub fn from_action_name(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "graphite" => Some(Self::Graphite),
            "midnight" => Some(Self::Midnight),
            "oled" => Some(Self::Oled),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DarkPalette {
    #[default]
    Graphite,
    Midnight,
    Oled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveTheme {
    Light,
    Graphite,
    Midnight,
    Oled,
}

impl EffectiveTheme {
    pub const ALL_CLASSES: [&'static str; 4] = [
        "nn-theme-light",
        "nn-theme-graphite",
        "nn-theme-midnight",
        "nn-theme-oled",
    ];

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Light => "nn-theme-light",
            Self::Graphite => "nn-theme-graphite",
            Self::Midnight => "nn-theme-midnight",
            Self::Oled => "nn-theme-oled",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppearancePreferences {
    pub mode: AppearanceMode,
    pub preferred_dark: DarkPalette,
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            mode: AppearanceMode::System,
            preferred_dark: DarkPalette::Graphite,
        }
    }
}

impl AppearancePreferences {
    pub fn resolve(&self, system: SystemScheme) -> EffectiveTheme {
        match self.mode {
            AppearanceMode::System => match system {
                SystemScheme::Light => EffectiveTheme::Light,
                SystemScheme::Dark => match self.preferred_dark {
                    DarkPalette::Graphite => EffectiveTheme::Graphite,
                    DarkPalette::Midnight => EffectiveTheme::Midnight,
                    DarkPalette::Oled => EffectiveTheme::Oled,
                },
            },
            AppearanceMode::Light => EffectiveTheme::Light,
            AppearanceMode::Graphite => EffectiveTheme::Graphite,
            AppearanceMode::Midnight => EffectiveTheme::Midnight,
            AppearanceMode::Oled => EffectiveTheme::Oled,
        }
    }
}
