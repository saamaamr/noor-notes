# Supabase Account Authentication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add production-capable Noor account email sign-up/sign-in and Google OAuth sign-in with secure session restoration, without changing local notes or enabling unfinished sync providers.

**Architecture:** Extend `noor-sync` with the missing Supabase Auth REST operations and a provider-independent PKCE request type. The GTK app loads reviewed public Supabase configuration, owns the short-lived loopback callback, stores only the refresh session in GNOME Keyring, and presents account state through a Libadwaita preferences window launched from `MainWindow`'s application menu.

**Tech Stack:** Rust 1.85+, `reqwest`, `serde`, `url`, `rand`, `sha2`, `base64`, Tokio loopback TCP, GTK4/libadwaita, GIO URI launcher, GNOME Secret Service via `oo7`, Wiremock

**Spec:** `docs/superpowers/specs/2026-09-03-cloud-account-and-backup-design.md`

## Global Constraints

- SQLite remains the local source of truth and the persistence schema is unchanged.
- Supabase receives no plaintext note title, body, tags, rich formatting, or encryption material.
- Production cloud endpoints require HTTPS; loopback HTTP is accepted only as an OAuth redirect target.
- Refresh tokens stay in GNOME Keyring; passwords, access tokens, refresh tokens, authorization codes, and PKCE verifiers are never logged.
- Google Drive scopes are not requested during Google account authentication.
- New UI integrates through `MainWindow`; it creates no dependency on legacy standalone `NoteWindow`.
- Cloud work is asynchronous and must not block GTK's UI thread.
- Missing cloud configuration leaves local-only Noor Notes usable and does not expose a fake successful state.

---

### Task 1: Supabase authentication protocol

**Files:**
- Modify: `crates/sync/Cargo.toml`
- Modify: `crates/sync/src/types.rs`
- Modify: `crates/sync/src/client.rs`
- Modify: `crates/sync/src/lib.rs`
- Modify: `crates/sync/tests/client.rs`

**Interfaces:**
- Produces: `AuthUser { id: String, email: String }`.
- Produces: `AuthSession { access_token, refresh_token, expires_in, user }`.
- Produces: `SignUpOutcome { user, session, confirmation_required }`.
- Produces: `OAuthPkce { authorization_url, verifier, state }`.
- Produces: `SupabaseClient::{sign_up, sign_in, google_oauth_pkce, exchange_oauth_code, refresh_session, user, sign_out}`.

- [ ] **Step 1: Write failing protocol tests**

Add Wiremock tests that assert exact endpoints and safe results:

```rust
#[tokio::test]
async fn signup_reports_email_confirmation_without_inventing_a_session() {
    // POST /auth/v1/signup returns { "user": ..., "session": null }.
    // Assert confirmation_required and session.is_none().
}

#[tokio::test]
async fn google_oauth_uses_s256_pkce_and_exchanges_the_returned_code() {
    // Assert provider=google, exact redirect_to, 43+ character challenge,
    // code_challenge_method=s256, then POST the code + original verifier.
}

#[tokio::test]
async fn refresh_and_logout_use_only_the_supabase_session() {
    // Assert refresh_token grant and authenticated /logout request.
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test -p noor-sync --test client`

Expected: compilation fails because the new types and methods do not exist.

- [ ] **Step 3: Implement minimal Auth REST and PKCE behavior**

Use `OsRng` to create independent verifier and state values, `Sha256` plus URL-safe unpadded Base64 for the S256 challenge, and `Url::query_pairs_mut()` for encoding. Send the public key only in the `apikey` header. Deserialize successful JSON into explicit response structs; map 400/401/403 to actionable non-secret error variants without including response bodies.

- [ ] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test -p noor-sync --test client --test security_policy`

Expected: all focused tests pass with no token material in output.

- [ ] **Step 5: Commit**

```bash
git add crates/sync
git commit -m "feat(sync): add secure Supabase account authentication"
```

### Task 2: Cloud configuration and secure session lifecycle

**Files:**
- Create: `apps/noor-notes/src/cloud_config.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Modify: `apps/noor-notes/src/account.rs`
- Modify: `apps/noor-notes/src/key_store.rs`
- Create: `apps/noor-notes/tests/account.rs`
- Create: `apps/noor-notes/tests/cloud_config.rs`

