use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceMode {
    #[default]
    System,
    Light,
    WarmPaper,
    CoolMist,
    Graphite,
    Midnight,
    Oled,
}

impl AppearanceMode {
    pub fn action_name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::WarmPaper => "warm-paper",
            Self::CoolMist => "cool-mist",
            Self::Graphite => "graphite",
            Self::Midnight => "midnight",
            Self::Oled => "oled",
        }
    }

    pub fn from_action_name(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "warm-paper" => Some(Self::WarmPaper),
            "cool-mist" => Some(Self::CoolMist),
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightPalette {
    #[default]
    Snow,
    WarmPaper,
    CoolMist,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveTheme {
    Light,
    WarmPaper,
    CoolMist,
    Graphite,
    Midnight,
    Oled,
}

impl EffectiveTheme {
    pub const ALL: [Self; 6] = [
        Self::Light,
        Self::WarmPaper,
        Self::CoolMist,
        Self::Graphite,
        Self::Midnight,
        Self::Oled,
    ];

    pub const ALL_CLASSES: [&'static str; 6] = [
        "nn-theme-light",
        "nn-theme-warm-paper",
        "nn-theme-cool-mist",
        "nn-theme-graphite",
        "nn-theme-midnight",
        "nn-theme-oled",
    ];

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Light => "nn-theme-light",
            Self::WarmPaper => "nn-theme-warm-paper",
            Self::CoolMist => "nn-theme-cool-mist",
            Self::Graphite => "nn-theme-graphite",
            Self::Midnight => "nn-theme-midnight",
            Self::Oled => "nn-theme-oled",
        }
    }

    pub const fn palette_prefix(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::WarmPaper => "warm",
            Self::CoolMist => "mist",
            Self::Graphite => "graphite",
            Self::Midnight => "midnight",
            Self::Oled => "oled",
        }
    }

    pub const fn is_light(self) -> bool {
        matches!(self, Self::Light | Self::WarmPaper | Self::CoolMist)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppearancePreferences {
    pub mode: AppearanceMode,
    pub preferred_light: LightPalette,
    pub preferred_dark: DarkPalette,
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            mode: AppearanceMode::System,
            preferred_light: LightPalette::Snow,
            preferred_dark: DarkPalette::Graphite,
        }
    }
}

impl AppearancePreferences {
    pub fn resolve(&self, system: SystemScheme) -> EffectiveTheme {
        match self.mode {
            AppearanceMode::System => match system {
                SystemScheme::Light => match self.preferred_light {
                    LightPalette::Snow => EffectiveTheme::Light,
                    LightPalette::WarmPaper => EffectiveTheme::WarmPaper,
                    LightPalette::CoolMist => EffectiveTheme::CoolMist,
                },
                SystemScheme::Dark => match self.preferred_dark {
                    DarkPalette::Graphite => EffectiveTheme::Graphite,
                    DarkPalette::Midnight => EffectiveTheme::Midnight,
                    DarkPalette::Oled => EffectiveTheme::Oled,
                },
            },
            AppearanceMode::Light => EffectiveTheme::Light,
            AppearanceMode::WarmPaper => EffectiveTheme::WarmPaper,
            AppearanceMode::CoolMist => EffectiveTheme::CoolMist,
            AppearanceMode::Graphite => EffectiveTheme::Graphite,
            AppearanceMode::Midnight => EffectiveTheme::Midnight,
            AppearanceMode::Oled => EffectiveTheme::Oled,
        }
    }
}
