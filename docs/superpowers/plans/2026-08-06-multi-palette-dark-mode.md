# Multi-Palette Dark Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add synchronized System, Light, Graphite, Midnight, and OLED appearance modes with theme-aware symbolic icon colors across every Noor Notes window.

**Architecture:** A pure appearance model validates and resolves preferences, while an `AppearanceManager` persists them atomically and applies one effective root CSS class to registered windows. Header, menu, and settings controls dispatch the same application action, so their state cannot diverge. Explicit GTK palette selectors replace the ineffective browser-style custom properties.

**Tech Stack:** Rust, GTK4 0.10, libadwaita 0.8, serde/serde_json, native symbolic icons, GTK CSS.

## Global Constraints

- Preserve the application ID, encrypted notes database, existing notes, and keyboard shortcuts.
- Store appearance preferences outside the notes database with atomic replacement.
- Add no analytics, telemetry, remote assets, network requests, or non-native icon system.
- Do not build, upload, release, reject, or modify a Snap Store revision.
- Keep the two existing untracked Snap artifacts unstaged.
- Graphite is the default preferred dark palette.
- Theme changes must update all open application windows immediately.
- High-contrast and reduced-motion desktop settings remain authoritative.

---

### Task 1: Pure Appearance Model and Atomic Preferences

**Files:**
- Create: `apps/noor-notes/src/appearance/model.rs`
- Create: `apps/noor-notes/src/appearance/preferences.rs`
- Create: `apps/noor-notes/src/appearance/mod.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Test: `apps/noor-notes/tests/appearance_preferences.rs`

**Interfaces:**
- Produces: `AppearanceMode::{System, Light, Graphite, Midnight, Oled}`, `DarkPalette::{Graphite, Midnight, Oled}`, `SystemScheme::{Light, Dark}`, `EffectiveTheme`, and `AppearancePreferences`.
- Produces: `AppearancePreferences::resolve(SystemScheme) -> EffectiveTheme`, `AppearanceStore::load() -> AppearancePreferences`, and `AppearanceStore::save(&AppearancePreferences) -> io::Result<()>`.

- [ ] Write tests proving defaults are System plus Graphite, all values round-trip, invalid JSON fails closed, and System resolves through the preferred dark palette.
- [ ] Run `cargo test -p noor-notes --test appearance_preferences` and confirm the missing-module failure.
- [ ] Implement serde-backed enums with stable kebab-case values and explicit fallback defaults.
- [ ] Implement owner-private atomic persistence using a sibling temporary file, flush, permission hardening, and rename.
- [ ] Rerun the focused test and `cargo clippy -p noor-notes --test appearance_preferences -- -D warnings`.
- [ ] Commit `feat: add appearance preferences`.

### Task 2: Application-Wide Appearance Manager

**Files:**
- Create: `apps/noor-notes/src/appearance/manager.rs`
- Modify: `apps/noor-notes/src/appearance/mod.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/app.rs`
- Test: `apps/noor-notes/tests/appearance_manager.rs`

**Interfaces:**
- Consumes: Task 1 appearance types and store.
- Produces: `AppearanceManager::new(store, style_manager)`, `register_window(&impl IsA<gtk::Window>)`, `set_mode(AppearanceMode)`, `cycle_dark_palette()`, `preferences()`, and `subscribe()`.
- Invariant: every registered window has exactly one of `nn-theme-light`, `nn-theme-graphite`, `nn-theme-midnight`, or `nn-theme-oled`.

- [ ] Write GTK tests for one effective class, multiple-window propagation, dark-palette cycling, System scheme changes, and persistence failure state.
- [ ] Run the focused test under `xvfb-run` and confirm failure before implementation.
- [ ] Implement shared `Rc<RefCell<_>>` state on the GTK main thread, weak window registration, stale-window cleanup, and a watch callback for controls.
- [ ] Map effective themes to libadwaita light/dark color schemes and apply root classes without changing note color classes.
- [ ] Show a non-destructive toast or dialog when persistence fails while retaining the session selection.
- [ ] Rerun focused tests and strict Clippy.
- [ ] Commit `feat: add application appearance manager`.

### Task 3: Explicit Dark Palettes and Adaptive Icon Colors

**Files:**
- Replace dark section in: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Create: `apps/noor-notes/tests/dark_palette_css.rs`

**Interfaces:**
- Consumes: root classes from Task 2.
- Produces: explicit component selectors for all four effective themes and semantic icon classes `nn-icon-neutral`, `nn-icon-secondary`, `nn-icon-active`, `nn-icon-success`, `nn-icon-warning`, and `nn-icon-destructive`.

- [ ] Write failing source and GTK parser tests requiring each palette to style windows, header bars, sidebars, cards, preview, canvas, entries, popovers, menus, dialogs, status bars, selections, focus rings, disabled controls, and symbolic icons.
- [ ] Add contrast tests for primary/secondary text, accent selection, error, and all dark paper foreground pairs.
- [ ] Remove the ineffective `--nn-*` declarations and legacy-only dark overrides.
- [ ] Implement Graphite with warm charcoal layers and restrained indigo; Midnight with blue-black layers and sky blue; OLED with near-black layers and accessible violet-blue.
- [ ] Add palette-scoped symbolic icon color rules: neutral follows foreground, active follows accent, disabled remains legible, and destructive remains neutral until destructive interaction.
- [ ] Add dark variants for Warm White, Cream, Yellow, Blue, Green, Pink, Purple, and Dark Slate paper colors, including link, caret, selection, and highlight styling.
- [ ] Run `xvfb-run -a cargo test -p noor-notes --test design_system --test dark_palette_css`.
- [ ] Commit `feat: add premium dark palettes`.

### Task 4: Synchronized Header and Main-Menu Controls

**Files:**
- Create: `apps/noor-notes/src/ui/appearance_button.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Test: `apps/noor-notes/tests/appearance_controls.rs`