**Interfaces:**
- Consumes: Task 1's `SupabaseClient`, `AuthSession`, `SignUpOutcome`, and `OAuthPkce`.
- Produces: `CloudConfig::load() -> Result<CloudConfig, CloudConfigError>` and `CloudConfig::client()`.
- Produces: cloneable `AccountController` methods for sign-up, sign-in, OAuth completion, restore, and sign-out.
- Produces: one keyring value under `SecretKind::CloudSession` and account `active`, serialized as `{ user_id, email, refresh_token }`.

- [ ] **Step 1: Write failing configuration and account tests**

```rust
#[test]
fn production_config_rejects_http_and_service_role_keys() {
    assert!(CloudConfig::new("http://example.com", "sb_publishable_test").is_err());
    assert!(CloudConfig::new("https://example.supabase.co", "service_role.secret").is_err());
}

#[tokio::test]
async fn sign_in_persists_one_restorable_cloud_session() {
    // Use InMemoryKeyStore + Wiremock client, sign in, then assert the active
    // keyring record contains identity and refresh token but not password.
}

#[tokio::test]
async fn sign_out_clears_only_cloud_session_material_even_when_revoke_is_offline() {
    // Seed CloudSession and WrappedVault entries; assert CloudSession removal
    // follows the documented lifecycle while unrelated DatabaseKey remains.
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test -p noor-notes --test cloud_config --test account`

Expected: compilation fails because `CloudConfig`, `CloudSession`, and controller methods do not exist.

- [ ] **Step 3: Implement configuration and controller**

Read runtime environment first for development, then build-time `option_env!` values:

```rust
const BUILT_URL: Option<&str> = option_env!("NOOR_SUPABASE_URL");
const BUILT_KEY: Option<&str> = option_env!("NOOR_SUPABASE_PUBLISHABLE_KEY");
```

Trim values, require an HTTPS URL with no credentials, reject empty keys and keys containing `service_role`, and return `NotConfigured` when either value is absent. Store the serialized active cloud session only after successful authentication. On restore, exchange the stored refresh token and replace the keyring value atomically. Sign-out attempts remote revocation and always clears the active cloud session; it never touches `DatabaseKey` or the database.

- [ ] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test -p noor-notes --test cloud_config --test account --test key_store`

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add apps/noor-notes/src/cloud_config.rs apps/noor-notes/src/lib.rs apps/noor-notes/src/account.rs apps/noor-notes/src/key_store.rs apps/noor-notes/tests/account.rs apps/noor-notes/tests/cloud_config.rs
git commit -m "feat(account): persist restorable cloud sessions"
```

### Task 3: Short-lived desktop OAuth callback

**Files:**
- Modify: `Cargo.toml`
- Modify: `apps/noor-notes/Cargo.toml`
- Create: `apps/noor-notes/src/oauth_callback.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Create: `apps/noor-notes/tests/oauth_callback.rs`

**Interfaces:**
- Consumes: Task 1's OAuth code, verifier, and state.
- Produces: `OAuthCallback::bind()`, `redirect_url()`, and `wait(expected_state, timeout)`.
- Callback address: `127.0.0.1:43817` only; path `/auth/callback`.
- Produces: `OAuthCallbackResult::Code(String)` or a redacted typed error.

- [ ] **Step 1: Write failing callback tests**

```rust
#[tokio::test]
async fn callback_accepts_one_matching_state_and_returns_a_success_page() {
    // Bind loopback, send GET /auth/callback?code=valid&state=expected,
    // assert returned code and HTTP 200 response.
}

