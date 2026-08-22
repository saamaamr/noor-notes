use std::cell::RefCell;
use std::rc::Rc;

use super::{EffectiveTheme, ThemePalette};

const COMPONENT_CSS: &str = include_str!("../../resources/design-system.css");

pub fn semantic_stylesheet(theme: EffectiveTheme) -> String {
    let palette = ThemePalette::for_theme(theme).gtk_css();
    let mut stylesheet = String::with_capacity(palette.len() + COMPONENT_CSS.len() + 1);
    stylesheet.push_str(&palette);
    stylesheet.push_str(COMPONENT_CSS);
    stylesheet
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ThemeStyleState {
    active: Option<EffectiveTheme>,
}

impl ThemeStyleState {
    pub fn activate(&mut self, theme: EffectiveTheme) -> bool {
        if self.active == Some(theme) {
            return false;
        }
        self.active = Some(theme);
        true
    }

    pub const fn active(self) -> Option<EffectiveTheme> {
        self.active
    }
}

#[derive(Clone)]
pub(super) struct ThemeStyleRuntime {
    inner: Rc<RefCell<RuntimeState>>,
}

struct RuntimeState {
    provider: gtk::CssProvider,
    theme: ThemeStyleState,
    installed: bool,
}

impl ThemeStyleRuntime {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(RuntimeState {
                provider: gtk::CssProvider::new(),
                theme: ThemeStyleState::default(),
                installed: false,
            })),
        }
    }

    pub fn install(&self, display: &gtk::gdk::Display, theme: EffectiveTheme) {
        let provider = {
            let mut inner = self.inner.borrow_mut();
            if inner.installed {
                None
            } else {
                inner.installed = true;
                Some(inner.provider.clone())
            }
        };
        if let Some(provider) = provider {
            gtk::style_context_add_provider_for_display(
                display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        self.apply(theme);
    }

    pub fn apply(&self, theme: EffectiveTheme) -> bool {
        let mut inner = self.inner.borrow_mut();
        if !inner.theme.activate(theme) {
            return false;
        }
        inner.provider.load_from_string(&semantic_stylesheet(theme));
        true
    }
}

pub fn install_static_styles(display: &gtk::gdk::Display, theme: EffectiveTheme) {
    ThemeStyleRuntime::new().install(display, theme);
}
