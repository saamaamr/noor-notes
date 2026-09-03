use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use noor_sync::AuthSession;
use zeroize::Zeroizing;

use crate::account::AccountController;
use crate::cloud_config::{CloudConfig, CloudConfigError};
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
}

#[derive(Clone)]
struct AccountControls {
    email: gtk::Entry,
    password: gtk::PasswordEntry,
    sign_up: gtk::Button,
    sign_in: gtk::Button,
    google: gtk::Button,
    sign_out: gtk::Button,
    status: gtk::Label,
    session: Rc<RefCell<Option<AuthSession>>>,
}

impl AccountSettings {
    pub fn new(
        app: &adw::Application,
        configuration: Result<CloudConfig, CloudConfigError>,
        keys: Arc<dyn KeyStore>,
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
            email: email.clone(),
            password: password.clone(),
            sign_up: sign_up.clone(),
            sign_in: sign_in.clone(),
            google: google.clone(),
            sign_out: sign_out.clone(),
            status: status.clone(),
            session: Rc::new(RefCell::new(None)),
        };

        if let Some(controller) = controller {
            connect_sign_in(&controls, controller.clone());
            connect_sign_up(&controls, controller.clone());
            connect_google(&controls, &window, controller.clone());
            connect_sign_out(&controls, controller.clone());
            connect_restore(&controls, &window, controller);
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
    controls.status.set_text(&format!(
        "Signed in as {} · Sync setup is ready",
        session.user.email
    ));
    controls.email.set_text(&session.user.email);
    controls.password.set_text("");
    controls.email.set_sensitive(false);
    controls.password.set_sensitive(false);
    controls.sign_up.set_visible(false);
    controls.sign_in.set_visible(false);
    controls.google.set_visible(false);
    controls.sign_out.set_visible(true);
    controls.session.replace(Some(session));
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
