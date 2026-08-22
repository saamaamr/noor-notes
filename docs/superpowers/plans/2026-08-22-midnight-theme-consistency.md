# Midnight Theme Consistency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every Noor Notes surface and interaction state readable and consistent in Snow and Midnight without rebuilding widgets or changing application functionality.

**Architecture:** A typed semantic palette renders the active `@nn_*` GTK color declarations. A single display-level runtime combines those declarations with shared Atomic component CSS and reloads the same provider when `AppearanceManager` changes effective theme. Shared popover primitives and contrast tests prevent future nested-surface regressions.

**Tech Stack:** Rust 2024, GTK4, libadwaita, GtkSourceView5, GTK CSS, existing Cargo test infrastructure.

**Spec:** `docs/superpowers/specs/2026-08-22-midnight-theme-consistency.md`

## Global Constraints

- Preserve every existing feature, storage format, note, editor mode, shortcut, sticky behavior, and source syntax palette.
- Use only Snow and Midnight as effective themes while preserving historical preference aliases.
- Add no dependency or external CSS framework.
- Reuse the current symbolic icon system and Atomic GTK utilities.
- Theme switching must not rebuild the editor or execute database/filesystem work.
- Production UI must not expose Theme Contrast Test.
- Existing dirty screenshot, PDF, JSON, and snap files in the primary checkout must remain untouched.

---

### Task 1: Semantic Palette and Contrast Contract

**Files:**
- Create: `apps/noor-notes/src/appearance/palette.rs`
- Modify: `apps/noor-notes/src/appearance/mod.rs`
- Create: `apps/noor-notes/tests/theme_contrast.rs`

**Interfaces:**
- Produces: `ThemePalette::for_theme(EffectiveTheme)`, `ThemePalette::gtk_css()`, and public semantic color fields consumed by the style runtime and tests.

- [ ] **Step 1: Write the failing contrast tests**

Add literal WCAG expectations for primary/popover/menu/editor text (4.5:1 minimum), secondary text (4.5:1), muted text (3.0:1), accent on soft accent, selection text, danger, and disabled text in both palettes. Assert that `gtk_css()` defines the complete shared `@nn_*` contract.

- [ ] **Step 2: Verify RED**

Run `cargo test -p noor-notes --test theme_contrast`; expect compilation failure because `appearance::ThemePalette` does not exist.

- [ ] **Step 3: Implement the two typed palettes**

Add the approved Snow/Midnight literals, a deterministic renderer for semantic GTK declarations, and export `ThemePalette` from `appearance`.

- [ ] **Step 4: Verify GREEN**

Run `cargo test -p noor-notes --test theme_contrast`; expect all contrast and contract tests to pass.

### Task 2: Single Live GTK Style Runtime

**Files:**
- Create: `apps/noor-notes/src/appearance/style_runtime.rs`
- Modify: `apps/noor-notes/src/appearance/mod.rs`
- Modify: `apps/noor-notes/src/appearance/manager.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/app.rs`
- Create: `apps/noor-notes/tests/theme_stylesheet.rs`

**Interfaces:**
- Consumes: `ThemePalette::gtk_css()`.
- Produces: `ThemeStyleRuntime::new()`, `install(display, theme)`, `apply(theme)`, and `AppearanceManager::install_styles(display)`.

- [ ] **Step 1: Write failing stylesheet/runtime tests**

Assert that generated Snow and Midnight stylesheets contain the same semantic aliases with different literal values, include shared component CSS exactly once, track the last applied theme, and skip redundant reload decisions.

- [ ] **Step 2: Verify RED**

Run `cargo test -p noor-notes --test theme_stylesheet`; expect compilation failure because the runtime API does not exist.

- [ ] **Step 3: Implement and wire the runtime**

Own one `gtk::CssProvider`, install it once for the display, reload active palette plus base CSS on effective-theme changes, and replace both duplicated `load_css` implementations. Guard same-theme reapplication while preserving window classes, libadwaita scheme, preference persistence, and listener ordering.

- [ ] **Step 4: Verify GREEN**

Run `cargo test -p noor-notes --test theme_stylesheet --test appearance_preferences`; expect pass.

### Task 3: Theme-Neutral Atomic Component CSS

