# Noor Notes Two-Theme Product Redesign Implementation Plan

> **For implementation:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with review checkpoints. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate Noor Notes into a polished Snow/Midnight product while preserving the current integrated editor, all real functionality, encrypted persistence, autosave, lifecycle behavior, and the updated 81-state baseline.

**Architecture:** Keep `MainWindow -> integrated NotePreview`, editor commands/adapters, `RichDocument`, `AutosaveQueue`, `SqliteNoteRepository`, and window-controller boundaries. Add a backward-compatible `System/Snow/Midnight` preference resolving to two effective themes, consolidate presentation through curated GTK Atomic CSS utilities plus focused state classes, and make only evidence-backed adaptive/performance changes.

**Tech Stack:** Rust 2024, GTK4 0.10, Libadwaita 0.8, GtkSourceView 5, Tokio, SQLx/SQLCipher, Serde, Cargo tests, Xvfb.

**Spec:** `docs/superpowers/specs/2026-08-22-noor-notes-two-theme-product-redesign-design.md`

## Global Constraints

- Rust stays version 1.85 with workspace edition 2024.
- Add no icon library, CSS generator, JavaScript pipeline, UI framework, editor engine, dependency, storage mechanism, or database migration.
- Do not change note IDs, encrypted payloads, database schema, keyring behavior, storage paths, `RichDocument`, or note serialization.
- Primary editor stays `MainWindow -> integrated NotePreview`; do not restore legacy `NoteWindow` as the integrated surface.
- Preserve all library, lifecycle, search, sort, import, shortcut, writing-assistance, sticky, Rich Text, Markdown, Plain Text, and Code capabilities.
- Do not expose legacy-only File/View/Tools/More/Find controls as integrated features.
- Every visible control calls a real command or opens a functional surface.
- Editor body stays 16 pixels; metadata/status stays 12-13 pixels.
- Wide layout stays dynamic: sidebar about 10%, notes about 18-20%, editor gets the remainder with readability clamps.
- Existing dirty files and screenshot deletions in the original `main` checkout are never reset, cleaned, or accidentally staged.
- Every production behavior change follows Red-Green-Refactor.

---

### Task 1: Preserve Current Source-Editor Undo Behavior

**Files:**
- Modify: `apps/noor-notes/tests/rich_editor.rs`
- Modify: `apps/noor-notes/src/editor/source_adapter.rs`

**Interfaces:**
- Consumes: `SourceEditorAdapter::new`, `SourceEditorAdapter::buffer`, and `RichBuffer::{prepare,can_undo,undo}`.
- Produces: source buffers using the same GTK history bridge as integrated toolbar/menu/shortcut Undo.

- [ ] **Step 1: Create and verify an isolated worktree**

```bash
git check-ignore -q .worktrees
git worktree add .worktrees/two-theme-product-redesign -b feature/two-theme-product-redesign
cd .worktrees/two-theme-product-redesign
cargo test --workspace
```

Expected: committed `main` baseline passes. If `.worktrees` is not ignored, add only `/.worktrees/` to `.gitignore`, commit that safety rule, then create the worktree.

- [ ] **Step 2: Write the failing source-history test**

```rust
#[test]
fn source_editor_typing_participates_in_shared_undo_history() {
    gtk::init().unwrap();
    let language = SourceLanguage::new("rust").unwrap();
    let editor = SourceEditorAdapter::new("fn main() {}", &language);
    let buffer: gtk::TextBuffer = editor.buffer().clone().upcast();
    assert!(!RichBuffer::can_undo(&buffer));
    buffer.insert_at_cursor("// Noor Notes\n");
    assert!(RichBuffer::can_undo(&buffer));
    RichBuffer::undo(&buffer);
    assert_eq!(buffer.text(&buffer.start_iter(), &buffer.end_iter(), true).as_str(), "fn main() {}");
}
```

- [ ] **Step 3: Verify RED**