#[tokio::test]
async fn callback_rejects_wrong_state_replay_non_get_and_oversized_requests() {
    // Each input returns a typed error and never exposes the supplied code.
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test -p noor-notes --test oauth_callback`

Expected: compilation fails because `oauth_callback` does not exist.

- [ ] **Step 3: Implement the bounded loopback receiver**

Enable Tokio `net` and `io-util` workspace features. Bind `TcpListener` to `127.0.0.1:43817`, accept a single connection under a five-minute timeout, read at most 8 KiB, require `GET`, exact callback path, matching state, and a single non-empty code. Write a small static success/error response and close the listener after one terminal result. If the port is occupied, return an actionable error and do not start OAuth.

- [ ] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test -p noor-notes --test oauth_callback`

Expected: all callback security tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml apps/noor-notes/Cargo.toml apps/noor-notes/src/lib.rs apps/noor-notes/src/oauth_callback.rs apps/noor-notes/tests/oauth_callback.rs Cargo.lock
git commit -m "feat(account): handle desktop OAuth callbacks safely"
```

### Task 4: Account and Google sign-in UI

**Files:**
- Create: `apps/noor-notes/src/ui/account_settings.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/app_header.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Create: `apps/noor-notes/tests/account_settings_ui.rs`
- Modify: `apps/noor-notes/tests/app_menu_contract.rs`

**Interfaces:**
- Consumes: `CloudConfig`, `AccountController`, `OAuthCallback`, and the existing application-action pattern.
- Produces: `AccountSettings::new(app, configuration, key_store)` and public test handles for the account controls/status.
- Produces: application action `app.account-settings` and menu label `Account & Sync…`.

- [ ] **Step 1: Write failing UI/menu tests**

```rust
#[test]
fn account_window_exposes_real_signed_out_actions_and_accessible_status() {
    // Assert email, password, Sign Up, Sign In, Continue with Google,
    // status text, focusability, password masking, and narrow-safe layout.
}

#[test]
fn application_menu_opens_account_and_sync_settings() {
    // Collect menu actions and assert app.account-settings exactly once.
}
```

- [ ] **Step 2: Run tests under Xvfb and verify RED**

Run: `xvfb-run -a cargo test -p noor-notes --test account_settings_ui --test app_menu_contract`

Expected: compilation/assertion failure because the window/action is absent.

- [ ] **Step 3: Implement the Libadwaita account window**

Create grouped rows for privacy explanation, email/password controls, Google sign-in, and status. Disable all submit controls while one request is active. Validate non-empty email and a minimum eight-character password before network calls. Launch OAuth with `gtk::UriLauncher`; wait asynchronously for the loopback callback; verify state through `OAuthCallback`; exchange the code through `AccountController`; then render the returned account identity. Missing configuration renders an explicit local-only state and disables authentication controls.

Do not add Drive or OneDrive buttons in this task because their provider flows are not implemented yet.

- [ ] **Step 4: Run focused UI tests and verify GREEN**

Run: `xvfb-run -a cargo test -p noor-notes --test account_settings_ui --test app_menu_contract`

Expected: all UI and menu tests pass without warnings or hangs.

- [ ] **Step 5: Commit**

```bash
git add apps/noor-notes/src/ui/account_settings.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/ui/app_header.rs apps/noor-notes/src/managed_app.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/account_settings_ui.rs apps/noor-notes/tests/app_menu_contract.rs
git commit -m "feat(ui): add Noor account and Google sign-in"
```

### Task 5: Packaging, documentation, and phase verification

**Files:**
- Modify: `snapcraft.yaml`
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Modify: `tests/snap_manifest.sh`
- Modify: `tests/flatpak_manifest.sh`
- Modify: `README.md`
- Modify: `docs/security.md`

**Interfaces:**
- Consumes: Task 3's loopback callback and Task 4's user-visible flow.
- Produces: the minimum sandbox permission contract and exact operator setup instructions for Supabase Google Auth and redirect allow-list.

- [ ] **Step 1: Write failing packaging assertions**

Require Snap's `network-bind` plug. Retain Flatpak’s existing `--share=network`, which covers outbound traffic and loopback listening. Assert documentation does not claim sync is enabled before production public configuration exists.

- [ ] **Step 2: Run packaging tests and verify RED**

Run: `bash tests/snap_manifest.sh && bash tests/flatpak_manifest.sh`

Expected: failure because the loopback-listener permissions are absent.

- [ ] **Step 3: Add minimum permissions and accurate documentation**

Document the exact build variables, HTTPS restriction, public-key rule, Google provider setup, exact `http://127.0.0.1:43817/auth/callback` redirect allow-list entry, keyring storage, local-only fallback, and the fact that Drive/OneDrive backup is still a subsequent phase. Do not include live credentials or tokens.

- [ ] **Step 4: Run the complete verification gate**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash tests/snap_manifest.sh
bash tests/flatpak_manifest.sh
git diff --check
git status --short
```

Expected: every command exits zero; `git status --short` shows only the intended Task 5 documentation and packaging changes before commit.

- [ ] **Step 5: Rebuild and install Noor Notes Dev**

Run: `bash scripts/install-local.sh`

Expected: development build succeeds, installs the separate `noor-notes-dev` launcher, and leaves the notes database untouched.

- [ ] **Step 6: Commit**

```bash
git add snapcraft.yaml packaging/flatpak/io.github.saamaamr.NoorNotes.yml tests/snap_manifest.sh tests/flatpak_manifest.sh README.md docs/security.md
git commit -m "docs: prepare secure account authentication setup"
```
