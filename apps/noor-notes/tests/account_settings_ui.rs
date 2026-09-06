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
    for control in [
        settings.email.upcast_ref::<gtk::Widget>(),
        settings.password.upcast_ref(),
        settings.sync_passphrase.upcast_ref(),
        settings.google.upcast_ref(),
        settings.sign_in.upcast_ref(),
        settings.create_vault.upcast_ref(),
        settings.backup_now.upcast_ref(),
    ] {
        assert_eq!(
            control.valign(),
            gtk::Align::Center,
            "Controls must not stretch vertically"
        );
    }
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
    // Exercise real GTK sizing in both palettes, including a narrow window.
    let provider = gtk::CssProvider::new();
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    for theme in noor_notes::appearance::EffectiveTheme::ALL {
        adw::StyleManager::default().set_color_scheme(if theme.is_light() {
            adw::ColorScheme::ForceLight
        } else {
            adw::ColorScheme::ForceDark
        });
        provider.load_from_string(&noor_notes::appearance::semantic_stylesheet(theme));
        for width in [560, 380] {
            settings.window.set_default_size(width, 640);
            for _ in 0..20 {
                while context.pending() {
                    context.iteration(false);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(settings.email.height() <= 40, "Email field stretched");
            assert!(settings.google.height() <= 40, "Google button stretched");
            assert!(
                settings.window.measure(gtk::Orientation::Horizontal, -1).0 <= width,
                "Account layout forces a wider minimum window"
            );
            if let Ok(directory) = std::env::var("NOOR_ACCOUNT_UI_PROOF_DIR") {
                let paintable = gtk::WidgetPaintable::new(Some(&settings.window));
                let snapshot = gtk::Snapshot::new();
                paintable.snapshot(
                    &snapshot,
                    settings.window.width() as f64,
                    settings.window.height() as f64,
                );
                let node = snapshot.to_node().expect("rendered account window");
                let renderer = settings.window.renderer().unwrap();
                let texture = renderer.render_texture(&node, None);
                texture
                    .save_to_png(format!("{directory}/account-{theme:?}-{width}.png"))
                    .unwrap();
            }
        }
    }
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
    for action in [
        &settings.create_vault,
        &settings.unlock_vault,
        &settings.confirm_recovery,
        &settings.sync_now,
    ] {
        let cell = action.parent().expect("responsive action cell");
        assert!(
            !cell.get_visible(),
            "Hidden actions must not leave empty grid cells"
        );
        action.set_visible(true);
        assert!(cell.get_visible(), "Action cell did not follow visibility");
        action.set_visible(false);
        assert!(!cell.get_visible());
    }

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