Run: `xvfb-run -a cargo test -p noor-notes --test rich_editor source_editor_typing_participates_in_shared_undo_history -- --exact`

Expected: FAIL because the source buffer has not joined `RichBuffer` history.

- [ ] **Step 4: Implement the minimum history preparation**

Add immediately after `sourceview5::Buffer` construction:

```rust
crate::rich_buffer::RichBuffer::prepare(&buffer.clone().upcast::<gtk::TextBuffer>());
```

Do not add another undo stack.

- [ ] **Step 5: Verify GREEN and commit**

```bash
xvfb-run -a cargo test -p noor-notes --test rich_editor
git diff --check
git add apps/noor-notes/src/editor/source_adapter.rs apps/noor-notes/tests/rich_editor.rs
git commit -m "fix: share undo history with source editors"
```

---

### Task 2: Canonicalize Historical Appearance Values

**Files:**
- Modify: `apps/noor-notes/src/appearance/model.rs`
- Modify: `apps/noor-notes/src/appearance/manager.rs`
- Modify: `apps/noor-notes/tests/appearance_preferences.rs`
- Modify: `apps/noor-notes/tests/appearance_manager.rs`
- Modify: `apps/noor-notes/tests/appearance_startup.rs`

**Interfaces:**
- Consumes: `AppearanceStore::load/save`, Serde, `AppearanceManager::set_mode`, and `SystemScheme`.
- Produces: `AppearanceMode::{System,Snow,Midnight}`, `EffectiveTheme::{Snow,Midnight}`, legacy aliases, and two CSS classes.

- [ ] **Step 1: Write failing migration tests**

```rust
#[test]
fn historical_modes_load_as_two_canonical_themes() {
    for (stored, mode, light, dark) in [
        ("light", AppearanceMode::Snow, EffectiveTheme::Snow, EffectiveTheme::Snow),
        ("warm-paper", AppearanceMode::Snow, EffectiveTheme::Snow, EffectiveTheme::Snow),
        ("cool-mist", AppearanceMode::Snow, EffectiveTheme::Snow, EffectiveTheme::Snow),
        ("graphite", AppearanceMode::Midnight, EffectiveTheme::Midnight, EffectiveTheme::Midnight),
        ("midnight", AppearanceMode::Midnight, EffectiveTheme::Midnight, EffectiveTheme::Midnight),
        ("oled", AppearanceMode::Midnight, EffectiveTheme::Midnight, EffectiveTheme::Midnight),
        ("system", AppearanceMode::System, EffectiveTheme::Snow, EffectiveTheme::Midnight),
    ] {
        let value = format!(r#"{{"mode":"{stored}"}}"#);
        let preferences: AppearancePreferences = serde_json::from_str(&value).unwrap();
        assert_eq!(preferences.mode, mode);
        assert_eq!(preferences.resolve(SystemScheme::Light), light);
        assert_eq!(preferences.resolve(SystemScheme::Dark), dark);
    }
}
```

