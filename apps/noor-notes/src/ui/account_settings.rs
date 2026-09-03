use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use noor_sync::{AuthSession, BackupProviderKind, SyncStatus};
use zeroize::Zeroizing;

use crate::account::AccountController;
use crate::cloud_backup::CloudBackupController;
use crate::cloud_config::{CloudConfig, CloudConfigError};
use crate::cloud_sync::{CloudSyncController, CloudSyncState};
use crate::key_store::KeyStore;
use crate::oauth_callback::OAuthCallback;

#[derive(Clone)]
pub struct AccountSettings {
    pub window: adw::PreferencesWindow,
    pub email: gtk::Entry,
    pub password: gtk::PasswordEntry,
    pub sign_up: gtk::Button,
    pub sign_in: gtk::Button,
    pub google: gtk::Button,
    pub sign_out: gtk::Button,
    pub status: gtk::Label,
    pub sync_passphrase: gtk::PasswordEntry,
    pub recovery_display: gtk::Label,
    pub create_vault: gtk::Button,
    pub unlock_vault: gtk::Button,
    pub confirm_recovery: gtk::Button,
    pub sync_now: gtk::Button,
    pub sync_status: gtk::Label,
    pub google_drive_connect: gtk::Button,
    pub google_drive_restore: gtk::Button,
    pub google_drive_status: gtk::Label,
    pub onedrive_connect: gtk::Button,
    pub onedrive_restore: gtk::Button,
    pub onedrive_status: gtk::Label,
    pub backup_now: gtk::Button,
    pub backup_status: gtk::Label,
}

#[derive(Clone)]
struct AccountControls {
    window: adw::PreferencesWindow,
    email: gtk::Entry,
    password: gtk::PasswordEntry,
    sign_up: gtk::Button,
    sign_in: gtk::Button,
    google: gtk::Button,
    sign_out: gtk::Button,
    status: gtk::Label,
    session: Rc<RefCell<Option<AuthSession>>>,
    sync: Option<CloudSyncController>,
    sync_group: adw::PreferencesGroup,
    passphrase_row: adw::ActionRow,
    sync_passphrase: gtk::PasswordEntry,
    recovery_display_row: adw::ActionRow,
    recovery_display: gtk::Label,
    recovery_entry_row: adw::ActionRow,
    recovery_entry: gtk::Entry,
    create_vault: gtk::Button,
    unlock_vault: gtk::Button,
    unlock_recovery: gtk::Button,
    confirm_recovery: gtk::Button,
    sync_now: gtk::Button,
    sync_status: gtk::Label,
    backup: Option<CloudBackupController>,
    backup_group: adw::PreferencesGroup,
    google_drive_connect: gtk::Button,
    google_drive_disconnect: gtk::Button,
    google_drive_restore: gtk::Button,
    google_drive_status: gtk::Label,
    onedrive_connect: gtk::Button,
    onedrive_disconnect: gtk::Button,
    onedrive_restore: gtk::Button,
    onedrive_status: gtk::Label,
    backup_now: gtk::Button,
    backup_status: gtk::Label,
}

impl AccountSettings {
    pub fn new(
        app: &adw::Application,
        configuration: Result<CloudConfig, CloudConfigError>,
        keys: Arc<dyn KeyStore>,
    ) -> Self {
        Self::new_with_services(app, configuration, keys, None, None)
    }

    pub fn new_with_sync(
        app: &adw::Application,
        configuration: Result<CloudConfig, CloudConfigError>,
        keys: Arc<dyn KeyStore>,
        sync: Option<CloudSyncController>,
    ) -> Self {
        Self::new_with_services(app, configuration, keys, sync, None)
    }

