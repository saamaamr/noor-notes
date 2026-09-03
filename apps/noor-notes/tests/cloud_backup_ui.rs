use std::sync::Arc;

use adw::prelude::*;
use noor_notes::cloud_backup::{BackupConfiguration, CloudBackupController};
use noor_notes::cloud_config::CloudConfig;
use noor_notes::cloud_sync::CloudSyncController;
use noor_notes::key_store::InMemoryKeyStore;
use noor_notes::ui::account_settings::AccountSettings;
use noor_storage::SqliteNoteRepository;
use noor_sync::{EndpointPolicy, SupabaseClient};

#[tokio::test(flavor = "current_thread")]
async fn backup_controls_are_real_responsive_and_explicit_when_providers_are_unconfigured() {
    gtk::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.BackupSettingsTest")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let keys = Arc::new(InMemoryKeyStore::default());
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteNoteRepository::open(&directory.path().join("notes.db"))
        .await
        .unwrap();
    let client = SupabaseClient::new(
        "https://example.supabase.co",
        "sb_publishable_test",
        EndpointPolicy::Production,
    )
    .unwrap();
    let sync = CloudSyncController::new(repository.clone(), client, keys.clone());
    let backup = CloudBackupController::new(
        repository,
        sync.clone(),
        keys.clone(),
        BackupConfiguration::default(),
    )
    .unwrap();
    let settings = AccountSettings::new_with_services(
        &app,
        CloudConfig::new("https://example.supabase.co", "sb_publishable_test"),
        keys,
        Some(sync),
        Some(backup),
    );

    settings.window.set_default_size(360, 640);
    settings.present();
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }

    assert_eq!(
        settings.google_drive_connect.label().as_deref(),
        Some("Connect")
    );
    assert_eq!(
        settings.google_drive_restore.label().as_deref(),
        Some("Restore Latest")
    );
    assert_eq!(
        settings.onedrive_connect.label().as_deref(),
        Some("Connect")
    );
    assert_eq!(
        settings.onedrive_restore.label().as_deref(),
        Some("Restore Latest")
    );
    assert_eq!(settings.backup_now.label().as_deref(), Some("Backup Now"));
    assert!(
        settings
            .google_drive_status
            .text()
            .contains("Not configured")
    );
    assert!(settings.onedrive_status.text().contains("Not configured"));
    assert!(!settings.google_drive_connect.is_sensitive());
    assert!(!settings.onedrive_connect.is_sensitive());
    assert!(!settings.backup_now.is_sensitive());
    assert!(settings.backup_status.text().contains("Unlock"));
}