Also assert canonical serialization is exactly `{"mode":"snow"}` or `{"mode":"midnight"}`, malformed files remain unchanged, and saved permissions remain `0o600`.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p noor-notes --test appearance_preferences`

Expected: compile/test failure because canonical Snow/Midnight variants do not exist.

- [ ] **Step 3: Implement the minimum compatible model**

```rust
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceMode {
    #[default]
    System,
    #[serde(alias = "light", alias = "warm-paper", alias = "cool-mist")]
    Snow,
    #[serde(alias = "graphite", alias = "oled")]
    Midnight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveTheme { Snow, Midnight }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct AppearancePreferences { pub mode: AppearanceMode }
```

`from_action_name` maps `snow|light|warm-paper|cool-mist` to Snow, `midnight|graphite|oled` to Midnight, `system` to System, and rejects unknown input.

- [ ] **Step 4: Simplify manager resolution and toggling**

```rust
pub fn toggle_theme(&self) -> io::Result<EffectiveTheme> {
    let next = match self.effective_theme() {
        EffectiveTheme::Snow => AppearanceMode::Midnight,
        EffectiveTheme::Midnight => AppearanceMode::Snow,
    };
    self.set_mode(next)?;
    Ok(self.effective_theme())
}
```

Keep native system observation only while mode is System. Apply/remove only `nn-theme-snow` and `nn-theme-midnight`.

- [ ] **Step 5: Verify GREEN and commit**

```bash
xvfb-run -a cargo test -p noor-notes --test appearance_preferences --test appearance_manager --test appearance_startup
cargo fmt --all -- --check
git diff --check
git add apps/noor-notes/src/appearance/model.rs apps/noor-notes/src/appearance/manager.rs apps/noor-notes/tests/appearance_preferences.rs apps/noor-notes/tests/appearance_manager.rs apps/noor-notes/tests/appearance_startup.rs
git commit -m "refactor: canonicalize appearance to snow and midnight"
```

---

### Task 3: Expose Only Snow and Midnight Controls

**Files:**
- Modify: `apps/noor-notes/src/ui/appearance_button.rs`
- Modify: `apps/noor-notes/src/ui/appearance_settings.rs`
- Modify: `apps/noor-notes/src/ui/app_header.rs`
- Modify: `apps/noor-notes/tests/appearance_controls.rs`
- Create: `apps/noor-notes/tests/app_menu_contract.rs`
- Modify: `apps/noor-notes/tests/accessibility.rs`

**Interfaces:**
- Consumes: Task 2 `AppearanceManager::toggle_theme`, `AppearanceMode::{Snow,Midnight}`, and `EffectiveTheme::{Snow,Midnight}`.
- Produces: one compact header toggle, two settings rows, and two application-menu theme actions.

- [ ] **Step 1: Write failing UI-contract tests**

```rust
#[test]
fn header_toggle_and_settings_expose_only_two_themes() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(directory.path().join("appearance.json")));
    manager.set_mode(AppearanceMode::Snow).unwrap();
    let control = AppearanceButton::new(manager.clone());
    assert_eq!(control.button.tooltip_text().as_deref(), Some("Switch to Midnight"));
    control.button.emit_clicked();
    assert_eq!(manager.preferences().mode, AppearanceMode::Midnight);
    assert_eq!(control.button.tooltip_text().as_deref(), Some("Switch to Snow"));

    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.AppearanceTest")
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let settings = AppearanceSettings::new(&app, manager);
    assert_eq!(settings.choice_count(), 2);
    let titles: Vec<_> = settings.choice_rows().iter().map(|row| row.title()).collect();
    assert_eq!(titles, ["Snow", "Midnight"]);
}
```

In the new `app_menu_contract.rs`, create an `AppHeader`, recursively inspect its public `main_menu.menu_model()` through `gio::MenuModelExt::{item_attribute_value,item_link}`, and assert `app.appearance::snow` and `app.appearance::midnight` exist while historical targets do not. Traverse both `gio::MENU_LINK_SECTION` and `gio::MENU_LINK_SUBMENU`; do not expose a production-only test accessor.

- [ ] **Step 2: Verify RED**

Run: `xvfb-run -a cargo test -p noor-notes --test appearance_controls --test app_menu_contract`

Expected: FAIL because seven historical choices remain.

- [ ] **Step 3: Implement the header toggle**

```rust
manager.subscribe(move |_, theme| {
    let (icon, tooltip) = match theme {
        EffectiveTheme::Snow => ("weather-clear-symbolic", "Switch to Midnight"),
        EffectiveTheme::Midnight => ("weather-clear-night-symbolic", "Switch to Snow"),
    };
    live_button.set_icon_name(icon);
    live_button.set_tooltip_text(Some(tooltip));
    live_button.update_property(&[gtk::accessible::Property::Label(tooltip)]);
});
```

Click calls `toggle_theme`; do not duplicate preference mutation.

- [ ] **Step 4: Build two settings rows and two menu actions**

```rust
[
    (AppearanceMode::Snow, "Snow", "Clean daytime theme", "nn-swatch-snow"),
    (AppearanceMode::Midnight, "Midnight", "Comfortable dark theme", "nn-swatch-midnight"),
]
```

Keep Import, Keyboard Shortcuts, Appearance Settings, Writing Assistance, and Quit unchanged.

- [ ] **Step 5: Verify GREEN and commit**

```bash
xvfb-run -a cargo test -p noor-notes --test appearance_controls --test app_menu_contract --test accessibility
git diff --check
git add apps/noor-notes/src/ui/appearance_button.rs apps/noor-notes/src/ui/appearance_settings.rs apps/noor-notes/src/ui/app_header.rs apps/noor-notes/tests/appearance_controls.rs apps/noor-notes/tests/app_menu_contract.rs apps/noor-notes/tests/accessibility.rs
git commit -m "ui: expose only snow and midnight themes"
```

---

### Task 4: Introduce Curated GTK Atomic CSS and Two Theme Layers

**Files:**
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/src/ui/app_header.rs`
- Modify: `apps/noor-notes/src/ui/library_sidebar.rs`
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/sticky_note_window.rs`
- Modify: `apps/noor-notes/src/editor/source_palette.rs`
- Modify: `apps/noor-notes/src/rich_color.rs`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/tests/dark_palette_css.rs`
- Modify: `apps/noor-notes/tests/source_palettes.rs`
- Modify: `apps/noor-notes/tests/rich_colors.rs`
- Modify: `apps/noor-notes/tests/compact_ui.rs`