    pub fn new_with_services(
        app: &adw::Application,
        configuration: Result<CloudConfig, CloudConfigError>,
        keys: Arc<dyn KeyStore>,
        sync: Option<CloudSyncController>,
        backup: Option<CloudBackupController>,
    ) -> Self {
        let window = adw::PreferencesWindow::builder()
            .application(app)
            .title("Account & Sync")
            .default_width(560)
            .default_height(600)
            .search_enabled(false)
            .build();
        window.add_css_class("nn-settings-window");
        window.add_css_class("nn-account-settings");
        if let Some(appearance) = crate::appearance::try_global() {
            appearance.register_window(&window);
        }

        let page = adw::PreferencesPage::new();
        page.set_title("Account & Sync");
        page.set_icon_name(Some("avatar-default-symbolic"));

        let privacy = adw::PreferencesGroup::new();
        privacy.add_css_class("nn-settings-group");
        privacy.set_title("Private by design");
        privacy.set_description(Some(
            "Your local notes keep working without an account. Cloud note content is encrypted on this device before upload.",
        ));
        page.add(&privacy);

        let account = adw::PreferencesGroup::new();
        account.add_css_class("nn-settings-group");
        account.set_title("Noor account");

        let email = gtk::Entry::builder()
            .hexpand(true)
            .input_purpose(gtk::InputPurpose::Email)
            .placeholder_text("you@example.com")
            .build();
        email.update_property(&[gtk::accessible::Property::Label("Email address")]);
        account.add(&entry_row(
            "Email",
            "Used for account access, never as a note encryption key",
            &email,
        ));

        let password = gtk::PasswordEntry::builder()
            .hexpand(true)
            .show_peek_icon(true)
            .placeholder_text("At least 8 characters")
            .build();
        password.update_property(&[gtk::accessible::Property::Label("Account password")]);
        account.add(&password_row(&password));

        let password_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        password_actions.set_halign(gtk::Align::End);
        let sign_up = gtk::Button::with_label("Sign Up");
        sign_up.add_css_class("nn-account-action");
        let sign_in = gtk::Button::with_label("Sign In");
        sign_in.add_css_class("suggested-action");
        sign_in.add_css_class("nn-account-action");
        password_actions.append(&sign_up);
        password_actions.append(&sign_in);
        let password_action_row = adw::ActionRow::builder()
            .title("Email and password")
            .subtitle("Create a new account or use an existing one")
            .build();
        password_action_row.add_css_class("nn-settings-row");
        password_action_row.add_suffix(&password_actions);
        account.add(&password_action_row);

        let google = gtk::Button::builder()
            .label("Continue with Google")
            .tooltip_text("Sign up or sign in with Google in your browser")
            .build();
        google.add_css_class("nn-account-google");
        google.update_property(&[gtk::accessible::Property::Label("Continue with Google")]);
        let google_row = adw::ActionRow::builder()
            .title("Google")
            .subtitle("Opens your browser; Drive permission is not requested")
            .build();
        google_row.add_css_class("nn-settings-row");
        google_row.add_suffix(&google);
        google_row.set_activatable_widget(Some(&google));
        account.add(&google_row);
        page.add(&account);
        let sync_group = adw::PreferencesGroup::new();
        sync_group.add_css_class("nn-settings-group");
        sync_group.set_title("End-to-end encrypted sync");
        sync_group.set_description(Some(
            "Your passphrase and recovery key stay on this device. Supabase stores only encrypted vault and note data.",
        ));
        sync_group.set_visible(false);

        let sync_passphrase = gtk::PasswordEntry::builder()
            .hexpand(true)
            .show_peek_icon(true)
            .placeholder_text("At least 12 characters")
            .build();
        sync_passphrase
            .update_property(&[gtk::accessible::Property::Label("Sync vault passphrase")]);
        let passphrase_row = adw::ActionRow::builder()
            .title("Vault passphrase")
            .subtitle("Used locally to unlock cloud ciphertext")
            .build();
        passphrase_row.add_css_class("nn-settings-row");
        passphrase_row.add_suffix(&sync_passphrase);
        passphrase_row.set_activatable_widget(Some(&sync_passphrase));
        sync_group.add(&passphrase_row);

        let recovery_display = gtk::Label::new(None);
        recovery_display.set_wrap(true);
        recovery_display.set_selectable(true);
        recovery_display.set_xalign(1.0);
        recovery_display
            .update_property(&[gtk::accessible::Property::Label("One-time recovery key")]);
        let recovery_display_row = adw::ActionRow::builder()
            .title("Save this recovery key")
            .subtitle("It is shown once; store it somewhere private")
            .build();
        recovery_display_row.add_css_class("nn-settings-row");
        recovery_display_row.add_suffix(&recovery_display);
        sync_group.add(&recovery_display_row);

        let recovery_entry = gtk::Entry::builder()
            .hexpand(true)
            .placeholder_text("Type the recovery key")
            .build();
        recovery_entry.update_property(&[gtk::accessible::Property::Label(
            "Recovery key confirmation",
        )]);
        let recovery_entry_row = adw::ActionRow::builder()
            .title("Recovery key")
            .subtitle("Retype the key to confirm, or use it to unlock this device")
            .build();
        recovery_entry_row.add_css_class("nn-settings-row");
        recovery_entry_row.add_suffix(&recovery_entry);
        recovery_entry_row.set_activatable_widget(Some(&recovery_entry));
        sync_group.add(&recovery_entry_row);

        let create_vault = gtk::Button::with_label("Create Encrypted Vault");
        let unlock_vault = gtk::Button::with_label("Unlock");
        let unlock_recovery = gtk::Button::with_label("Use Recovery Key");
        let confirm_recovery = gtk::Button::with_label("Confirm Recovery Key");
        let sync_now = gtk::Button::with_label("Sync Now");
        sync_now.add_css_class("suggested-action");
        let sync_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        sync_actions.set_halign(gtk::Align::End);
        for button in [
            &create_vault,
            &unlock_vault,
            &unlock_recovery,
            &confirm_recovery,
            &sync_now,
        ] {
            button.add_css_class("nn-account-action");
            button.set_sensitive(false);
            button.set_visible(false);
            sync_actions.append(button);
        }
        let sync_action_row = adw::ActionRow::builder()
            .title("Encrypted workspace")
            .subtitle("Sync runs in the background and keeps local editing available")
            .build();
        sync_action_row.add_css_class("nn-settings-row");
        sync_action_row.add_suffix(&sync_actions);
        sync_group.add(&sync_action_row);

        let sync_status = gtk::Label::new(Some("Sign in to configure encrypted sync"));
        sync_status.set_wrap(true);
        sync_status.set_xalign(1.0);
        sync_status.update_property(&[gtk::accessible::Property::Label("Encrypted sync status")]);
        let sync_status_row = adw::ActionRow::builder()
            .title("Sync status")
            .subtitle("Local notes remain available during network errors")
            .build();
        sync_status_row.add_css_class("nn-settings-row");
        sync_status_row.add_suffix(&sync_status);
        sync_group.add(&sync_status_row);
        page.add(&sync_group);

        let backup_group = adw::PreferencesGroup::new();
        backup_group.add_css_class("nn-settings-group");
        backup_group.set_title("Encrypted Drive backups");
        backup_group.set_description(Some(
            "Optional recovery copies use only each provider's private app folder. Provider access is separate from Noor account sign-in.",
        ));
        backup_group.set_visible(false);

        let google_drive_connect = gtk::Button::with_label("Connect");
        let google_drive_restore = gtk::Button::with_label("Restore Latest");
        let google_drive_disconnect = gtk::Button::with_label("Disconnect");
        google_drive_disconnect.add_css_class("destructive-action");
        let google_drive_status = gtk::Label::new(Some("Not connected"));
        google_drive_status.set_wrap(true);
        google_drive_status.set_xalign(1.0);
        let google_actions = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .column_spacing(6)
            .row_spacing(6)
            .min_children_per_line(1)
            .max_children_per_line(4)
            .build();
        google_actions.append(&google_drive_status);
        google_actions.append(&google_drive_connect);
        google_actions.append(&google_drive_restore);
        google_actions.append(&google_drive_disconnect);
        let google_backup_row = adw::ActionRow::builder()
            .title("Google Drive App Data")
            .subtitle("Hidden app-only storage · drive.appdata scope")
            .build();
        google_backup_row.add_css_class("nn-settings-row");
        google_backup_row.add_suffix(&google_actions);
        backup_group.add(&google_backup_row);

        let onedrive_connect = gtk::Button::with_label("Connect");
        let onedrive_restore = gtk::Button::with_label("Restore Latest");
        let onedrive_disconnect = gtk::Button::with_label("Disconnect");
        onedrive_disconnect.add_css_class("destructive-action");
        let onedrive_status = gtk::Label::new(Some("Not connected"));
        onedrive_status.set_wrap(true);
        onedrive_status.set_xalign(1.0);
        let onedrive_actions = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .column_spacing(6)
            .row_spacing(6)
            .min_children_per_line(1)
            .max_children_per_line(4)
            .build();
        onedrive_actions.append(&onedrive_status);
        onedrive_actions.append(&onedrive_connect);
        onedrive_actions.append(&onedrive_restore);
        onedrive_actions.append(&onedrive_disconnect);
        let onedrive_backup_row = adw::ActionRow::builder()
            .title("OneDrive App Folder")
            .subtitle("App-owned storage · Files.ReadWrite.AppFolder scope")
            .build();
        onedrive_backup_row.add_css_class("nn-settings-row");
        onedrive_backup_row.add_suffix(&onedrive_actions);
        backup_group.add(&onedrive_backup_row);

        let backup_now = gtk::Button::with_label("Backup Now");
        backup_now.add_css_class("suggested-action");
        let backup_status = gtk::Label::new(Some("Unlock encrypted sync to use backups"));
        backup_status.set_wrap(true);
        backup_status.set_xalign(1.0);
        let backup_action_row = adw::ActionRow::builder()
            .title("Encrypted recovery copy")
            .subtitle("Creates current and timestamped archives for each connected provider")
            .build();
        backup_action_row.add_css_class("nn-settings-row");
        backup_action_row.add_suffix(&backup_status);
        backup_action_row.add_suffix(&backup_now);
        backup_group.add(&backup_action_row);
        for button in [
            &google_drive_connect,
            &google_drive_restore,
            &google_drive_disconnect,
            &onedrive_connect,
            &onedrive_restore,
            &onedrive_disconnect,
            &backup_now,
        ] {
            button.add_css_class("nn-account-action");
            button.set_sensitive(false);
        }
        for button in [
            &google_drive_restore,
            &google_drive_disconnect,
            &onedrive_restore,
            &onedrive_disconnect,
        ] {
            button.set_visible(false);
        }
        if let Some(controller) = backup.as_ref() {
            if !controller.available(BackupProviderKind::GoogleDrive) {
                google_drive_status.set_text("Not configured in this build");
            }
            if !controller.available(BackupProviderKind::OneDrive) {
                onedrive_status.set_text("Not configured in this build");
            }
        }
        page.add(&backup_group);

        let state = adw::PreferencesGroup::new();
        state.add_css_class("nn-settings-group");
        state.set_title("Status");
        let status = gtk::Label::new(None);
        status.set_wrap(true);
        status.set_xalign(1.0);
        status.update_property(&[gtk::accessible::Property::Label("Account status")]);
        let sign_out = gtk::Button::with_label("Sign Out");
        sign_out.add_css_class("destructive-action");
        sign_out.set_visible(false);
        let state_row = adw::ActionRow::builder()
            .title("Cloud account")
            .subtitle("Local notes are always available")
            .build();
        state_row.add_css_class("nn-settings-row");
        state_row.add_suffix(&status);
        state_row.add_suffix(&sign_out);
        state.add(&state_row);
        page.add(&state);
        window.add(&page);

        let controller = configuration
            .ok()
            .and_then(|configuration| configuration.client().ok())
            .map(|client| AccountController::with_key_store(client, keys));
        let configured = controller.is_some();
        if configured {
            status.set_text("Sign in to set up encrypted sync");
        } else {
            status.set_text("Cloud is not configured · Local notes remain available");
        }
        for button in [&sign_up, &sign_in, &google] {
            button.set_sensitive(configured);
        }

        let controls = AccountControls {
            window: window.clone(),
            email: email.clone(),
            password: password.clone(),
            sign_up: sign_up.clone(),
            sign_in: sign_in.clone(),
            google: google.clone(),
            sign_out: sign_out.clone(),
            status: status.clone(),
            session: Rc::new(RefCell::new(None)),
            sync: sync.clone(),
            sync_group: sync_group.clone(),
            passphrase_row: passphrase_row.clone(),
            sync_passphrase: sync_passphrase.clone(),
            recovery_display_row: recovery_display_row.clone(),
            recovery_display: recovery_display.clone(),
            recovery_entry_row: recovery_entry_row.clone(),
            recovery_entry: recovery_entry.clone(),
            create_vault: create_vault.clone(),
            unlock_vault: unlock_vault.clone(),
            unlock_recovery: unlock_recovery.clone(),
            confirm_recovery: confirm_recovery.clone(),
            sync_now: sync_now.clone(),
            sync_status: sync_status.clone(),
            backup: backup.clone(),
            backup_group: backup_group.clone(),
            google_drive_connect: google_drive_connect.clone(),
            google_drive_disconnect: google_drive_disconnect.clone(),
            google_drive_restore: google_drive_restore.clone(),
            google_drive_status: google_drive_status.clone(),
            onedrive_connect: onedrive_connect.clone(),
            onedrive_disconnect: onedrive_disconnect.clone(),
            onedrive_restore: onedrive_restore.clone(),
            onedrive_status: onedrive_status.clone(),
            backup_now: backup_now.clone(),
            backup_status: backup_status.clone(),
        };

        if let Some(controller) = controller {
            connect_sign_in(&controls, controller.clone());
            connect_sign_up(&controls, controller.clone());
            connect_google(&controls, &window, controller.clone());
            connect_sign_out(&controls, controller.clone());
            connect_restore(&controls, &window, controller);
        }
        if controls.sync.is_some() {
            connect_sync_actions(&controls);
        }
        if controls.backup.is_some() {
            connect_backup_actions(&controls);
        }

        Self {
            window,
            email,
            password,
            sign_up,
            sign_in,
            google,
            sign_out,
            status,
            sync_passphrase,
            recovery_display,
            create_vault,
            unlock_vault,
            confirm_recovery,
            sync_now,
            sync_status,
            google_drive_connect,
            google_drive_restore,
            google_drive_status,
            onedrive_connect,
            onedrive_restore,
            onedrive_status,
            backup_now,
            backup_status,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn entry_row(title: &str, subtitle: &str, entry: &gtk::Entry) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    row.add_css_class("nn-settings-row");
    row.add_suffix(entry);
    row.set_activatable_widget(Some(entry));
    row
}

fn password_row(password: &gtk::PasswordEntry) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title("Password")
        .subtitle("Stored only by Supabase Auth; never written to Noor Notes storage")
        .build();
    row.add_css_class("nn-settings-row");
    row.add_suffix(password);
    row.set_activatable_widget(Some(password));
    row
}

fn validate_credentials(controls: &AccountControls) -> Option<(String, Zeroizing<String>)> {
    let email = controls.email.text().trim().to_owned();
    let password = Zeroizing::new(controls.password.text().to_string());
    if email.is_empty() || !email.contains('@') {
        controls.status.set_text("Enter a valid email address");
        controls.email.grab_focus();
        return None;
    }
    if password.chars().count() < 8 {
        controls
            .status
            .set_text("Password must contain at least 8 characters");
        controls.password.grab_focus();
        return None;
    }
    Some((email, password))
}

fn set_busy(controls: &AccountControls, busy: bool) {
    for button in [&controls.sign_up, &controls.sign_in, &controls.google] {
        button.set_sensitive(!busy);
    }
    controls.email.set_sensitive(!busy);
    controls.password.set_sensitive(!busy);
}

fn show_signed_in(controls: &AccountControls, session: AuthSession) {
    controls
        .status
        .set_text(&format!("Signed in as {}", session.user.email));
    controls.email.set_text(&session.user.email);
    controls.password.set_text("");
    controls.email.set_sensitive(false);
    controls.password.set_sensitive(false);
    controls.sign_up.set_visible(false);
    controls.sign_in.set_visible(false);
    controls.google.set_visible(false);
    controls.sign_out.set_visible(true);
    controls.session.replace(Some(session.clone()));
    let Some(sync) = controls.sync.clone() else {
        return;
    };
    controls.sync_group.set_visible(true);
    controls.sync_status.set_text("Checking encrypted vault…");
    let controls = controls.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        match sync.attach_session(session).await {
            Ok(state) => {
                update_sync_state(&controls, state);
                if state == CloudSyncState::Ready {
                    restore_backup_connections(&controls);
                }
            }
            Err(error) => {
                controls
                    .sync_status
                    .set_text(&format!("Could not check encrypted sync: {error}"));
            }
        }
    });
}

