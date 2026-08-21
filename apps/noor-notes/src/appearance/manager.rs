use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use adw::prelude::*;

use super::{AppearanceMode, AppearancePreferences, AppearanceStore, EffectiveTheme, SystemScheme};

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
        if let Some(settings) = gtk::Settings::default() {
            apply_motion_class(&window, settings.is_gtk_enable_animations());
            let window = window.downgrade();
            settings.connect_gtk_enable_animations_notify(move |settings| {
                if let Some(window) = window.upgrade() {
                    apply_motion_class(&window, settings.is_gtk_enable_animations());
                }
            });
        }
    }

    pub fn set_mode(&self, mode: AppearanceMode) -> io::Result<()> {
        let (preferences, save_result) = {
            let mut state = self.state.borrow_mut();
            state.preferences.mode = mode;
            let save_result = state.store.save(&state.preferences);
            (state.preferences.clone(), save_result)
        };
        self.apply(preferences);
        save_result
    }

    pub fn toggle_theme(&self) -> io::Result<EffectiveTheme> {
        let next = match self.effective_theme() {
            EffectiveTheme::Snow => AppearanceMode::Midnight,
            EffectiveTheme::Midnight => AppearanceMode::Snow,
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
            EffectiveTheme::Snow => "Snow",
            EffectiveTheme::Midnight => "Midnight",
        }
    }

    pub fn subscribe(&self, callback: impl Fn(AppearancePreferences, EffectiveTheme) + 'static) {
        let callback: Listener = Rc::new(callback);
        callback(self.preferences(), self.effective_theme());
        self.state.borrow_mut().listeners.push(callback);
    }

    pub fn install_action(&self, app: &adw::Application) {
        let action = gtk::gio::SimpleAction::new_stateful(
            "appearance",
            Some(gtk::glib::VariantTy::STRING),
            &self.preferences().mode.action_name().to_variant(),
        );
        let selection = self.clone();
        action.connect_activate(move |action, parameter| {
            let Some(value) = parameter.and_then(|value| value.str()) else {
                return;
            };
            let Some(mode) = AppearanceMode::from_action_name(value) else {
                return;
            };
            if selection.set_mode(mode).is_ok() {
                action.set_state(&mode.action_name().to_variant());
            }
        });
        app.add_action(&action);
        let startup = self.clone();
        let live_action = action.clone();
        app.connect_startup(move |_| {
            let action = live_action.clone();
            startup.subscribe(move |preferences, _| {
                action.set_state(&preferences.mode.action_name().to_variant());
            });
            startup.initialize_native_theme();
        });
    }

    fn initialize_native_theme(&self) {
        let system_observer = self.clone();
        adw::StyleManager::default().connect_dark_notify(move |_| {
            let preferences = system_observer.preferences();
            if preferences.mode == AppearanceMode::System {
                system_observer.apply(preferences);
            }
        });
        self.apply(self.preferences());
    }

    fn apply(&self, preferences: AppearancePreferences) {
        let style_manager = adw::StyleManager::default();
        if preferences.mode == AppearanceMode::System {
            style_manager.set_color_scheme(adw::ColorScheme::Default);
        }
        let theme = preferences.resolve(current_system_scheme());
        let (windows, listeners) = {
            let mut state = self.state.borrow_mut();
            state.windows.retain(|window| window.upgrade().is_some());
            (state.windows.clone(), state.listeners.clone())
        };
        for window in windows.into_iter().filter_map(|window| window.upgrade()) {
            apply_theme_class(&window, theme);
        }
        if preferences.mode != AppearanceMode::System {
            style_manager.set_color_scheme(if theme.is_light() {
                adw::ColorScheme::ForceLight
            } else {
                adw::ColorScheme::ForceDark
            });
        }
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

fn apply_motion_class(window: &gtk::Window, animations_enabled: bool) {
    if animations_enabled {
        window.remove_css_class("nn-reduced-motion");
    } else {
        window.add_css_class("nn-reduced-motion");
    }
}