**Interfaces:**
- Consumes: Task 2 `EffectiveTheme::{Snow,Midnight}` and current semantic widget classes.
- Produces: curated single-purpose utilities, focused state classes, Snow defaults, Midnight overrides, and two GtkSourceView palettes.

- [ ] **Step 1: Write failing Atomic CSS and two-theme tests**

```rust
#[test]
fn curated_atomic_utilities_cover_the_approved_scale() {
    for utility in [
        ".nn-p-8", ".nn-p-12", ".nn-m-4", ".nn-h-32", ".nn-h-36",
        ".nn-radius-6", ".nn-radius-8", ".nn-text-body", ".nn-text-meta",
        ".nn-text-muted", ".nn-surface", ".nn-icon-button", ".nn-focus-ring",
    ] {
        assert!(CSS.contains(utility), "missing atomic utility: {utility}");
    }
}

#[test]
fn only_snow_and_midnight_theme_layers_remain() {
    for required in [".nn-theme-snow", ".nn-theme-midnight"] {
        assert!(CSS.contains(required));
    }
    for obsolete in [
        ".nn-theme-light", ".nn-theme-warm-paper", ".nn-theme-cool-mist",
        ".nn-theme-graphite", ".nn-theme-oled",
    ] {
        assert!(!CSS.contains(obsolete), "obsolete theme layer: {obsolete}");
    }
    assert!(CSS.lines().count() <= 500);
}
```

Keep GTK `CssProvider` parsing-error capture and selected-card contrast tests for both themes.

- [ ] **Step 2: Verify RED**

Run: `xvfb-run -a cargo test -p noor-notes --test design_system --test dark_palette_css --test source_palettes --test rich_colors --test compact_ui`

Expected: FAIL because utilities and Snow class do not exist, historical layers remain, and CSS exceeds the budget.

- [ ] **Step 3: Define compact Atomic CSS utilities**

```css
.nn-p-8 { padding: 8px; }
.nn-p-12 { padding: 12px; }
.nn-m-4 { margin: 4px; }
.nn-h-32 { min-height: 32px; }
.nn-h-36 { min-height: 36px; }
.nn-radius-6 { border-radius: 6px; }
.nn-radius-8 { border-radius: 8px; }
.nn-text-body { font-size: 16px; }
.nn-text-meta { font-size: 13px; color: @nn_text_secondary; }
.nn-text-muted { color: @nn_text_muted; }
.nn-surface { background: @nn_surface; color: @nn_text; }
.nn-icon-button { min-width: 32px; min-height: 32px; padding: 0; }
```