fn show_signed_out(controls: &AccountControls, message: &str) {
    controls.status.set_text(message);
    controls.email.set_sensitive(true);
    controls.password.set_sensitive(true);
    controls.sign_up.set_visible(true);
    controls.sign_in.set_visible(true);
    controls.google.set_visible(true);
    controls.sign_up.set_sensitive(true);
    controls.sign_in.set_sensitive(true);
    controls.google.set_sensitive(true);
    controls.sign_out.set_visible(false);
    controls.session.replace(None);
    controls.sync_group.set_visible(false);
    controls.backup_group.set_visible(false);
    controls.sync_passphrase.set_text("");
    controls.recovery_display.set_text("");
    controls.recovery_entry.set_text("");
}

fn update_sync_state(controls: &AccountControls, state: CloudSyncState) {
    controls
        .sync_group
        .set_visible(state != CloudSyncState::SignedOut);
    controls.passphrase_row.set_visible(false);
    controls.recovery_display_row.set_visible(false);
    controls.recovery_entry_row.set_visible(false);
    let backup_unlocked = matches!(
        state,
        CloudSyncState::Ready | CloudSyncState::Offline | CloudSyncState::Error
    );
    controls
        .backup_group
        .set_visible(state != CloudSyncState::SignedOut && controls.backup.is_some());
    if let Some(backup) = controls.backup.as_ref() {
        controls
            .google_drive_connect
            .set_sensitive(backup_unlocked && backup.available(BackupProviderKind::GoogleDrive));
        controls
            .onedrive_connect
            .set_sensitive(backup_unlocked && backup.available(BackupProviderKind::OneDrive));
        controls.backup_now.set_sensitive(backup_unlocked);
    }
    for button in [
        &controls.create_vault,
        &controls.unlock_vault,
        &controls.unlock_recovery,
        &controls.confirm_recovery,
        &controls.sync_now,
    ] {
        button.set_visible(false);
        button.set_sensitive(false);
    }
    match state {
        CloudSyncState::SignedOut => controls
            .sync_status
            .set_text("Sign in to configure encrypted sync"),
        CloudSyncState::EnrollmentRequired => {
            controls.passphrase_row.set_visible(true);
            controls.create_vault.set_visible(true);
            controls.create_vault.set_sensitive(true);
            controls
                .sync_status
                .set_text("Create a private vault to start encrypted sync");
        }
        CloudSyncState::RecoveryConfirmation => {
            controls.recovery_display_row.set_visible(true);
            controls.recovery_entry_row.set_visible(true);
            controls.confirm_recovery.set_visible(true);
            controls.confirm_recovery.set_sensitive(true);
            controls
                .sync_status
                .set_text("Save and retype the one-time recovery key");
        }
        CloudSyncState::Locked => {
            controls.passphrase_row.set_visible(true);
            controls.recovery_entry_row.set_visible(true);
            controls.unlock_vault.set_visible(true);
            controls.unlock_vault.set_sensitive(true);
            controls.unlock_recovery.set_visible(true);
            controls.unlock_recovery.set_sensitive(true);
            controls
                .sync_status
                .set_text("Unlock this device with the vault passphrase or recovery key");
        }
        CloudSyncState::Ready => {
            controls.sync_now.set_visible(true);
            controls.sync_now.set_sensitive(true);
            controls.sync_status.set_text("Encrypted sync is ready");
        }
        CloudSyncState::Running => {
            controls.sync_now.set_visible(true);
            controls.sync_status.set_text("Syncing encrypted notes…");
        }
        CloudSyncState::Offline => {
            controls.sync_now.set_visible(true);
            controls.sync_now.set_sensitive(true);
            controls
                .sync_status
                .set_text("Offline · Local editing is available; retry when connected");
        }
        CloudSyncState::AuthRequired => controls
            .sync_status
            .set_text("Sign in again to continue encrypted sync"),
        CloudSyncState::Error => {
            controls.sync_now.set_visible(true);
            controls.sync_now.set_sensitive(true);
            controls
                .sync_status
                .set_text("Sync could not finish · Local notes are safe; retry available");
        }
    }
}

