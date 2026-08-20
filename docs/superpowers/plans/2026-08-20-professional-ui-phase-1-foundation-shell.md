# Professional UI Phase 1: Foundation and Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the visible library shell, adaptive pane sizing, navigation, note cards, empty states, and reading workspace with the approved professional component system while preserving repository, autosave, search, sorting, and note lifecycle behavior.

**Architecture:** Keep `MainWindow` as the state/repository coordinator. Add focused header and pane-allocation components, retain the existing virtualized `NoteCollection`, and keep `NotePreview` as the compatibility-facing document workspace while replacing its presentation structure. All visual values come from the existing centralized GTK stylesheet.

**Tech Stack:** Rust 1.85, GTK4, Libadwaita, existing `noor-domain`, `noor-storage`, `AutosaveQueue`, and GTK integration tests under Xvfb.

**Spec:** `docs/superpowers/specs/2026-08-20-professional-product-ui-redesign-design.md`

## Global Constraints

- Do not change note IDs, encrypted payloads, the database schema, keyring behavior, or storage paths.
- Do not add dependencies or a second design system.
- Preserve Snow, Warm Paper, Cool Mist, Graphite, Midnight, and OLED themes.
- Target wide pane ratios of about 10% navigation, 20% notes, and the remainder document, with explicit readability guards.
- Preserve the existing search debounce, sort values, virtualized note list, autosave path, and lifecycle services.
- Preserve Rich Text internal margins of 8 pixels horizontally and 5 pixels vertically.
- Do not modify or stage unrelated worktree changes.

---

### Task 1: Complete the Semantic Foundation Contracts

**Files:**
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/resources/design-system.css:1-483`

**Interfaces:**
- Consumes: the existing GTK named-color and `.nn-*` class system.
- Produces: semantic surface, text, interaction, elevation, geometry, typography, and reduced-motion contracts used by every later phase.

- [ ] **Step 1: Write the failing semantic-token and primitive tests**

Add tests that require the missing roles and prohibit component-local Snow literals:

```rust
#[test]
fn professional_system_defines_complete_semantic_roles() {
    for token in [
        "@define-color nn_popover_bg",
        "@define-color nn_modal_bg",
        "@define-color nn_input_bg",
        "@define-color nn_text_disabled",
        "@define-color nn_text_inverse",
        "@define-color nn_border_strong",
        "@define-color nn_accent_strong",
        "@define-color nn_danger_soft",
        "@define-color nn_info",
        ".nn-control-compact",
        ".nn-control-primary",
        ".nn-surface-elevated",
        ".nn-menu-surface",
        ".nn-document-title",
    ] {
        assert!(CSS.contains(token), "missing professional role: {token}");
    }
}

