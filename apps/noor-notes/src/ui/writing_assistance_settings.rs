use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;

use crate::key_store::{KeyStore, SecretKind};
use crate::writing_assistance::{
    CloudAssistanceClient, SpellService, WritingAssistancePreferences, WritingAssistanceStore,
};

#[derive(Clone)]
pub struct WritingAssistanceSettings {
    pub window: adw::PreferencesWindow,
    pub spelling: gtk::Switch,
    pub grammar: gtk::Switch,
    pub offline_prediction: gtk::Switch,
    pub cloud: gtk::Switch,
    pub language: gtk::DropDown,
    pub endpoint: gtk::Entry,
    pub model: gtk::Entry,
    pub api_key: gtk::PasswordEntry,
    pub test_connection: gtk::Button,
    pub validation_status: gtk::Label,
    privacy: String,
}

impl WritingAssistanceSettings {
    pub fn new(
        app: &adw::Application,
        store: WritingAssistanceStore,
        keys: Arc<dyn KeyStore>,
    ) -> Self {
        let preferences = Rc::new(RefCell::new(store.load()));
        let window = adw::PreferencesWindow::builder()
            .application(app)
            .title("Writing Assistance")
            .default_width(620)
            .default_height(680)
            .search_enabled(false)
            .build();
        let page = adw::PreferencesPage::new();
        page.set_title("Writing Assistance");
        page.set_icon_name(Some("accessories-text-editor-symbolic"));

        let local = adw::PreferencesGroup::new();
        local.set_title("Private on-device assistance");
        local.set_description(Some(
            "Spelling uses installed dictionaries. English grammar and learned predictions stay on this device.",
        ));
        let spelling = switch_row(
            &local,
            "Spelling",
            "Underline misspelled words using installed system dictionaries",
            preferences.borrow().spelling,
        );
        let grammar = switch_row(
            &local,
            "Grammar",
            "Check English grammar offline",
            preferences.borrow().grammar,
        );
        let offline_prediction = switch_row(
            &local,
            "Offline predictions",
            "Learn suggestions only from active and archived encrypted note bodies",
            preferences.borrow().offline_prediction,
        );
        let languages = SpellService::installed_languages();
        let mut labels = vec!["Automatic".to_owned()];
        let mut codes = vec!["auto".to_owned()];
        for language in languages {
            if !codes.iter().any(|code| code == &language.code) {
                labels.push(format!("{} ({})", language.name, language.code));
                codes.push(language.code);
            }
        }
        let language_refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
        let language = gtk::DropDown::from_strings(&language_refs);
        language.update_property(&[gtk::accessible::Property::Label("Spelling language")]);
        let selected = codes
            .iter()
            .position(|code| code == &preferences.borrow().language)
            .unwrap_or(0);
        language.set_selected(selected as u32);
        let language_row = adw::ActionRow::builder()
            .title("Language")
            .subtitle("Automatic follows the installed default dictionary")
            .build();
        language_row.add_suffix(&language);
        language_row.set_activatable_widget(Some(&language));
        local.add(&language_row);
        page.add(&local);

        let online = adw::PreferencesGroup::new();
        online.set_title("Optional online provider");
        let privacy = "Online grammar sends only the current paragraph (up to 2,000 characters). Online prediction sends only a nearby sentence (up to 800 characters). Titles, tags, other notes, account data, and encryption keys are never sent.".to_owned();
        online.set_description(Some(&privacy));
        let endpoint = gtk::Entry::builder()
            .placeholder_text("https://provider.example/v1")
            .hexpand(true)
            .text(&preferences.borrow().provider.base_url)
            .build();
        endpoint.update_property(&[gtk::accessible::Property::Label("Provider endpoint")]);
        online.add(&entry_row("Provider endpoint", &endpoint));
        let model = gtk::Entry::builder()
            .placeholder_text("Model name")
            .hexpand(true)
            .text(&preferences.borrow().provider.model)
            .build();
        model.update_property(&[gtk::accessible::Property::Label("Provider model")]);
        online.add(&entry_row("Model", &model));
        let api_key = gtk::PasswordEntry::builder()
            .placeholder_text("Stored in GNOME Keyring")
            .hexpand(true)
            .show_peek_icon(true)
            .build();
        api_key.update_property(&[gtk::accessible::Property::Label("Provider API key")]);
        online.add(&entry_row("API key", &api_key));
        let test_connection = gtk::Button::with_label("Test Connection");
        test_connection.add_css_class("suggested-action");
        test_connection.update_property(&[gtk::accessible::Property::Label(
            "Test writing assistance provider connection",
        )]);
        let validation_status =
            gtk::Label::new(Some(if preferences.borrow().provider.is_validated() {
                "Provider validated"
            } else {
                "Provider not validated"
            }));
        validation_status.set_wrap(true);
        let validation_row = adw::ActionRow::builder()
            .title("Connection")
            .subtitle_lines(2)
            .build();
        validation_row.add_suffix(&validation_status);
        validation_row.add_suffix(&test_connection);
        online.add(&validation_row);
        let cloud = switch_row(
            &online,
            "Online AI assistance",
            "Used only after a successful connection test and explicit enablement",
            preferences.borrow().cloud_enabled,
        );
        cloud.set_sensitive(preferences.borrow().provider.is_validated());
        page.add(&online);
        window.add(&page);

        connect_local_switch(
            &spelling,
            preferences.clone(),
            store.clone(),
            |value, enabled| {
                value.spelling = enabled;
            },
        );
        connect_local_switch(
            &grammar,
            preferences.clone(),
            store.clone(),
            |value, enabled| {
                value.grammar = enabled;
            },
        );
        connect_local_switch(
            &offline_prediction,
            preferences.clone(),
            store.clone(),
            |value, enabled| value.offline_prediction = enabled,
        );
        {
            let preferences = preferences.clone();
            let store = store.clone();
            let codes = codes.clone();
            language.connect_selected_notify(move |dropdown| {
                let index = dropdown.selected() as usize;
                if let Some(code) = codes.get(index) {
                    preferences.borrow_mut().language = code.clone();
                    let _ = store.save(&preferences.borrow());
                }
            });
        }
        for entry in [&endpoint, &model] {
            let preferences = preferences.clone();
            let store = store.clone();
            let endpoint = endpoint.clone();
            let model = model.clone();
            let cloud = cloud.clone();
            let status = validation_status.clone();
            entry.connect_changed(move |_| {
                preferences
                    .borrow_mut()
                    .update_provider(&endpoint.text(), &model.text());
                cloud.set_active(false);
                cloud.set_sensitive(false);
                status.set_text("Provider not validated");
                let _ = store.save(&preferences.borrow());
            });
        }
        {
            let preferences = preferences.clone();
            let store = store.clone();
            cloud.connect_active_notify(move |switch| {
                if !switch.is_sensitive() && switch.is_active() {
                    switch.set_active(false);
                    return;
                }
                preferences.borrow_mut().cloud_enabled = switch.is_active();
                let _ = store.save(&preferences.borrow());
            });
        }
        {
            let preferences = preferences.clone();
            let store = store.clone();
            let endpoint = endpoint.clone();
            let model = model.clone();
            let api_key = api_key.clone();
            let cloud = cloud.clone();
            let status = validation_status.clone();
            test_connection.connect_clicked(move |button| {
                button.set_sensitive(false);
                cloud.set_active(false);
                cloud.set_sensitive(false);
                status.set_text("Testing connection…");
                preferences
                    .borrow_mut()
                    .update_provider(&endpoint.text(), &model.text());
                let configuration = preferences.borrow().provider.clone();
                let entered_key = api_key.text().as_bytes().to_vec();
                let keys = keys.clone();
                let preferences = preferences.clone();
                let store = store.clone();
                let cloud = cloud.clone();
                let status = status.clone();
                let button = button.clone();
                gtk::glib::MainContext::default().spawn_local(async move {
                    let key = if entered_key.is_empty() {
                        keys.get(SecretKind::WritingAssistanceApiKey, "provider")
                            .await
                            .ok()
                            .flatten()
                            .map(|value| value.to_vec())
                    } else if keys
                        .put(
                            SecretKind::WritingAssistanceApiKey,
                            "provider",
                            &entered_key,
                        )
                        .await
                        .is_ok()
                    {
                        Some(entered_key)
                    } else {
                        None
                    };
                    let result = match CloudAssistanceClient::new(configuration, key) {
                        Ok(client) => client.test_connection().await,
                        Err(error) => Err(error),
                    };
                    if result.is_ok() {
                        preferences.borrow_mut().mark_provider_validated();
                        let _ = store.save(&preferences.borrow());
                        cloud.set_sensitive(true);
                        status
                            .set_text("Provider validated — online assistance can now be enabled");
                    } else {
                        status.set_text("Connection failed — local assistance remains available");
                    }
                    button.set_sensitive(true);
                });
            });
        }

        Self {
            window,
            spelling,
            grammar,
            offline_prediction,
            cloud,
            language,
            endpoint,
            model,
            api_key,
            test_connection,
            validation_status,
            privacy,
        }
    }

    pub fn privacy_text(&self) -> &str {
        &self.privacy
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn switch_row(
    group: &adw::PreferencesGroup,
    title: &str,
    subtitle: &str,
    active: bool,
) -> gtk::Switch {
    let switch = gtk::Switch::builder()
        .active(active)
        .valign(gtk::Align::Center)
        .focusable(true)
        .build();
    switch.update_property(&[gtk::accessible::Property::Label(title)]);
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .activatable(true)
        .build();
    row.add_suffix(&switch);
    row.set_activatable_widget(Some(&switch));
    group.add(&row);
    switch
}

fn entry_row(title: &str, entry: &impl IsA<gtk::Widget>) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(title).build();
    row.add_suffix(entry);
    row.set_activatable_widget(Some(entry));
    row
}

fn connect_local_switch(
    switch: &gtk::Switch,
    preferences: Rc<RefCell<WritingAssistancePreferences>>,
    store: WritingAssistanceStore,
    update: impl Fn(&mut WritingAssistancePreferences, bool) + 'static,
) {
    switch.connect_active_notify(move |switch| {
        update(&mut preferences.borrow_mut(), switch.is_active());
        let _ = store.save(&preferences.borrow());
    });
}
