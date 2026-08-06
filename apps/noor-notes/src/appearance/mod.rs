mod manager;
mod model;
mod preferences;

pub use manager::AppearanceManager;
pub use model::{AppearanceMode, AppearancePreferences, DarkPalette, EffectiveTheme, SystemScheme};
pub use preferences::AppearanceStore;
