use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use adw::prelude::*;

use super::{
    AppearanceMode, AppearancePreferences, AppearanceStore, DarkPalette, EffectiveTheme,
    SystemScheme,
};

type Listener = Rc<dyn Fn(AppearancePreferences, EffectiveTheme)>;

#[derive(Clone)]
pub struct AppearanceManager {
    state: Rc<RefCell<State>>,
}

struct State {
    store: AppearanceStore,
    preferences: AppearancePreferences,
    windows: Vec<gtk::glib::WeakRef<gtk::Window>>,
    listeners: Vec<Listener>,
}

impl AppearanceManager {
    pub fn new(store: AppearanceStore) -> Self {
        let preferences = store.load();
        Self {
            state: Rc::new(RefCell::new(State {
                store,
                preferences,
                windows: Vec::new(),
                listeners: Vec::new(),
            })),
        }
    }

    pub fn register_window(&self, window: &impl IsA<gtk::Window>) {
        let window = window.as_ref().clone();
        self.state.borrow_mut().windows.push(window.downgrade());
        self.apply_to_window(&window);
    }

    pub fn set_mode(&self, mode: AppearanceMode) -> io::Result<()> {
        let preferences = {
            let mut state = self.state.borrow_mut();
            state.preferences.mode = mode;
            state.preferences.preferred_dark = match mode {
                AppearanceMode::Graphite => DarkPalette::Graphite,
                AppearanceMode::Midnight => DarkPalette::Midnight,
                AppearanceMode::Oled => DarkPalette::Oled,
                AppearanceMode::System | AppearanceMode::Light => state.preferences.preferred_dark,
            };
            state.store.save(&state.preferences)?;
            state.preferences.clone()
        };
        self.apply(preferences);
        Ok(())
    }

    pub fn cycle_dark_palette(&self) -> io::Result<EffectiveTheme> {
        let next = match self.state.borrow().preferences.preferred_dark {
            DarkPalette::Graphite => AppearanceMode::Midnight,
            DarkPalette::Midnight => AppearanceMode::Oled,
            DarkPalette::Oled => AppearanceMode::Graphite,
        };
        self.set_mode(next)?;
        Ok(self.effective_theme())
    }

    pub fn preferences(&self) -> AppearancePreferences {
        self.state.borrow().preferences.clone()
    }

    pub fn effective_theme(&self) -> EffectiveTheme {
        self.state
            .borrow()
            .preferences
            .resolve(current_system_scheme())
    }

    pub fn active_label(&self) -> &'static str {
        match self.effective_theme() {
            EffectiveTheme::Light => "Light",
            EffectiveTheme::Graphite => "Graphite",
            EffectiveTheme::Midnight => "Midnight",
            EffectiveTheme::Oled => "OLED",
        }
    }

    pub fn subscribe(&self, callback: impl Fn(AppearancePreferences, EffectiveTheme) + 'static) {
        let callback: Listener = Rc::new(callback);
        callback(self.preferences(), self.effective_theme());
        self.state.borrow_mut().listeners.push(callback);
    }

    fn apply(&self, preferences: AppearancePreferences) {
        let theme = preferences.resolve(current_system_scheme());
        let (windows, listeners) = {
            let mut state = self.state.borrow_mut();
            state.windows.retain(|window| window.upgrade().is_some());
            (state.windows.clone(), state.listeners.clone())
        };
        for window in windows.into_iter().filter_map(|window| window.upgrade()) {
            apply_theme_class(&window, theme);
        }
        adw::StyleManager::default().set_color_scheme(match theme {
            EffectiveTheme::Light => adw::ColorScheme::ForceLight,
            _ => adw::ColorScheme::ForceDark,
        });
        for listener in listeners {
            listener(preferences.clone(), theme);
        }
    }

    fn apply_to_window(&self, window: &gtk::Window) {
        apply_theme_class(window, self.effective_theme());
    }
}

fn current_system_scheme() -> SystemScheme {
    if adw::StyleManager::default().is_dark() {
        SystemScheme::Dark
    } else {
        SystemScheme::Light
    }
}

fn apply_theme_class(window: &gtk::Window, theme: EffectiveTheme) {
    for class in EffectiveTheme::ALL_CLASSES {
        window.remove_css_class(class);
    }
    window.add_css_class(theme.css_class());
}
