use std::sync::Arc;

use adw::prelude::*;
use noor_notes::cloud_config::{CloudConfig, CloudConfigError};
use noor_notes::key_store::InMemoryKeyStore;
use noor_notes::ui::account_settings::AccountSettings;

#[test]
fn account_window_exposes_real_actions_and_an_explicit_local_only_state() {
    gtk::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.AccountSettingsTest")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();

    let config = CloudConfig::new("https://example.supabase.co", "sb_publishable_test").unwrap();
    let settings = AccountSettings::new(&app, Ok(config), Arc::new(InMemoryKeyStore::default()));

    assert!(settings.window.has_css_class("nn-settings-window"));
    assert!(settings.window.has_css_class("nn-account-settings"));
    settings.present();
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
    assert!(settings.email.grab_focus());
    assert!(settings.password.grab_focus());
    assert!(settings.password.shows_peek_icon());
    assert!(settings.sign_up.is_sensitive());
    assert!(settings.sign_in.is_sensitive());
    assert!(settings.google.is_sensitive());
    assert!(!settings.sign_out.is_visible());
    assert!(settings.status.text().contains("encrypted sync"));
    assert_eq!(
        settings.create_vault.label().as_deref(),
        Some("Create Encrypted Vault")
    );
    assert_eq!(settings.unlock_vault.label().as_deref(), Some("Unlock"));
    assert_eq!(
        settings.confirm_recovery.label().as_deref(),
        Some("Confirm Recovery Key")
    );
    assert_eq!(settings.sync_now.label().as_deref(), Some("Sync Now"));
    assert!(settings.sync_passphrase.shows_peek_icon());
    assert!(settings.recovery_display.is_selectable());
    assert!(!settings.create_vault.is_visible());
    assert!(!settings.sync_now.is_visible());

    let local_only = AccountSettings::new(
        &app,
        Err(CloudConfigError::NotConfigured),
        Arc::new(InMemoryKeyStore::default()),
    );
    assert!(!local_only.sign_up.is_sensitive());
    assert!(!local_only.sign_in.is_sensitive());
    assert!(!local_only.google.is_sensitive());
    assert!(
        local_only
            .status
            .text()
            .contains("Local notes remain available")
    );
    assert!(!local_only.create_vault.is_sensitive());
    assert!(!local_only.unlock_vault.is_sensitive());
    assert!(!local_only.sync_now.is_sensitive());
}