Use GTK-supported properties only. Keep selected, destructive, sticky-pin, editor-mode, and complex row behavior in focused semantic state classes.
Keep inter-widget spacing in GTK widget properties such as `Box::set_spacing(8)`; do not emulate the web `gap` property in CSS.

- [ ] **Step 4: Attach utilities to repeated widget structures**

In each listed Rust UI file, replace repeated geometry/typography-only component classes with explicit `add_css_class` calls, for example:

```rust
for class in ["nn-icon-button", "nn-h-32", "nn-radius-6", "nn-focus-ring"] {
    button.add_css_class(class);
}
```

Do not remove component classes that own behavior or state. Do not create runtime-generated class names.

- [ ] **Step 5: Consolidate Snow/Midnight tokens and palettes**

Snow defaults include `#F6F7F9`, `#F4F6F8`, `#F8F9FB`, `#FFFFFF`, `#1F2937`, `#475467`, `#667085`, `#E4E7EC`, `#EEF0F2`, `#4F6FE8`, and `#EEF2FF`. Midnight overrides use `#0F1724`, `#111A2A`, `#121C2D`, `#172235`, `#1D2A40`, `#F1F5F9`, `#CBD5E1`, `#94A3B8`, `#26364D`, `#6D8BFF`, and `#1D2A4A`.

```rust
pub const fn scheme_id(theme: EffectiveTheme) -> &'static str {
    match theme {
        EffectiveTheme::Snow => "noor-light",
        EffectiveTheme::Midnight => "noor-midnight",
    }
}
```

Keep stored rich-color preset/custom values unchanged; render through `theme.is_light()`.

- [ ] **Step 6: Verify GREEN and commit**

```bash
xvfb-run -a cargo test -p noor-notes --test design_system --test dark_palette_css --test source_palettes --test rich_colors --test compact_ui --test accessibility
cargo fmt --all -- --check
git diff --check
git add apps/noor-notes/resources/design-system.css apps/noor-notes/src/ui/app_header.rs apps/noor-notes/src/ui/library_sidebar.rs apps/noor-notes/src/ui/note_card.rs apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/src/sticky_note_window.rs apps/noor-notes/src/editor/source_palette.rs apps/noor-notes/src/rich_color.rs apps/noor-notes/tests/design_system.rs apps/noor-notes/tests/dark_palette_css.rs apps/noor-notes/tests/source_palettes.rs apps/noor-notes/tests/rich_colors.rs apps/noor-notes/tests/compact_ui.rs
git commit -m "ui: adopt atomic gtk styling for snow and midnight"
```

---

### Task 5: Refine Dynamic Pane Allocation and Document Hierarchy

**Files:**
- Modify: `apps/noor-notes/src/ui/adaptive_layout.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Modify: `apps/noor-notes/tests/adaptive_layout.rs`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/tests/preview_editor_surface.rs`
- Modify: `apps/noor-notes/tests/sticky_note_window.rs`

**Interfaces:**
- Consumes: `allocation_for_width`, `apply_library_layout`, current `NotePreview`, Atomic CSS utilities, and `StickyNoteWindow`.
- Produces: dynamic 10/18/remainder wide allocation, 16-pixel body, 20-pixel integrated title, compact controls, and body-only sticky presentation.

- [ ] **Step 1: Write failing adaptive and hierarchy tests**

```rust
#[test]
fn wide_allocation_targets_ten_eighteen_and_remaining_document_width() {
    let standard = allocation_for_width(LibraryLayoutMode::Wide, 1_180, false);
    assert_eq!(standard.sidebar, 160);
    assert_eq!(standard.collection, 280);
    let large = allocation_for_width(LibraryLayoutMode::Wide, 1_920, false);
    assert_eq!(large.sidebar, 192);
    assert_eq!(large.collection, 346);
    assert_eq!(large.navigation, 538);
}
```