fn refresh_backup_controls(controls: &AccountControls) {
    let Some(backup) = controls.backup.clone() else {
        return;
    };
    let controls = controls.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        let google = backup.connected(BackupProviderKind::GoogleDrive).await;
        let onedrive = backup.connected(BackupProviderKind::OneDrive).await;
        controls.google_drive_connect.set_visible(!google);
        controls.google_drive_disconnect.set_visible(google);
        controls.google_drive_restore.set_visible(google);
        controls.google_drive_disconnect.set_sensitive(google);
        controls.google_drive_restore.set_sensitive(google);
        if google {
            controls.google_drive_status.set_text("Connected");
        }
        controls.onedrive_connect.set_visible(!onedrive);
        controls.onedrive_disconnect.set_visible(onedrive);
        controls.onedrive_restore.set_visible(onedrive);
        controls.onedrive_disconnect.set_sensitive(onedrive);
        controls.onedrive_restore.set_sensitive(onedrive);
        if onedrive {
            controls.onedrive_status.set_text("Connected");
        }
        controls.backup_now.set_sensitive(google || onedrive);
    });
}

fn restore_backup_connections(controls: &AccountControls) {
    let Some(backup) = controls.backup.clone() else {
        return;
    };
    let controls = controls.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        for result in backup.restore_connections().await {
            provider_status(&controls, result.provider).set_text(&result.message);
        }
        refresh_backup_controls(&controls);
    });
}

