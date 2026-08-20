# Professional UI Phase 3: Sticky, Themes, Accessibility, and Product Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the approved product redesign across sticky/read-only windows, settings, dialogs, all themes, accessibility, responsive behavior, and final regression/manual verification without changing persistence or native window semantics.

**Architecture:** Reuse the Phase 1 semantic tokens and Phase 2 document components. `StickyNoteWindow` remains a lightweight read-only host over `NoteEditorSurface`; `AppearanceManager` remains the single theme authority; existing settings/dialog controllers retain behavior and receive shared presentation primitives. Final tests verify cross-window state synchronization and theme-safe interaction contracts.

**Tech Stack:** Rust 1.85, GTK4, Libadwaita, existing `noor-windowing`, existing appearance store/manager, Xvfb GTK tests, and the current local install script.

**Spec:** `docs/superpowers/specs/2026-08-20-professional-product-ui-redesign-design.md`

## Global Constraints

- Sticky body shows note content only; note title appears once in the compact native top bar.
- Always on Top must continue through `WindowController`; do not bypass native capability checks.
- Keep Snow, Warm Paper, Cool Mist, Graphite, Midnight, and OLED backed by one semantic component architecture.
- Keep every secondary window registered with `AppearanceManager`.
- Do not weaken focus indicators, accessible names, destructive confirmations, or reduced-motion handling.
- Manual verification must use a disposable profile/database, never the user's real notes.
- Do not modify or stage unrelated worktree changes.

---

### Task 1: Replace the Sticky Window Presentation and Synchronize Its State

**Files:**
- Modify: `apps/noor-notes/src/sticky_note_window.rs`
- Modify: `apps/noor-notes/src/ui/note_editor_surface.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/tests/sticky_note_window.rs`
- Modify: `apps/noor-notes/tests/note_preview_edit.rs`

**Interfaces:**
- Consumes: selected `Note`, `WindowController` capabilities, sticky close callback, and main preview read-only state.
- Produces: compact title bar, body-only read surface, semantic Always on Top state, and reliable main-window state reset after close.

- [ ] **Step 1: Write failing sticky presentation/state tests**

```rust
#[test]
fn sticky_window_has_one_title_and_a_body_only_document_surface() {
    let sticky = fixture_sticky_window();
    assert!(sticky.window.has_css_class("nn-sticky-note-window"));
    assert_eq!(descendants_with_class(&sticky.window, "nn-document-title").len(), 0);
    assert_eq!(descendants_with_class(&sticky.window, "nn-sticky-body").len(), 1);
    assert_eq!(sticky.always_on_top.tooltip_text().as_deref(), Some("Keep this note always on top"));
}

#[test]
fn closing_sticky_resets_the_main_read_only_action() {
    let preview = fixture_preview();
    preview.open_read_only();
    preview.close_sticky_for_test();
    assert_eq!(preview.read_only_label(), "Open read-only");
    assert!(!preview.is_sticky_open());
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test sticky_note_window --test note_preview_edit closing_sticky -- --nocapture
```

- [ ] **Step 3: Recompose sticky content**

Use a compact `adw::HeaderBar`, `adw::WindowTitle` with note title only, an icon/toggle control for Always on Top, and a read-only `NoteEditorSurface` body class. Remove duplicated title and edited metadata from the body. Apply the note color as a restrained accent class, not a saturated full surface.

- [ ] **Step 4: Preserve native capability and close behavior**

Keep `WindowController::set_above` routing and X11/Wayland native ID logic. Unsupported desktops disable the toggle with explanatory text. Toggle failures restore the previous state and keep the window usable. Invoke the existing close callback exactly once.

