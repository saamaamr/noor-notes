use std::cell::Cell;
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use libspelling::{Checker, Provider, TextBufferAdapter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellLanguage {
    pub code: String,
    pub name: String,
}

pub struct SpellService;

impl SpellService {
    pub fn attach(
        buffer: &sourceview5::Buffer,
        view: &sourceview5::View,
        language: &str,
        enabled: bool,
    ) -> SpellSession {
        libspelling::init();
        let provider = Provider::default();
        let selected = selected_language(&provider, language);
        let available = selected.is_some();
        let checker = Checker::new(Some(&provider), selected.as_deref());
        let adapter = TextBufferAdapter::new(buffer, &checker);
        let menu = gio::Menu::new();
        menu.append_section(Some("Spelling"), &adapter.menu_model());
        view.set_extra_menu(Some(&menu));
        view.insert_action_group("spelling", Some(&adapter));
        adapter.set_enabled(enabled && available);
        SpellSession {
            adapter,
            provider,
            available: Rc::new(Cell::new(available)),
            requested_enabled: Rc::new(Cell::new(enabled)),
            _checker: checker,
            _menu: menu,
        }
    }

    pub fn installed_languages() -> Vec<SpellLanguage> {
        libspelling::init();
        let provider = Provider::default();
        let mut languages = provider
            .list_languages()
            .iter::<libspelling::Language>()
            .filter_map(Result::ok)
            .filter_map(|language| {
                Some(SpellLanguage {
                    code: language.code()?.to_string(),
                    name: language.name()?.to_string(),
                })
            })
            .collect::<Vec<_>>();
        languages.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.code.cmp(&right.code))
        });
        languages.dedup_by(|left, right| left.code == right.code);
        languages
    }
}

#[derive(Clone)]
pub struct SpellSession {
    adapter: TextBufferAdapter,
    provider: Provider,
    available: Rc<Cell<bool>>,
    requested_enabled: Rc<Cell<bool>>,
    _checker: Checker,
    _menu: gio::Menu,
}

impl SpellSession {
    pub fn is_available(&self) -> bool {
        self.available.get()
    }

    pub fn is_enabled(&self) -> bool {
        self.adapter.is_enabled()
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.requested_enabled.set(enabled);
        self.adapter.set_enabled(enabled && self.available.get());
    }

    pub fn set_language(&self, language: &str) {
        let selected = selected_language(&self.provider, language);
        self.available.set(selected.is_some());
        if let Some(language) = selected {
            self.adapter.set_language(&language);
        }
        self.adapter
            .set_enabled(self.requested_enabled.get() && self.available.get());
    }
}

fn selected_language(provider: &Provider, language: &str) -> Option<String> {
    let language = language.trim();
    let requested = if language.eq_ignore_ascii_case("auto") {
        provider.default_code().map(|value| value.to_string())
    } else {
        (!language.is_empty()).then(|| language.to_owned())
    }?;
    provider.supports_language(&requested).then_some(requested)
}