**Interfaces:**
- Consumes: `AppearanceManager`.
- Produces: application action `app.appearance` with string targets `system`, `light`, `graphite`, `midnight`, and `oled`; produces `AppearanceButton` for cycling dark palettes.

- [ ] Write failing tests for action targets, active menu state, header-cycle order, tooltip text, accessible labels, keyboard focus, and immediate cross-window synchronization.
- [ ] Register one stateful application action and use it for every direct mode choice.
- [ ] Add the compact palette button to both library and editor headers without displacing native window controls.
- [ ] Add a labeled Appearance submenu to the main menu with radio-state feedback and shortcut hints where appropriate.
- [ ] Update symbolic icon color classes and tooltip text whenever the effective palette changes.
- [ ] Rerun focused UI tests under Xvfb and strict Clippy.
- [ ] Commit `feat: add synchronized appearance controls`.

### Task 5: Native Appearance Settings Window

**Files:**
- Create: `apps/noor-notes/src/ui/appearance_settings.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Test: `apps/noor-notes/tests/appearance_settings.rs`

**Interfaces:**
- Consumes: the same stateful `app.appearance` action and `AppearanceManager`.
- Produces: a reusable libadwaita preferences window containing live palette preview rows.

- [ ] Write failing tests for five labeled choices, three dark previews, selected-state synchronization, accessible descriptions, Escape close, and 200% scaling-safe layout.
- [ ] Build an `adw::PreferencesWindow` with an Appearance page, Theme group, five action rows, and compact palette swatches using semantic CSS classes.
- [ ] Explain that System follows GNOME while retaining the preferred dark palette.
- [ ] Add Settings to the main menu and ensure opening it reuses/presents one window rather than creating duplicates.
- [ ] Verify header, menu, and settings changes remain synchronized in both directions.
- [ ] Run focused tests under Xvfb and commit `feat: add appearance settings`.

### Task 6: Regression, Installation, and Visual Verification

**Files:**
- Modify: `README.md`
- Modify focused tests if a verified accessibility or visual defect is found.
- Create a temporary screenshot harness and remove it before commit.
- Output screenshots only to `/tmp/noor-notes-dark-palettes/`.

**Interfaces:**
- Produces final verification evidence; no production interface.

- [ ] Add README instructions for System, Light, Graphite, Midnight, OLED, palette cycling, and adaptive icons.
- [ ] Run keyboard-only checks for header, menu, Settings, Escape, and focus order.
- [ ] Verify theme changes across a library window, two editor windows, popovers, and dialogs.
- [ ] Verify rich note formatting and existing notes remain accessible without a database migration.
- [ ] Capture fresh PNGs for Graphite library/editor, Midnight library/editor, OLED library/editor, Appearance menu, Settings, dark paper colors, and light regression.
- [ ] Remove the temporary harness and confirm screenshots, logs, databases, credentials, binaries, and Snap artifacts are not staged.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test --workspace`.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo build --release`, installer contract tests, and `git diff --check`.
- [ ] Install the verified user-local binary and launch it without altering note data.
- [ ] Commit `docs: document multi-palette appearance`.
- [ ] Push only after the user-approved workflow and all verification commands succeed.