- [ ] **Step 5: Run sticky and preview tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test sticky_note_window --test note_preview_edit
```

- [ ] **Step 6: Commit sticky/read-only polish**

```bash
git add apps/noor-notes/src/sticky_note_window.rs apps/noor-notes/src/ui/note_editor_surface.rs apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/tests/sticky_note_window.rs apps/noor-notes/tests/note_preview_edit.rs
git commit -m "ui: polish sticky reading and synchronize its state"
```

### Task 2: Make Every Theme Override the Complete Semantic Palette

**Files:**
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/src/appearance/model.rs`
- Modify: `apps/noor-notes/src/appearance/manager.rs`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/tests/appearance_manager.rs`
- Modify: `apps/noor-notes/tests/source_palettes.rs`

**Interfaces:**
- Consumes: `EffectiveTheme`, Phase 1 semantic roles, source-mode palette isolation.
- Produces: complete theme-safe values for every shared component and correct live theme application across all registered windows.

- [ ] **Step 1: Write failing complete-palette tests**

```rust
#[test]
fn every_effective_theme_overrides_required_semantic_roles() {
    for class in EffectiveTheme::ALL_CLASSES {
        let block = theme_block(CSS, class);
        for role in [
            "nn_app_bg", "nn_sidebar_bg", "nn_list_bg", "nn_editor_bg",
            "nn_surface", "nn_surface_raised", "nn_popover_bg", "nn_input_bg",
            "nn_text_primary", "nn_text_secondary", "nn_text_muted",
            "nn_border", "nn_border_strong", "nn_accent", "nn_accent_soft",
            "nn_focus", "nn_danger", "nn_danger_soft",
        ] {
            assert!(block.contains(role), "{class} missing {role}");
        }
    }
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test design_system every_effective_theme -- --nocapture
```

- [ ] **Step 3: Complete semantic theme blocks**

Define each role once per theme and remove shared-component literal colors. Preserve source canvas palettes in `editor/source_palette.rs`; source syntax colors do not inherit document-surface colors blindly.

- [ ] **Step 4: Verify live registered-window updates**

Expand manager tests to register main, sticky, preferences, and secondary test windows, switch Snow → Graphite → OLED → Snow, and assert exactly one effective theme class on each.

- [ ] **Step 5: Run appearance and palette tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test design_system --test appearance_manager --test source_palettes
```

- [ ] **Step 6: Commit cross-theme completeness**

```bash
git add apps/noor-notes/resources/design-system.css apps/noor-notes/src/appearance/model.rs apps/noor-notes/src/appearance/manager.rs apps/noor-notes/tests/design_system.rs apps/noor-notes/tests/appearance_manager.rs apps/noor-notes/tests/source_palettes.rs
git commit -m "ui: complete semantic palettes across all themes"
```

### Task 3: Professionalize Appearance and Writing Settings

**Files:**
- Modify: `apps/noor-notes/src/ui/appearance_settings.rs`
- Modify: `apps/noor-notes/src/ui/writing_assistance_settings.rs`
- Modify: `apps/noor-notes/src/ui/appearance_button.rs`
- Modify: `apps/noor-notes/tests/appearance_controls.rs`
- Modify: `apps/noor-notes/tests/appearance_preferences.rs`

**Interfaces:**
- Consumes: existing appearance actions/store and writing-assistance preferences.
- Produces: consistent preference groups, rows, swatches, validation, and live state without changing preference formats.

- [ ] **Step 1: Write failing settings-component tests**

```rust
#[test]
fn appearance_settings_use_shared_professional_rows() {
    let settings = fixture_settings();
    assert_eq!(settings.choice_count(), 7);
    assert!(settings.window.has_css_class("nn-settings-window"));
    for row in settings.choice_rows() {
        assert!(row.activatable_widget().is_some());
        assert!(!row.title().is_empty());
        assert!(!row.subtitle().is_empty());
    }
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test appearance_controls appearance_settings_use -- --nocapture
```

- [ ] **Step 3: Recompose settings with shared rows**

Keep the seven real appearance choices and current writing-assistance options. Use consistent page/group titles, descriptions, suffix controls, focus order, live selected state, and validation feedback. Theme swatches must use theme classes/tokens, not independent color literals.

- [ ] **Step 4: Register and synchronize secondary windows**

Ensure every settings window registers with `AppearanceManager`, updates while open, and persists through the existing stores.

- [ ] **Step 5: Run settings tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test appearance_controls --test appearance_preferences
```

- [ ] **Step 6: Commit settings polish**

```bash
git add apps/noor-notes/src/ui/appearance_settings.rs apps/noor-notes/src/ui/writing_assistance_settings.rs apps/noor-notes/src/ui/appearance_button.rs apps/noor-notes/tests/appearance_controls.rs apps/noor-notes/tests/appearance_preferences.rs
git commit -m "ui: unify appearance and writing settings"
```

### Task 4: Unify Dialogs, Confirmations, and Secondary Windows

**Files:**
- Create: `apps/noor-notes/src/ui/dialog_primitives.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/import_dialog.rs`
- Modify: `apps/noor-notes/src/services/trash_command.rs`
- Modify: `apps/noor-notes/tests/import_flow.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`
- Modify: `apps/noor-notes/tests/note_actions.rs`

**Interfaces:**
- Consumes: existing rename, Go to Line, import, export, mode-conversion, move-to-trash, and permanent-delete flows.
- Produces: common dialog spacing, button order, danger styling, theme registration, focus restoration, and error presentation.

- [ ] **Step 1: Write failing dialog-contract tests**

```rust
#[test]
fn destructive_confirmations_use_one_explicit_danger_contract() {
    assert!(TRASH_COMMAND.contains("dialog_primitives::confirm_destructive"));
    assert!(TRASH_COMMAND.contains("Move to Trash"));
    assert!(NOTE_ACTIONS.contains("Permanently Delete"));
}

#[test]
fn modal_actions_release_menu_grabs_before_presenting() {
    assert!(NOTE_WINDOW.contains("popdown_before_dialog"));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test toolbar_actions destructive_confirmations -- --nocapture
```

- [ ] **Step 3: Add shared dialog primitives**

Provide normal confirmation, destructive confirmation, text-entry, and focused error helpers around current Libadwaita dialogs. Keep cancel/secondary/primary ordering, explicit irreversible copy, destructive appearance, parent/transient relationship, and focus restoration.

- [ ] **Step 4: Migrate existing flows without changing outcomes**

Move rename, Go to Line, import, mode conversion, trash, permanent delete, and export failure presentation onto the primitives. Do not change repository calls, confirmation requirements, or conversion recovery behavior.

- [ ] **Step 5: Run flow tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test import_flow --test toolbar_actions --test note_actions
```

- [ ] **Step 6: Commit common secondary surfaces**

```bash
git add apps/noor-notes/src/ui/dialog_primitives.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/note_window.rs apps/noor-notes/src/import_dialog.rs apps/noor-notes/src/services/trash_command.rs apps/noor-notes/tests/import_flow.rs apps/noor-notes/tests/toolbar_actions.rs apps/noor-notes/tests/note_actions.rs
git commit -m "ui: unify dialogs and destructive confirmations"
```

### Task 5: Enforce Accessibility and Reduced-Motion Contracts

**Files:**
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/src/ui/library_sidebar.rs`
- Modify: `apps/noor-notes/src/ui/note_collection.rs`
- Modify: `apps/noor-notes/src/ui/editor_menu_bar.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Create: `apps/noor-notes/tests/accessibility.rs`
- Modify: `apps/noor-notes/tests/library_ui.rs`

**Interfaces:**
- Consumes: all icon controls, navigation rows, note cards, editor menus/popovers, selected/read-only/always-on-top state.
- Produces: accessible names, semantic selected/checked state, visible focus, logical tab order, Escape behavior, 200% scaling safety, and reduced-motion CSS.

- [ ] **Step 1: Write failing accessibility tests**

```rust
#[test]
fn all_icon_only_controls_have_names_and_tooltips() {
    gtk::init().unwrap();
    let shell = fixture_shell();
    for control in icon_only_buttons(&shell) {
        assert!(accessible_name(&control).is_some(), "missing accessible name");
        assert!(control.tooltip_text().is_some(), "missing tooltip");
    }
}

#[test]
fn reduced_motion_rule_disables_nonessential_transitions() {
    assert!(CSS.contains("@media (prefers-reduced-motion: reduce)"));
    assert!(CSS.contains("transition: none"));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test accessibility -- --nocapture
```

- [ ] **Step 3: Apply semantic accessibility states**

Expose selected note/navigation state and checked toggles through GTK accessibility properties in addition to color. Give every unfamiliar icon an accessible name/tooltip. Ensure destructive and disabled controls retain readable contrast.

- [ ] **Step 4: Verify focus, Escape, scaling, and motion**

Add a visible focus contract across all themes. Menus/popovers/dialogs close with Escape and restore logical focus. Test controls and labels at 200% scaling/large text for clipping. Disable nonessential transitions for reduced motion.

- [ ] **Step 5: Run accessibility and library tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test accessibility --test library_ui
```

- [ ] **Step 6: Commit accessibility contracts**

```bash
git add apps/noor-notes/resources/design-system.css apps/noor-notes/src/ui/library_sidebar.rs apps/noor-notes/src/ui/note_collection.rs apps/noor-notes/src/ui/editor_menu_bar.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/tests/accessibility.rs apps/noor-notes/tests/library_ui.rs
git commit -m "ui: enforce accessible focus and motion contracts"
```

### Task 6: Verify Real Responsive Widget Allocations

**Files:**
- Modify: `apps/noor-notes/src/ui/adaptive_layout.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/tests/adaptive_layout.rs`
- Modify: `apps/noor-notes/tests/editor_presentation.rs`

**Interfaces:**
- Consumes: Phase 1 ratio policy and Phase 2 toolbar priorities.
- Produces: real wide/medium/narrow GTK allocations with no preview sliver, header collision, or toolbar wrap.

- [ ] **Step 1: Add failing real-allocation tests**

```rust
#[test]
fn real_shell_respects_dynamic_wide_ratios_and_guards() {
    let shell = allocated_shell(1440, 900);
    assert!((160..=220).contains(&shell.sidebar.allocated_width()));
    assert!((280..=360).contains(&shell.collection.allocated_width()));
    assert!(shell.document.allocated_width() > shell.collection.allocated_width());
}

#[test]
fn narrow_document_has_back_navigation_and_no_hidden_preview_sliver() {
    let shell = allocated_document(620, 760);
    assert!(shell.back.is_visible());
    assert_eq!(shell.navigation.allocated_width(), 0);
    assert!(shell.document.allocated_width() >= 600);
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test adaptive_layout real_shell -- --nocapture
```

- [ ] **Step 3: Fix allocation integration, not the pure ratio policy**

Recalculate from actual allocated width, clamp to safety guards, and switch destination state at existing breakpoints. Keep sidebar about 10%, notes about 20%, editor remainder on wide layouts. Compact header/toolbar controls by priority instead of wrapping.

- [ ] **Step 4: Run responsive tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test adaptive_layout --test editor_presentation
```

- [ ] **Step 5: Commit responsive verification fixes**

```bash
git add apps/noor-notes/src/ui/adaptive_layout.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/tests/adaptive_layout.rs apps/noor-notes/tests/editor_presentation.rs
git commit -m "ui: verify adaptive shell and editor allocations"
```

### Task 7: Run Full Automated and Disposable-Profile Manual Verification

**Files:**
- Modify only defects found in their owning source/test files.
- Do not modify the user's screenshot PDF or Snap artifacts.

- [ ] **Step 1: Run formatter, check, and strict Clippy**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo check --workspace
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets -- -D warnings
```

- [ ] **Step 2: Run all workspace tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test --workspace
```

If an individual non-GTK test is incompatible with Xvfb wrapping, run that test normally and report the environment distinction. Do not modify production code to bypass display initialization.

- [ ] **Step 3: Build the locked release artifact**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo build --locked --release -p noor-notes
```

- [ ] **Step 4: Install the dev build through the existing script**

```bash
PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh
```

- [ ] **Step 5: Verify with an isolated user-data profile**

Launch the installed development build with an isolated XDG data/config/cache directory under a fresh `mktemp -d` directory. Create disposable Rich, Markdown, Plain Text, and Code notes and verify:

- Wide, medium, narrow-list, narrow-document, and sticky layouts.
- Snow, Warm Paper, Cool Mist, Graphite, Midnight, OLED, then back to Snow.
- Search, clear search, all four sort modes, all library sections, and empty states.
- Create, title edit, body edit, autosave, duplicate, archive, restore, trash, permanent-delete confirmation.
- Undo/redo, Bold/Italic/Underline/Strikethrough, sizes, alignment, colors, highlight, lists, clear formatting, emoji, Find/Replace, zoom, word wrap, and mode conversion safeguards.
- Read-only open/close synchronization and Always on Top supported/unsupported behavior.
- Long title, long URL/token, keyboard focus, Escape, and 200% scaling.

- [ ] **Step 6: Verify repository hygiene**

```bash
git diff --check
git status --short
```

Expected: all automated and manual checks pass; the user's existing screenshot deletions, reference PDF, and Snap files remain untouched and unstaged.
