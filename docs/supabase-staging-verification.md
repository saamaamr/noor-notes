# Supabase staging verification — 2026-09-06

## Configured and observed

- Applied both repository migrations: encrypted revisions and encrypted vaults.
- Both tables have RLS enabled and four owner-scoped policies.
- Google provider enabled; nonce validation remains enabled and users without email remain disallowed.
- Desktop loopback redirect and per-attempt state redirect are allowlisted.
- Dedicated Google web client uses the Supabase Auth callback (not the desktop loopback URL).
- Replaced the disclosed OAuth secret; disabled and deleted the old secret. No OAuth client secret is stored in this repository or desktop build.
- Live transactional SQL smoke test passed: each authenticated fixture user sees only its own rows, cross-owner writes are denied, and anonymous reads return no rows. All fixtures were rolled back.

## Application regression fixes

- A rejected refresh no longer leaves manual sync permanently `Running`.
- Cursor-storage errors leave an explicit error state rather than a stuck running state.
- Account controls refresh their enabled state after failed as well as successful attempts.
- Offline, authentication-required, and error cycles no longer report “Sync complete”.

## Verification

- `cargo test -p noor-sync`: 27 tests passed using local HTTP fixtures.
- Account/config/callback/sync/account-settings GTK suites: 17 tests passed after the fixes; GTK used Xvfb.
- `cargo clippy -p noor-notes --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed before this documentation update.

## Still required before claiming live end-to-end readiness

- Sign in through the installed Dev application's **Account & Sync** window using a real Google account; confirm callback and session restoration.
- Create/unlock the encrypted vault, sync a disposable note, then verify a second device can download/decrypt it and return edits.
- Verify offline retry and account sign-out with the real provider.
- Email confirmation's production landing URL is not configured; the dashboard Site URL remains the default localhost value.
- Automatic background scheduling is not claimed: current account sync is manual.
- Concurrent revision collisions, timestamp-based incremental cursor ordering, and concurrent vault enrollment need dedicated multi-device validation.

Mock HTTP tests and database isolation tests do not prove successful live OAuth token exchange or two-device synchronization. Do not label this deployment fully verified or publish a stable sync release on that basis alone.
