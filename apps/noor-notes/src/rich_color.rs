use crate::appearance::EffectiveTheme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorRole {
    Foreground,
    Highlight,
}

impl ColorRole {
    pub const fn tag_prefix(self) -> &'static str {
        match self {
            Self::Foreground => "noor-fg-",
            Self::Highlight => "noor-bg-",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
}

const FOREGROUND_PRESETS: [ColorPreset; 7] = [
    ColorPreset {
        id: "slate",
        label: "Slate",
        light: "#334155",
        dark: "#E2E8F0",
    },
    ColorPreset {
        id: "blue",
        label: "Blue",
        light: "#1D4ED8",
        dark: "#93C5FD",
    },
    ColorPreset {
        id: "teal",
        label: "Teal",
        light: "#0F766E",
        dark: "#5EEAD4",
    },
    ColorPreset {
        id: "green",
        label: "Green",
        light: "#15803D",
        dark: "#86EFAC",
    },
    ColorPreset {
        id: "amber",
        label: "Amber",
        light: "#A16207",
        dark: "#FCD34D",
    },
    ColorPreset {
        id: "red",
        label: "Red",
        light: "#B91C1C",
        dark: "#FCA5A5",
    },
    ColorPreset {
        id: "purple",
        label: "Purple",
        light: "#7E22CE",
        dark: "#D8B4FE",
    },
];

const HIGHLIGHT_PRESETS: [ColorPreset; 7] = [
    ColorPreset {
        id: "yellow",
        label: "Yellow",
        light: "#FEF3C7",
        dark: "#5F4B16",
    },
    ColorPreset {
        id: "blue",
        label: "Blue",
        light: "#DBEAFE",
        dark: "#1E3A5F",
    },
    ColorPreset {
        id: "mint",
        label: "Mint",
        light: "#CCFBF1",
        dark: "#134E4A",
    },
    ColorPreset {
        id: "green",
        label: "Green",
        light: "#DCFCE7",
        dark: "#14532D",
    },
    ColorPreset {
        id: "peach",
        label: "Peach",
        light: "#FFEDD5",
        dark: "#7C2D12",
    },
    ColorPreset {
        id: "pink",
        label: "Pink",
        light: "#FCE7F3",
        dark: "#6B214B",
    },
    ColorPreset {
        id: "lavender",
        label: "Lavender",
        light: "#EDE9FE",
        dark: "#4C3575",
    },
];

pub const fn presets(role: ColorRole) -> &'static [ColorPreset] {
    match role {
        ColorRole::Foreground => &FOREGROUND_PRESETS,
        ColorRole::Highlight => &HIGHLIGHT_PRESETS,
    }
}

pub fn normalize_stored(role: ColorRole, value: &str) -> Option<String> {
    let value = value.trim();
    if is_rgb(value) {
        return Some(value.to_ascii_uppercase());
    }
    let normalized = match (role, value) {
        (ColorRole::Foreground, "charcoal") => "slate",
        (ColorRole::Highlight, "charcoal") => "charcoal",
        (ColorRole::Highlight, "red") => "pink",
        _ => value,
    };
    presets(role)
        .iter()
        .any(|preset| preset.id == normalized)
        .then(|| normalized.to_string())
        .or_else(|| {
            (role == ColorRole::Highlight && normalized == "charcoal")
                .then(|| normalized.to_string())
        })
}

pub fn rendered_color(role: ColorRole, value: &str, theme: EffectiveTheme) -> Option<String> {
    let normalized = normalize_stored(role, value)?;
    if normalized.starts_with('#') {
        return Some(normalized);
    }
    if role == ColorRole::Highlight && normalized == "charcoal" {
        return Some(
            if theme.is_light() {
                "#D8C99B"
            } else {
                "#5B5030"
            }
            .to_string(),
        );
    }
    let preset = presets(role)
        .iter()
        .find(|preset| preset.id == normalized)?;
    Some(
        if theme.is_light() {
            preset.light
        } else {
            preset.dark
        }
        .to_string(),
    )
}

pub fn tag_name(role: ColorRole, value: &str) -> Option<String> {
    let normalized = normalize_stored(role, value)?;
    let encoded = normalized
        .strip_prefix('#')
        .map(|rgb| format!("hex-{rgb}"))
        .unwrap_or(normalized);
    Some(format!("{}{encoded}", role.tag_prefix()))
}

pub fn stored_value_from_tag(role: ColorRole, name: &str) -> Option<String> {
    let encoded = name.strip_prefix(role.tag_prefix())?;
    if let Some(rgb) = encoded.strip_prefix("hex-") {
        return normalize_stored(role, &format!("#{rgb}"));
    }
    normalize_stored(role, encoded)
}

fn is_rgb(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..]
            .iter()
            .all(|byte| byte.is_ascii_hexdigit())
}