Add CSS assertions for `.nn-preview-title { font-size: 20px;`, `.nn-text-body { font-size: 16px;`, `.nn-text-meta { font-size: 13px;`, and caption/status at 12 pixels. Retain real GTK medium/narrow and sticky one-title/body-only assertions.

- [ ] **Step 2: Verify RED**

Run: `xvfb-run -a cargo test -p noor-notes --test adaptive_layout --test design_system --test preview_editor_surface --test sticky_note_window`

Expected: FAIL on 18% large-window collection width and 20-pixel integrated title.

- [ ] **Step 3: Implement the minimum allocation and hierarchy changes**

```rust
let sidebar = ((width * 10 + 50) / 100).clamp(160, 220);
let collection = ((width * 18 + 50) / 100).clamp(280, 360);
```

Use 20 pixels for integrated title, 16 for reading/editor body, 13 for metadata, and 12 for captions/status. Preserve long-string wrapping, selected-card contrast, dynamic preview padding, source canvas width, Rich Text document serialization, and internal editor margins.

- [ ] **Step 4: Verify GREEN and commit**

```bash
xvfb-run -a cargo test -p noor-notes --test adaptive_layout --test design_system --test preview_editor_surface --test sticky_note_window --test editor_presentation
git diff --check
git add apps/noor-notes/src/ui/adaptive_layout.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/adaptive_layout.rs apps/noor-notes/tests/design_system.rs apps/noor-notes/tests/preview_editor_surface.rs apps/noor-notes/tests/sticky_note_window.rs
git commit -m "ui: refine adaptive workspace and document hierarchy"
```

---

### Task 6: Serialize Sticky Preference Persistence

**Files:**
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/tests/sticky_lifecycle.rs`

**Interfaces:**
- Consumes: `AutosaveQueue::flush`, `persist_sticky_preferences`, the integrated sticky callbacks, and a shared Tokio mutex.
- Produces: one ordered preference-write lane per `MainWindow`, so the latest confirmed Always-on-Top intent is always the last persisted value.

- [ ] **Step 1: Write the failing rapid-intent persistence test**

Create an actual repository and autosave queue, hold the shared preference mutex, enqueue `always_on_top = true`, then enqueue `always_on_top = false`, release the mutex, and assert the reopened note is `false`. The test calls the same serialized helper used by `MainWindow`; it must not inspect only UI state.

```rust
let lock = Arc::new(tokio::sync::Mutex::new(()));
let gate = lock.clone().lock_owned().await;
let first = tokio::spawn(persist_sticky_preferences_serialized(
    repository.clone(), autosave.clone(), lock.clone(), note.id,
    None, Some(true), now + Duration::seconds(1),
));
tokio::task::yield_now().await;
let latest = tokio::spawn(persist_sticky_preferences_serialized(
    repository.clone(), autosave, lock, note.id,
    None, Some(false), now + Duration::seconds(2),
));
tokio::task::yield_now().await;
drop(gate);
first.await.unwrap().unwrap();
latest.await.unwrap().unwrap();
assert!(!repository.get_note(note.id).await.unwrap().unwrap().always_on_top);
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p noor-notes --test sticky_lifecycle rapid_sticky_preferences_persist_latest_intent -- --exact`

Expected: compile failure because the serialized persistence helper and shared lane do not exist.

- [ ] **Step 3: Implement the minimum ordered persistence lane**

Add one `Arc<tokio::sync::Mutex<()>>` to `MainWindow`. The shared helper acquires it before `autosave.flush(id)` and holds it through `persist_sticky_preferences`. Both read-only close updates and Always-on-Top updates use this exact helper. Do not add a second repository, storage format, queue thread, or blocking GTK call.

- [ ] **Step 4: Verify GREEN and commit**

```bash
cargo test -p noor-notes --test sticky_lifecycle
xvfb-run -a cargo test -p noor-notes --test sticky_note_window
git diff --check
git add apps/noor-notes/src/ui/library_window.rs apps/noor-notes/tests/sticky_lifecycle.rs
git commit -m "fix: serialize sticky preference persistence"
```

---

### Task 7: Complete Functional, Performance, Release, Git, and Dev-Build Verification

**Files:**
- Modify only after a reproducible failing test identifies the smallest responsible production file.
- Do not modify note schema, encrypted payloads, user database, updated PDF/JSON references, or unrelated dirty files.

**Interfaces:**
- Consumes: Tasks 1-6 and existing repository verification/install scripts.
- Produces: a verified feature branch, synchronized GitHub `main`, and rebuilt Noor Notes Dev binary.

- [ ] **Step 1: Verify editor commands and persistence**

```bash
xvfb-run -a cargo test -p noor-notes --test editor_command_capabilities --test editor_history --test editor_menu_bar --test editor_presentation --test rich_editor --test rich_editor_ui --test rich_formatting_persistence --test emoji_insertion --test note_preview_edit --test library_preview_autosave --test view_only_mode
```

Expected: all Rich Text, Markdown, Plain Text, Code, read-only, history, formatting, emoji, and autosave tests pass. A real failure requires `superpowers:systematic-debugging` and a focused failing regression test before any correction.

- [ ] **Step 2: Verify library, sticky, settings, and adaptive behavior**

```bash
xvfb-run -a cargo test -p noor-notes --test adaptive_layout --test library_ui --test library_archive_action --test note_actions --test trash_actions --test search --test sticky_lifecycle --test sticky_note_window --test appearance_controls --test appearance_manager --test writing_assistance_settings_ui --test shortcuts --test accessibility
```

Expected: all workflows pass, including latest-intent Always-on-Top persistence.

- [ ] **Step 3: Audit dependency duplication and release size**

```bash
cargo tree --duplicates
bash tests/release_binary_size.sh
```

Record platform-driven duplicates as accepted unless `rg` proves a direct dependency unused. Remove no dependency speculatively.

- [ ] **Step 4: Run strict complete verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash scripts/security-check.sh
bash tests/release_workflow.sh
bash tests/snap_manifest.sh
git diff --check
git status --short
```