fn connect_backup_actions(controls: &AccountControls) {
    connect_provider(controls, BackupProviderKind::GoogleDrive);
    connect_provider(controls, BackupProviderKind::OneDrive);
    connect_provider_disconnect(controls, BackupProviderKind::GoogleDrive);
    connect_provider_disconnect(controls, BackupProviderKind::OneDrive);
    connect_provider_restore(controls, BackupProviderKind::GoogleDrive);
    connect_provider_restore(controls, BackupProviderKind::OneDrive);

    let controls = controls.clone();
    let button = controls.backup_now.clone();
    button.connect_clicked(move |_| {
        controls.backup_now.set_sensitive(false);
        controls
            .backup_status
            .set_text("Encrypting and uploading backup…");
        let controls = controls.clone();
        let backup = controls
            .backup
            .clone()
            .expect("backup controller is available");
        gtk::glib::MainContext::default().spawn_local(async move {
            let results = backup.backup_now("desktop").await;
            if results.is_empty() {
                controls
                    .backup_status
                    .set_text("Connect a backup provider first");
            } else {
                controls.backup_status.set_text(
                    &results
                        .iter()
                        .map(|result| {
                            format!("{}: {}", provider_title(result.provider), result.message)
                        })
                        .collect::<Vec<_>>()
                        .join(" · "),
                );
            }
            refresh_backup_controls(&controls);
        });
    });
}