#[test]
fn transient_surfaces_share_one_component_language() {
    let rule = CSS
        .split(".nn-menu-surface {")
        .nth(1)
        .and_then(|css| css.split('}').next())
        .expect("shared menu surface rule");
    assert!(rule.contains("background: @nn_popover_bg"));
    assert!(rule.contains("border: 1px solid @nn_border"));
    assert!(rule.contains("border-radius: 10px"));
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test design_system professional_system -- --nocapture
```

Expected: FAIL because the new semantic roles/classes do not exist.

- [ ] **Step 3: Add centralized roles and shared primitives**

Add the roles at the stylesheet root and implement shared primitives without changing theme-specific component structure:

```css
@define-color nn_popover_bg @nn_surface_raised;
@define-color nn_modal_bg @nn_surface_raised;
@define-color nn_input_bg @nn_surface;
@define-color nn_text_disabled alpha(@nn_text_muted, .72);
@define-color nn_text_inverse #ffffff;
@define-color nn_border_strong #d0d5dd;
@define-color nn_accent_strong #344fc4;
@define-color nn_danger_soft #fef2f2;
@define-color nn_info #2563eb;

.nn-control-compact { min-width: 32px; min-height: 32px; padding: 0 8px; border-radius: 6px; }
.nn-control-primary { min-height: 36px; padding: 0 12px; border-radius: 8px; }
.nn-surface-elevated { background: @nn_surface_raised; border: 1px solid @nn_border; border-radius: 12px; }
.nn-menu-surface { background: @nn_popover_bg; border: 1px solid @nn_border; border-radius: 10px; padding: 6px; }
.nn-document-title { font-size: 30px; font-weight: 700; }
```

Add explicit semantic overrides for non-Snow themes where a role cannot safely inherit its Snow value. Keep all existing source-canvas palette isolation rules.

- [ ] **Step 4: Run the complete design-system test and verify GREEN**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test design_system
```

Expected: PASS with no GTK CSS parsing errors.

- [ ] **Step 5: Commit the semantic foundation**

```bash
git add apps/noor-notes/tests/design_system.rs apps/noor-notes/resources/design-system.css
git commit -m "ui: consolidate professional semantic design roles"
```

### Task 2: Replace Fixed Pane Positions with Ratio Allocation

**Files:**
- Modify: `apps/noor-notes/src/ui/adaptive_layout.rs:1-89`
- Modify: `apps/noor-notes/tests/adaptive_layout.rs`

**Interfaces:**
- Consumes: `LibraryLayoutMode`, actual allocated window width, and narrow content/list state.
- Produces: `LibraryPaneAllocation` and `allocation_for_width(mode, width, showing_content)` for `MainWindow::apply_layout`.

- [ ] **Step 1: Write failing allocation-policy tests**

```rust
use noor_notes::ui::adaptive_layout::{allocation_for_width, LibraryLayoutMode};

#[test]
fn wide_allocation_targets_ten_twenty_seventy_with_readability_guards() {
    let standard = allocation_for_width(LibraryLayoutMode::Wide, 1_180, false);
    assert_eq!(standard.sidebar, 160);
    assert_eq!(standard.collection, 280);
    assert_eq!(standard.navigation, 440);

    let large = allocation_for_width(LibraryLayoutMode::Wide, 1_920, false);
    assert_eq!(large.sidebar, 192);
    assert_eq!(large.collection, 360);
    assert_eq!(large.navigation, 552);
}

#[test]
fn narrow_allocation_never_leaves_a_preview_sliver() {
    assert_eq!(
        allocation_for_width(LibraryLayoutMode::Narrow, 620, true).navigation,
        0
    );
    assert_eq!(
        allocation_for_width(LibraryLayoutMode::Narrow, 620, false).navigation,
        620
    );
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test adaptive_layout wide_allocation -- --nocapture
```

Expected: FAIL because `LibraryPaneAllocation` and `allocation_for_width` do not exist.

- [ ] **Step 3: Implement the pure allocation policy**

Replace fixed `481`/`300` positions with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryPaneAllocation {
    pub sidebar: i32,
    pub collection: i32,
    pub navigation: i32,
}

pub fn allocation_for_width(
    mode: LibraryLayoutMode,
    width: i32,
    showing_content: bool,
) -> LibraryPaneAllocation {
    match mode {
        LibraryLayoutMode::Wide => {
            let sidebar = ((width as f32 * 0.10).round() as i32).clamp(160, 220);
            let collection = ((width as f32 * 0.20).round() as i32).clamp(280, 360);
            LibraryPaneAllocation { sidebar, collection, navigation: sidebar + collection }
        }
        LibraryLayoutMode::Medium => {
            let collection = ((width as f32 * 0.34).round() as i32).clamp(280, 360);
            LibraryPaneAllocation { sidebar: 0, collection, navigation: collection }
        }
        LibraryLayoutMode::Narrow if showing_content =>
            LibraryPaneAllocation { sidebar: 0, collection: 0, navigation: 0 },
        LibraryLayoutMode::Narrow =>
            LibraryPaneAllocation { sidebar: 0, collection: width, navigation: width },
    }
}
```

Make `pane_position` delegate to the new policy or remove it after updating all callers.

- [ ] **Step 4: Run adaptive tests and verify GREEN**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test adaptive_layout
```

Expected: PASS, including the real allocation test that gives the visible narrow preview more than 500 pixels.

- [ ] **Step 5: Commit ratio allocation**

```bash
git add apps/noor-notes/src/ui/adaptive_layout.rs apps/noor-notes/tests/adaptive_layout.rs
git commit -m "ui: allocate library panes from window ratios"
```

### Task 3: Extract the Professional Application Header

**Files:**
- Create: `apps/noor-notes/src/ui/app_header.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs:85-205`
- Modify: `apps/noor-notes/tests/library_ui.rs`

**Interfaces:**
- Consumes: `AppearanceManager`, initial `NoteSort`, and existing application actions.
- Produces: `AppHeader { widget, back, search_toggle, sort }` plus `set_compact(bool)`.

- [ ] **Step 1: Write the failing header component test**

```rust
#[test]
fn professional_header_exposes_only_real_primary_controls() {
    gtk::init().unwrap();
    let header = AppHeader::new(global(), NoteSort::UpdatedDesc);
    assert!(header.widget.has_css_class("nn-app-header"));
    assert_eq!(header.sort.selected(), 0);
    assert_eq!(header.back.tooltip_text().as_deref(), Some("Back to notes"));
    assert_eq!(header.search_toggle.tooltip_text().as_deref(), Some("Search notes (Ctrl+F)"));
    assert!(header.new_note.has_css_class("suggested-action"));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test library_ui professional_header -- --nocapture
```

Expected: FAIL because `AppHeader` does not exist.

- [ ] **Step 3: Implement `AppHeader` using existing actions**

Move the current header construction into a component while preserving `app.new-note`, `app.import-xpad`, `app.shortcuts`, `app.appearance::*`, `app.appearance-settings`, `app.writing-assistance-settings`, and `app.quit`. Use public fields required by `MainWindow`:

```rust
#[derive(Clone)]
pub struct AppHeader {
    pub widget: adw::HeaderBar,
    pub new_note: gtk::Button,
    pub back: gtk::Button,
    pub search_toggle: gtk::ToggleButton,
    pub sort: gtk::DropDown,
}
```

Apply `nn-control-primary` to New Note and `nn-header-control nn-control-compact` to secondary icon controls. Keep native window controls and the existing centered product identity.

- [ ] **Step 4: Replace inline header construction and verify GREEN**

Update `MainWindow::new` to own the extracted fields, preserve all bindings/signals, then run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test library_ui --test accessibility
```

Expected: PASS; action names, tooltips, and accessible labels remain real.

- [ ] **Step 5: Commit the header component**

```bash
git add apps/noor-notes/src/ui/app_header.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/tests/library_ui.rs
git commit -m "ui: replace inline library header with app header"
```

### Task 4: Apply Ratio Allocation in the Real Shell

**Files:**
- Modify: `apps/noor-notes/src/ui/library_window.rs:50-455`
- Modify: `apps/noor-notes/src/ui/library_sidebar.rs:18-96`
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/tests/compact_ui.rs`

**Interfaces:**
- Consumes: `allocation_for_width` from Task 2 and `AppHeader` from Task 3.
- Produces: real sidebar/collection requests and outer paned position that track actual allocation.

- [ ] **Step 1: Write a failing allocated-widget ratio test**

Create a window at 1,180 pixels, apply layout, and assert the actual navigation and preview allocations:

```rust
assert!((150..=170).contains(&sidebar.width()));
assert!((270..=290).contains(&collection_stack.width()));
assert!(preview.widget.width() >= 700);
```

Also assert a 1,920-pixel allocation respects the 220/360 upper guards and leaves the document dominant.

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test compact_ui dynamic_shell -- --nocapture
```

Expected: FAIL because current requests remain 180/300 and the outer position remains fixed.

- [ ] **Step 3: Wire the ratio policy into `MainWindow::apply_layout`**

Use actual allocation and configure the two navigation children before applying outer visibility:

```rust
let allocation = allocation_for_width(
    mode,
    self.window.width().max(self.window.default_width()),
    self.showing_content.get(),
);
self.sidebar.widget.set_width_request(allocation.sidebar);
self.collection_stack.set_width_request(allocation.collection);
self.panes.set_position(allocation.navigation);
```

Remove constructor-time fixed requests/positions. Keep narrow start/end visibility explicit so the preview cannot become a sliver.

- [ ] **Step 4: Run layout suites and verify GREEN**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test adaptive_layout --test compact_ui --test library_ui
```

Expected: PASS at wide, medium, and narrow allocations.

- [ ] **Step 5: Commit real shell allocation**

```bash
git add apps/noor-notes/src/ui/library_window.rs apps/noor-notes/src/ui/library_sidebar.rs apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/compact_ui.rs
git commit -m "ui: make the three-pane shell allocation adaptive"
```

### Task 5: Replace Sidebar and Note Card Presentation

**Files:**
- Modify: `apps/noor-notes/src/ui/library_sidebar.rs`
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/src/ui/note_collection.rs`
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: existing `LibrarySection`, `CardAction`, and `gio::ListStore`/`gtk::ListView` selection model.
- Produces: compact accessible navigation rows and lifecycle-aware document cards without changing callbacks.

- [ ] **Step 1: Write failing structure/state tests**

Require a 40-pixel sidebar row, four-pixel rail, two-line preview, status descriptions, and separated destructive action:

```rust
assert_eq!(row.height_request(), 40);
assert_eq!(color_rail.width_request(), 4);
assert_eq!(preview.lines(), 2);
assert_eq!(menu.tooltip_text().as_deref(), Some("Note actions"));
assert!(trash_action.has_css_class("destructive-action"));
assert!(trash_action.has_css_class("nn-menu-danger"));
```

Add an assertion that selected cards expose an additional focus/selection cue and retain explicit readable foregrounds in Snow.

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test library_ui redesigned_library -- --nocapture
```

Expected: FAIL on the new row/card/menu presentation contracts.

- [ ] **Step 3: Implement the compact structures**

Set row/card widget properties in Rust and move visual styling to semantic classes. Keep card contents stable while making preview two lines and ensuring status icons do not shift layout. Build action popovers in lifecycle order and apply `nn-menu-surface`, `nn-menu-row`, `nn-menu-separator`, and `nn-menu-danger` classes.

- [ ] **Step 4: Run library/action tests and verify GREEN**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test library_ui --test note_card_archive --test library_archive_action --test note_actions
```

Expected: PASS; Archive/Trash/Restore behavior remains unchanged.

- [ ] **Step 5: Commit navigation and card replacement**

```bash
git add apps/noor-notes/src/ui/library_sidebar.rs apps/noor-notes/src/ui/note_card.rs apps/noor-notes/src/ui/note_collection.rs apps/noor-notes/tests/library_ui.rs apps/noor-notes/resources/design-system.css
git commit -m "ui: replace sidebar and note card presentation"
```

### Task 6: Professionalize Empty and Search States

**Files:**
- Modify: `apps/noor-notes/src/ui/empty_state.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs:438-585`
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/tests/search.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: `LibrarySection`, existing search entry text, projected notes, and result count.
- Produces: section-aware title/description/icon and quiet result/status feedback.

- [ ] **Step 1: Write failing empty-state and search-result tests**

```rust
empty.update(LibrarySection::Pinned, false);
assert_eq!(empty.title_text(), "No pinned notes");
assert!(empty.description_text().contains("Pin important notes"));
empty.update(LibrarySection::AllNotes, true);
assert_eq!(empty.title_text(), "No notes found");
assert!(empty.description_text().contains("clear the search"));
```

Expose read-only test getters rather than traversing labels by position.

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test library_ui empty_state -- --nocapture
```

Expected: FAIL because the getters and final copy/icon contract do not exist.

- [ ] **Step 3: Implement explicit semantic empty states**

Store the title/description labels, expose `title_text()`/`description_text()`, assign section-appropriate symbolic icons, and style the empty state with semantic title, metadata, and subdued action guidance. Keep search fields and debounce logic unchanged.

- [ ] **Step 4: Run search/library tests and verify GREEN**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test search --test library_ui
```

Expected: PASS for searchable fields, result projection, and all empty states.

- [ ] **Step 5: Commit empty/search polish**

```bash
git add apps/noor-notes/src/ui/empty_state.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/search.rs apps/noor-notes/resources/design-system.css
git commit -m "ui: refine library empty and search states"
```

### Task 7: Replace the Reading Workspace Presentation

**Files:**
- Modify: `apps/noor-notes/src/ui/note_preview.rs:42-511`
- Modify: `apps/noor-notes/src/ui/editor_canvas.rs`
- Modify: `apps/noor-notes/tests/preview_editor_surface.rs`
- Modify: `apps/noor-notes/tests/note_preview_edit.rs`
- Modify: `apps/noor-notes/tests/library_preview_autosave.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: existing `BodyEditHandler`, `EditFinishedHandler`, `ReadOnlyHandler`, `RichBuffer`, and `AutosaveQueue` path.
- Produces: coherent label-based reading mode, entry-based edit mode, shared alignment grid, and unchanged autosave callbacks.

- [ ] **Step 1: Write failing reading/edit transition tests**

Require reading mode to show the title label and hide the title entry, then invert in edit mode:

```rust
preview.show_note(&note);
assert_eq!(preview.title_stack_child_name(), "label");
assert!(!preview.toolbar_visible());
preview.begin_editing();
assert_eq!(preview.title_stack_child_name(), "entry");
assert!(preview.toolbar_visible());
assert_eq!(preview.editor().left_margin(), 8);
assert_eq!(preview.editor().top_margin(), 5);
```

Add a long unbroken content allocation assertion that the document never grows beyond its viewport.

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test preview_editor_surface reading_mode -- --nocapture
```

Expected: FAIL because the current preview keeps the entry child visible in reading mode and lacks test-facing transition methods.

- [ ] **Step 3: Implement the document grid and real read/edit states**

Use the title label in reading mode, title entry in editing mode, one shared vertical document box, and a dynamic `adw::Clamp` whose content remains readable. Keep the existing callbacks and snapshot logic. Add small test-facing methods that delegate to the same private transition path used by the Edit/Done button; do not create a test-only alternate behavior.

Keep:

```rust
configure_editor_canvas(&editor, true); // retains 8 horizontal / 5 vertical margins
```

Move Edit and Open Read-only into a compact action group. Make the body selectable and long-token safe through GTK wrapping and allocation, not by changing stored content.

- [ ] **Step 4: Run preview/autosave tests and verify GREEN**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test preview_editor_surface --test note_preview_edit --test library_preview_autosave
```

Expected: PASS; title/body edits still schedule and flush through the existing autosave queue.

- [ ] **Step 5: Commit the document workspace presentation**

```bash
git add apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/src/ui/editor_canvas.rs apps/noor-notes/tests/preview_editor_surface.rs apps/noor-notes/tests/note_preview_edit.rs apps/noor-notes/tests/library_preview_autosave.rs apps/noor-notes/resources/design-system.css
git commit -m "ui: replace preview with a coherent document workspace"
```

### Task 8: Phase 1 Integration Gate

**Files:**
- Modify only if a failing test identifies a Phase 1 regression.

**Interfaces:**
- Consumes: Tasks 1–7.
- Produces: a stable professional shell ready for editor component replacement.

- [ ] **Step 1: Run formatting and focused Phase 1 suites**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test design_system --test adaptive_layout --test compact_ui --test library_ui --test accessibility --test preview_editor_surface --test note_preview_edit --test library_preview_autosave --test search --test note_actions
```

Expected: PASS without GTK CSS parsing errors or allocation failures.

- [ ] **Step 2: Run non-UI compatibility suites**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test autosave --test library_state --test library_preferences --test note_commands
```

Expected: PASS; no storage or lifecycle semantics changed.

- [ ] **Step 3: Review the diff for scope and whitespace**

```bash
git diff --check
git status --short
```

Expected: no whitespace errors; unrelated pre-existing screenshot/snap changes remain unstaged and untouched.

- [ ] **Step 4: Commit any test-proven integration correction**

Stage only files changed to fix a demonstrated Phase 1 regression and commit with:

```bash
git commit -m "test: close professional shell integration gaps"
```
