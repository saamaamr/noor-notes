use std::cell::RefCell;

mod manager;
mod model;
mod preferences;

pub use manager::AppearanceManager;
pub use model::{
    AppearanceMode, AppearancePreferences, DarkPalette, EffectiveTheme, LightPalette, SystemScheme,
};
pub use preferences::AppearanceStore;

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