fn connect_provider(controls: &AccountControls, kind: BackupProviderKind) {
    let controls = controls.clone();
    let button = provider_connect(&controls, kind);
    button.connect_clicked(move |_| {
        provider_status(&controls, kind).set_text("Opening provider authorization…");
        provider_connect(&controls, kind).set_sensitive(false);
        let controls = controls.clone();
        let backup = controls
            .backup
            .clone()
            .expect("backup controller is available");
        gtk::glib::MainContext::default().spawn_local(async move {
            let result: Result<(), String> = async {
                let callback = match kind {
                    BackupProviderKind::GoogleDrive => OAuthCallback::bind_google_backup().await,
                    BackupProviderKind::OneDrive => OAuthCallback::bind_onedrive_backup().await,
                }
                .map_err(|error| error.to_string())?;
                let authorization = backup
                    .authorization(kind)
                    .map_err(|error| error.to_string())?;
                let launcher = gtk::UriLauncher::new(authorization.authorization_url.as_str());
                launcher
                    .launch_future(Some(&controls.window))
                    .await
                    .map_err(|_| "Could not open the browser".to_owned())?;
                provider_status(&controls, kind).set_text("Finish authorization in your browser…");
                let code = callback
                    .wait(&authorization.state, Duration::from_secs(300))
                    .await
                    .map_err(|error| error.to_string())?;
                backup
                    .connect(kind, &code, &authorization.verifier)
                    .await
                    .map_err(|error| error.to_string())
            }
            .await;
            match result {
                Ok(()) => provider_status(&controls, kind).set_text("Connected"),
                Err(error) => provider_status(&controls, kind)
                    .set_text(&format!("Connection failed: {error}")),
            }
            refresh_backup_controls(&controls);
        });
    });
}

fn connect_provider_disconnect(controls: &AccountControls, kind: BackupProviderKind) {
    let controls = controls.clone();
    let button = provider_disconnect(&controls, kind);
    button.connect_clicked(move |_| {
        let controls = controls.clone();
        let backup = controls
            .backup
            .clone()
            .expect("backup controller is available");
        gtk::glib::MainContext::default().spawn_local(async move {
            match backup.disconnect(kind).await {
                Ok(()) => provider_status(&controls, kind).set_text("Disconnected on this device"),
                Err(error) => provider_status(&controls, kind)
                    .set_text(&format!("Disconnect failed: {error}")),
            }
            refresh_backup_controls(&controls);
        });
    });
}

fn connect_provider_restore(controls: &AccountControls, kind: BackupProviderKind) {
    let controls = controls.clone();
    let button = provider_restore(&controls, kind);
    button.connect_clicked(move |_| {
        let controls = controls.clone();
        let backup = controls.backup.clone().expect("backup controller is available");
        provider_status(&controls, kind).set_text("Checking encrypted archives…");
        gtk::glib::MainContext::default().spawn_local(async move {
            let result: Result<Option<_>, String> = async {
                let objects = backup
                    .list_backups(kind)
                    .await
                    .map_err(|error| error.to_string())?;
                let count = objects
                    .iter()
                    .filter(|object| !object.name.ends_with(".upload"))
                    .count();
                let Some(latest) = objects
                    .into_iter()
                    .filter(|object| !object.name.ends_with(".upload"))
                    .max_by_key(|object| object.modified_at)
                else {
                    return Ok(None);
                };
                let preview = backup
                    .preview_restore(kind, latest)
                    .await
                    .map_err(|error| error.to_string())?;
                provider_status(&controls, kind).set_text(&format!(
                    "{count} archives · Latest has {} notes",
                    preview.archive.note_count
                ));
                let confirmed = crate::ui::dialog_primitives::confirm_action(
                    &controls.window,
                    "Restore encrypted backup?",
                    &format!(
                        "{} contains {} notes from {}. Newer local notes are preserved; conflicts become copies.",
                        provider_title(kind),
                        preview.archive.note_count,
                        preview.archive.created_at.format("%Y-%m-%d %H:%M UTC")
                    ),
                    "Restore",
                )
                .await;
                if !confirmed {
                    return Ok(Some(None));
                }
                let report = backup
                    .restore(&preview.token)
                    .await
                    .map_err(|error| error.to_string())?;
                Ok(Some(Some(report)))
            }
            .await;
            match result {
                Ok(Some(Some(report))) => provider_status(&controls, kind).set_text(&format!(
                    "Restore complete · {} applied · {} conflicts · {} unchanged",
                    report.applied, report.conflicts, report.ignored
                )),
                Ok(Some(None)) => provider_status(&controls, kind).set_text("Restore cancelled"),
                Ok(None) => provider_status(&controls, kind).set_text("No encrypted backups found"),
                Err(error) => provider_status(&controls, kind)
                    .set_text(&format!("Restore failed: {error}")),
            }
        });
    });
}