**Files:**
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/tests/dark_palette_css.rs`

**Interfaces:**
- Consumes: active `@nn_*` aliases from the runtime.
- Produces: reusable theme-neutral surface, control, menu, dropdown, selection, settings, sticky, status, and editor rules.

- [ ] **Step 1: Replace source-presence assertions with behavioral CSS contract tests**

Parse component declarations and fail when reusable chrome references `nn_snow_*`, `nn_midnight_*`, or direct theme-dependent foreground/background literals. Assert semantic rules for popover contents, model buttons, dropdown rows, toolbar states, selection, settings, sticky, status, and disabled controls.

- [ ] **Step 2: Verify RED**

Run `cargo test -p noor-notes --test design_system --test dark_palette_css`; expect failures from global Snow aliases and missing nested popover/dropdown rules.

- [ ] **Step 3: Refactor the shared Atomic CSS**

Remove palette declarations from the component stylesheet, replace scattered Midnight chrome overrides with semantic aliases, add complete GTK node coverage, retain only explicit data-swatch/paper variants, and keep the curated stylesheet below the existing size guard.

- [ ] **Step 4: Verify GREEN**

Run the same tests; expect all pure assertions to pass. Run the GTK CSS parser test under the supported display setup when available.

### Task 4: Shared Popover Primitive and Complete Surface Adoption

**Files:**
- Create: `apps/noor-notes/src/ui/popover_primitives.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/formatting_popover.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/ui/editor_menu_bar.rs`
- Modify: `apps/noor-notes/src/ui/app_header.rs`
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/src/ui/note_writing_assistance.rs`
- Modify: `apps/noor-notes/src/writing_assistance/issue_popover.rs`
- Modify: `apps/noor-notes/src/writing_assistance/prediction_overlay.rs`
- Create: `apps/noor-notes/tests/popover_theme_contract.rs`

**Interfaces:**
- Produces: `themed_popover(child)` and `style_popover(popover)` used by every application-owned GTK popover.

- [ ] **Step 1: Write failing real-widget coverage tests**

Instantiate toolbar, formatting, menu bar, note actions, writing assistance, issue, and prediction-related popovers where public construction permits; assert the shared semantic surface class and focus/autohide behavior. Cover remaining internal popovers through their public owner widgets rather than source-text grep.

- [ ] **Step 2: Verify RED**

Run the test under the supported GTK display setup; expect uncovered Emoji, More, Note Settings, Writing Assistance, issue, and prediction popovers to fail the class contract.

- [ ] **Step 3: Adopt the shared primitive**

Replace direct popover construction or immediately style existing popovers. Preserve callbacks, selection, keyboard Escape handling, and existing menu behavior.

- [ ] **Step 4: Verify GREEN**

Run popover, accessibility, toolbar, emoji, and writing-assistance UI tests; expect pass or separately record only display-initialization failures.

### Task 5: Development Theme Contrast Test Action

**Files:**
- Modify: `apps/noor-notes/src/managed_app.rs`
- Modify: `apps/noor-notes/src/ui/app_header.rs`
- Create: `apps/noor-notes/tests/theme_contrast_action.rs`

**Interfaces:**
- Consumes: `AppearanceManager::toggle_theme()`.
- Produces: development-only `app.theme-contrast-test` that cycles the live application theme; no production action or menu row.

- [ ] **Step 1: Write failing feature-gated tests**

Assert that a development build registers and exposes `Theme Contrast Test`, and a default build neither registers nor lists it. Verify two activations produce Snow → Midnight → Snow through the real manager.

- [ ] **Step 2: Verify RED**

Run the targeted test with and without `--features development`; expect missing-action failures in development mode.

- [ ] **Step 3: Implement the gated live action**

Register the action and menu item only with `cfg(feature = "development")`; call the existing manager rather than duplicating theme state.

- [ ] **Step 4: Verify GREEN**

Run both feature variants; expect the action contract to pass.

### Task 6: Full Verification, Live Dev Rebuild, and Main Integration

**Files:**
- Modify only files required by failures attributable to this change.

**Interfaces:**
- Produces: verified feature commit ready for merge and installation.

- [ ] **Step 1: Run formatting and static verification**

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo check -p noor-notes --features development`, and `git diff --check`.

- [ ] **Step 2: Run GTK-specific verification**

Run the supported Xvfb/GTK tests. If GTK initialization is blocked by the sandbox, preserve the exact output and keep the pure semantic/contrast suite as the non-environment proof.

- [ ] **Step 3: Manually exercise the real development UI**

Build Noor Notes Dev, switch Snow → Midnight → Snow, and inspect integrated editor menus, Formatting, font size, Emoji, More, sort, main menu, note actions, settings, source modes, sticky, selection, focus, hover, active, and disabled states.

- [ ] **Step 4: Commit, merge, push, and rebuild**

Commit only theme implementation/tests/docs, merge the isolated branch into current `main` without staging the primary checkout's screenshot artifacts, push `main`, verify local/remote hashes, and run `scripts/install-local.sh` to install the rebuilt Noor Notes Dev.
