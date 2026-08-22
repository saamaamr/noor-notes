use std::cell::RefCell;

mod manager;
mod model;
mod palette;
mod preferences;
mod style_runtime;

pub use manager::AppearanceManager;
pub use model::{AppearanceMode, AppearancePreferences, EffectiveTheme, SystemScheme};
pub use palette::ThemePalette;
pub use preferences::AppearanceStore;
pub use style_runtime::{ThemeStyleState, install_static_styles, semantic_stylesheet};

thread_local! {
    static GLOBAL: RefCell<Option<AppearanceManager>> = const { RefCell::new(None) };
}

pub fn install_global(manager: AppearanceManager) {
    GLOBAL.with(|global| global.replace(Some(manager)));
}

pub fn global() -> AppearanceManager {
    try_global().expect("appearance manager must be installed")
}

pub fn try_global() -> Option<AppearanceManager> {
    GLOBAL.with(|global| global.borrow().clone())
}