fn provider_connect(controls: &AccountControls, kind: BackupProviderKind) -> gtk::Button {
    match kind {
        BackupProviderKind::GoogleDrive => controls.google_drive_connect.clone(),
        BackupProviderKind::OneDrive => controls.onedrive_connect.clone(),
    }
}

fn provider_disconnect(controls: &AccountControls, kind: BackupProviderKind) -> gtk::Button {
    match kind {
        BackupProviderKind::GoogleDrive => controls.google_drive_disconnect.clone(),
        BackupProviderKind::OneDrive => controls.onedrive_disconnect.clone(),
    }
}

fn provider_restore(controls: &AccountControls, kind: BackupProviderKind) -> gtk::Button {
    match kind {
        BackupProviderKind::GoogleDrive => controls.google_drive_restore.clone(),
        BackupProviderKind::OneDrive => controls.onedrive_restore.clone(),
    }
}

fn provider_status(controls: &AccountControls, kind: BackupProviderKind) -> gtk::Label {
    match kind {
        BackupProviderKind::GoogleDrive => controls.google_drive_status.clone(),
        BackupProviderKind::OneDrive => controls.onedrive_status.clone(),
    }
}

fn provider_title(kind: BackupProviderKind) -> &'static str {
    match kind {
        BackupProviderKind::GoogleDrive => "Google Drive",
        BackupProviderKind::OneDrive => "OneDrive",
    }
}

fn connect_sync_actions(controls: &AccountControls) {
    let sync = controls.sync.clone().expect("sync controller is available");
    let controls_for_create = controls.clone();
    let sync_for_create = sync.clone();
    controls.create_vault.connect_clicked(move |_| {
        let passphrase = Zeroizing::new(controls_for_create.sync_passphrase.text().to_string());
        controls_for_create.sync_passphrase.set_text("");
        let controls = controls_for_create.clone();
        let sync = sync_for_create.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match sync.begin_enrollment(passphrase.as_bytes()).await {
                Ok(recovery) => {
                    controls.recovery_display.set_text(&recovery);
                    update_sync_state(&controls, CloudSyncState::RecoveryConfirmation);
                }
                Err(error) => controls
                    .sync_status
                    .set_text(&format!("Could not create encrypted vault: {error}")),
            }
        });
    });

    let controls_for_confirm = controls.clone();
    let sync_for_confirm = sync.clone();
    controls.confirm_recovery.connect_clicked(move |_| {
        let recovery = Zeroizing::new(controls_for_confirm.recovery_entry.text().to_string());
        controls_for_confirm.recovery_entry.set_text("");
        let controls = controls_for_confirm.clone();
        let sync = sync_for_confirm.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match sync.confirm_enrollment(&recovery).await {
                Ok(()) => {
                    controls.recovery_display.set_text("");
                    update_sync_state(&controls, CloudSyncState::Ready);
                    restore_backup_connections(&controls);
                }
                Err(error) => controls
                    .sync_status
                    .set_text(&format!("Recovery confirmation failed: {error}")),
            }
        });
    });

    let controls_for_unlock = controls.clone();
    let sync_for_unlock = sync.clone();
    controls.unlock_vault.connect_clicked(move |_| {
        let passphrase = Zeroizing::new(controls_for_unlock.sync_passphrase.text().to_string());
        controls_for_unlock.sync_passphrase.set_text("");
        let controls = controls_for_unlock.clone();
        let sync = sync_for_unlock.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match sync.unlock_with_passphrase(passphrase.as_bytes()).await {
                Ok(()) => {
                    update_sync_state(&controls, CloudSyncState::Ready);
                    restore_backup_connections(&controls);
                }
                Err(error) => controls
                    .sync_status
                    .set_text(&format!("Could not unlock encrypted sync: {error}")),
            }
        });
    });

    let controls_for_recovery = controls.clone();
    let sync_for_recovery = sync.clone();
    controls.unlock_recovery.connect_clicked(move |_| {
        let recovery = Zeroizing::new(controls_for_recovery.recovery_entry.text().to_string());
        controls_for_recovery.recovery_entry.set_text("");
        let controls = controls_for_recovery.clone();
        let sync = sync_for_recovery.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match sync.unlock_with_recovery(&recovery).await {
                Ok(()) => {
                    update_sync_state(&controls, CloudSyncState::Ready);
                    restore_backup_connections(&controls);
                }
                Err(error) => controls
                    .sync_status
                    .set_text(&format!("Could not use recovery key: {error}")),
            }
        });
    });

    let controls_for_sync = controls.clone();
    controls.sync_now.connect_clicked(move |_| {
        update_sync_state(&controls_for_sync, CloudSyncState::Running);
        let controls = controls_for_sync.clone();
        let sync = sync.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match sync.run_once("desktop").await {
                Ok(cycle) => {
                    update_sync_state(&controls, sync.state().await);
                    if cycle.status == SyncStatus::Idle {
                        controls.sync_status.set_text(&format!(
                            "Sync complete · {} uploaded · {} downloaded",
                            cycle.uploaded, cycle.downloaded
                        ));
                    }
                }
                Err(error) => controls
                    .sync_status
                    .set_text(&format!("Sync failed: {error}")),
            }
        });
    });
}

