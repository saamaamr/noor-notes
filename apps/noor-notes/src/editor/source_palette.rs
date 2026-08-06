use std::sync::OnceLock;

use gtk::{gio, glib};
use sourceview5::prelude::*;

use crate::appearance::EffectiveTheme;

const RESOURCE_PATH: &str = "resource:///io/github/saamaamr/NoorNotes/styles";
static REGISTERED: OnceLock<bool> = OnceLock::new();

pub fn register() -> bool {
    *REGISTERED.get_or_init(|| {
        let bytes = glib::Bytes::from_static(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/noor-notes.gresource"
        )));
        let Ok(resource) = gio::Resource::from_data(&bytes) else {
            return false;
        };
        gio::resources_register(&resource);
        let manager = sourceview5::StyleSchemeManager::default();
        manager.prepend_search_path(RESOURCE_PATH);
        manager.force_rescan();
        manager.scheme("noor-light").is_some()
    })
}

pub const fn scheme_id(theme: EffectiveTheme) -> &'static str {
    match theme {
        EffectiveTheme::Light => "noor-light",
        EffectiveTheme::Graphite => "noor-graphite",
        EffectiveTheme::Midnight => "noor-midnight",
        EffectiveTheme::Oled => "noor-oled",
    }
}

pub fn apply(buffer: &sourceview5::Buffer, theme: EffectiveTheme) -> Option<glib::GString> {
    register();
    let manager = sourceview5::StyleSchemeManager::default();
    let fallback = match theme {
        EffectiveTheme::Light => "Adwaita",
        EffectiveTheme::Graphite | EffectiveTheme::Midnight | EffectiveTheme::Oled => {
            "Adwaita-dark"
        }
    };
    let scheme = manager
        .scheme(scheme_id(theme))
        .or_else(|| manager.scheme(fallback))?;
    let id = scheme.id();
    buffer.set_style_scheme(Some(&scheme));
    Some(id)
}