Expected: every command exits 0 and status contains only intentional branch files.

- [ ] **Step 5: Request code review and resolve valid findings**

Invoke `superpowers:requesting-code-review`. Every valid Critical/Important finding gets a failing regression test, minimum fix, and a repeated Step 4.

- [ ] **Step 6: Protect the original checkout's two pre-existing edits**

Verify the feature branch contains byte-identical final versions of `apps/noor-notes/src/editor/source_adapter.rs` and `apps/noor-notes/tests/rich_editor.rs`. From the original checkout:

```bash
git diff -- apps/noor-notes/src/editor/source_adapter.rs apps/noor-notes/tests/rich_editor.rs > /tmp/noor-preexisting-source-undo.patch
git stash push -m noor-preexisting-source-undo -- apps/noor-notes/src/editor/source_adapter.rs apps/noor-notes/tests/rich_editor.rs
```

Do not stash or alter screenshot deletions, generated references, or snap artifacts.

- [ ] **Step 7: Fast-forward and push GitHub main**

```bash
git checkout main
git merge --ff-only feature/two-theme-product-redesign
git push origin main
git rev-parse HEAD
git rev-parse origin/main
```

Expected: local and remote IDs match. Keep the recovery patch until committed files prove the prior behavior is preserved.

- [ ] **Step 8: Rebuild and verify Noor Notes Dev**

```bash
PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh
test -x /home/mamun/.local/bin/noor-notes
/home/mamun/.local/bin/noor-notes --version
cargo test -p noor-notes --test local_dev_installer
/home/mamun/.local/bin/noor-notes --help
```

Use the existing development identity. Automated startup must not open or mutate the user's personal database. Report final commit, remote match, binary/version, verification results, and any environment-only GTK limitation.