fn connect_sign_in(controls: &AccountControls, controller: AccountController) {
    let controls = controls.clone();
    let button = controls.sign_in.clone();
    button.connect_clicked(move |_| {
        let Some((email, password)) = validate_credentials(&controls) else {
            return;
        };
        set_busy(&controls, true);
        controls.status.set_text("Signing in…");
        controls.password.set_text("");
        let controls = controls.clone();
        let controller = controller.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match controller.sign_in(&email, &password).await {
                Ok(session) => show_signed_in(&controls, session),
                Err(error) => {
                    set_busy(&controls, false);
                    controls
                        .status
                        .set_text(&format!("Sign in failed: {error}"));
                }
            }
        });
    });
}

fn connect_sign_up(controls: &AccountControls, controller: AccountController) {
    let controls = controls.clone();
    let button = controls.sign_up.clone();
    button.connect_clicked(move |_| {
        let Some((email, password)) = validate_credentials(&controls) else {
            return;
        };
        set_busy(&controls, true);
        controls.status.set_text("Creating account…");
        controls.password.set_text("");
        let controls = controls.clone();
        let controller = controller.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match controller.sign_up(&email, &password).await {
                Ok(outcome) => match outcome.session {
                    Some(session) => show_signed_in(&controls, session),
                    None => {
                        set_busy(&controls, false);
                        controls.status.set_text(&format!(
                            "Confirmation sent to {} · Verify the email, then sign in",
                            outcome.user.email
                        ));
                    }
                },
                Err(error) => {
                    set_busy(&controls, false);
                    controls
                        .status
                        .set_text(&format!("Account creation failed: {error}"));
                }
            }
        });
    });
}

fn connect_google(
    controls: &AccountControls,
    window: &adw::PreferencesWindow,
    controller: AccountController,
) {
    let controls = controls.clone();
    let window = window.clone();
    let button = controls.google.clone();
    button.connect_clicked(move |_| {
        set_busy(&controls, true);
        controls.status.set_text("Preparing Google sign-in…");
        let controls = controls.clone();
        let controller = controller.clone();
        let window = window.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            let result = async {
                let callback = OAuthCallback::bind().await?;
                let oauth = controller.google_oauth_pkce(callback.redirect_url().as_str())?;
                let launcher = gtk::UriLauncher::new(oauth.authorization_url.as_str());
                launcher
                    .launch_future(Some(&window))
                    .await
                    .map_err(|_| crate::oauth_callback::OAuthCallbackError::Io)?;
                controls
                    .status
                    .set_text("Finish signing in with Google in your browser…");
                let code = callback
                    .wait(&oauth.state, Duration::from_secs(300))
                    .await?;
                controller
                    .complete_google_sign_in(&code, &oauth.verifier)
                    .await
                    .map_err(GoogleSignInError::Account)
            }
            .await;
            match result {
                Ok(session) => show_signed_in(&controls, session),
                Err(error) => {
                    set_busy(&controls, false);
                    controls
                        .status
                        .set_text(&format!("Google sign-in failed: {error}"));
                }
            }
        });
    });
}

#[derive(Debug, thiserror::Error)]
enum GoogleSignInError {
    #[error(transparent)]
    Callback(#[from] crate::oauth_callback::OAuthCallbackError),
    #[error(transparent)]
    Account(#[from] crate::account::AccountError),
}

fn connect_sign_out(controls: &AccountControls, controller: AccountController) {
    let controls = controls.clone();
    let button = controls.sign_out.clone();
    button.connect_clicked(move |button| {
        button.set_sensitive(false);
        controls.status.set_text("Signing out…");
        let access_token = controls
            .session
            .borrow()
            .as_ref()
            .map(|session| session.access_token.clone());
        let controls = controls.clone();
        let controller = controller.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            let result = controller.sign_out(access_token.as_deref()).await;
            if let Some(sync) = controls.sync.as_ref() {
                sync.disable().await;
            }
            if let Some(backup) = controls.backup.as_ref() {
                backup.lock().await;
            }
            let message = if result.is_ok() {
                "Signed out · Local notes remain available"
            } else {
                "Signed out locally · Cloud revocation will retry after the next sign-in"
            };
            show_signed_out(&controls, message);
            controls.sign_out.set_sensitive(true);
        });
    });
}

fn connect_restore(
    controls: &AccountControls,
    window: &adw::PreferencesWindow,
    controller: AccountController,
) {
    let restored = Rc::new(Cell::new(false));
    let controls = controls.clone();
    window.connect_map(move |_| {
        if restored.replace(true) {
            return;
        }
        set_busy(&controls, true);
        controls.status.set_text("Checking saved account…");
        let controls = controls.clone();
        let controller = controller.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match controller.restore_session().await {
                Ok(Some(session)) => show_signed_in(&controls, session),
                Ok(None) => show_signed_out(&controls, "Sign in to set up encrypted sync"),
                Err(error) => {
                    show_signed_out(&controls, &format!("Sign in required: {error}"));
                }
            }
        });
    });
}
