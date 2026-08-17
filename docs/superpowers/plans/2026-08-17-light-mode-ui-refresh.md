# Light Mode UI Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a calm, professional Noor Notes Light Mode with clearer pane hierarchy, restrained note colors, compact actions, readable preview typography, consistent controls, accessible states, and unchanged application behavior.

**Architecture:** Keep `design-system.css` as the only visual token layer and refine existing GTK widgets rather than replacing them. Make small component changes only where CSS cannot express the required semantics: explicit sidebar/icon classes, compact card actions, safe Pango wrapping, narrow-window state, and header control classes.

**Tech Stack:** Rust 1.85, GTK4 0.10, Libadwaita 0.8, GtkSourceView 5, Pango, Cargo integration tests, Xvfb

## Global Constraints

- Preserve notes, storage, encryption, autosave, sync, imports, search, sorting, note actions, editor modes, shortcuts, sticky behavior, and window controls.
- Preserve Rich Text canvas margins at exactly 5 pixels top/bottom and 8 pixels left/right.
- Keep Graphite, Midnight, and OLED functional; do not leak Light Mode literals into shared dark components.
- Use the existing symbolic icon system and `design-system.css`; add no dependency and no competing design system.
- Use semantic GTK color tokens and restrained 120–180 millisecond transitions.
- Keep practical icon targets at least 32 pixels and preserve accessible labels, tooltips, keyboard focus, and tab order.
- Do not commit temporary screenshots, databases, capture harnesses, logs, build products, or the two pre-existing `.snap` files.
- Do not push unless the user explicitly requests it.

---

### Task 1: Establish the semantic Light Mode palette and interaction states

**Files:**
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/resources/design-system.css:1-33`
- Modify: `apps/noor-notes/resources/design-system.css:173-304`

**Interfaces:**
- Consumes: existing GTK `@define-color` tokens and root theme classes.
- Produces: semantic Light Mode tokens consumed by every later CSS task; unchanged `nn-theme-graphite`, `nn-theme-midnight`, and `nn-theme-oled` ownership of dark surfaces.

- [ ] **Step 1: Write failing token and contrast-state contracts**

Add this test to `apps/noor-notes/tests/design_system.rs`:

```rust
#[test]
fn light_mode_uses_professional_semantic_tokens_and_neutral_interactions() {
    for declaration in [
        "@define-color nn_app_bg #f7f8fa;",
        "@define-color nn_sidebar_bg #f6f7f9;",
        "@define-color nn_note_list_bg #fafafb;",
        "@define-color nn_surface #ffffff;",
        "@define-color nn_hover #f1f3f6;",
        "@define-color nn_text #1f2937;",
        "@define-color nn_text_secondary #667085;",
        "@define-color nn_text_muted #6b7280;",
        "@define-color nn_border #e5e7eb;",
        "@define-color nn_border_subtle #eef0f2;",
        "@define-color nn_accent #4f6fe8;",
        "@define-color nn_accent_hover #425fcc;",
        "@define-color nn_accent_soft #eef2ff;",
        "@define-color nn_danger #dc2626;",
        "@define-color nn_success #16a34a;",
        "@define-color nn_scrollbar #c7cdd6;",
        "@define-color nn_scrollbar_hover #aeb7c4;",
    ] {
        assert!(CSS.contains(declaration), "missing Light token: {declaration}");
    }
    let button_hover = CSS
        .split("button:hover")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("button hover rules");
    assert!(button_hover.contains("@nn_hover"));
    assert!(!button_hover.contains("@nn_accent"));
    for theme in ["graphite", "midnight", "oled"] {
        assert!(
            CSS.contains(&format!(".nn-theme-{theme} .nn-sidebar-row:selected")),
            "dark sidebar selection must be explicit for {theme}"
        );
    }
}
```

Extend `replacement_design_system_defines_semantic_light_dark_and_accessible_states` to require `@define-color nn_note_list_bg` and `@define-color nn_focus_ring`. Header and card action classes are introduced and tested in their own later tasks.

- [ ] **Step 2: Run the design-system test and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test design_system --locked
```

Expected: FAIL because `nn_note_list_bg` and the new exact Light Mode declarations do not exist.

- [ ] **Step 3: Replace only the shared Light token block and global states**

Use this semantic block at the top of `design-system.css`:

```css
@define-color nn_bg #f7f8fa;
@define-color nn_app_bg #f7f8fa;
@define-color nn_sidebar_bg #f6f7f9;
@define-color nn_note_list_bg #fafafb;
@define-color nn_surface #ffffff;
@define-color nn_editor_bg #ffffff;
@define-color nn_surface_raised #ffffff;
@define-color nn_hover #f1f3f6;
@define-color nn_text #1f2937;
@define-color nn_text_secondary #667085;
@define-color nn_text_muted #6b7280;
@define-color nn_border #e5e7eb;
@define-color nn_border_subtle #eef0f2;
@define-color nn_accent #4f6fe8;
@define-color nn_accent_hover #425fcc;
@define-color nn_accent_soft #eef2ff;
@define-color nn_danger #dc2626;
@define-color nn_success #16a34a;
@define-color nn_scrollbar #c7cdd6;
@define-color nn_scrollbar_hover #aeb7c4;
@define-color nn_warning #9a6400;
@define-color nn_error #dc2626;
@define-color nn_focus #4f6fe8;
@define-color nn_focus_ring alpha(@nn_accent, .25);
@define-color nn_selected #eef2ff;
```

Change global hover/focus behavior to:

```css
button:hover { background: @nn_hover; }
button:checked, button:selected { background: @nn_accent_soft; color: @nn_accent; }
button:focus-visible, entry:focus-visible, row:focus-visible, textview:focus-visible {
  outline: 1px solid @nn_focus;
  outline-offset: 1px;
  box-shadow: 0 0 0 3px @nn_focus_ring;
}
```

Add explicit dark sidebar selection rules beside the existing dark card-selection rules, using each palette's existing selected surface and accent:

```css
.nn-theme-graphite .nn-sidebar-row:selected { background: #30364a; color: #ffffff; box-shadow: inset 3px 0 #9aafff; }
.nn-theme-midnight .nn-sidebar-row:selected { background: #203858; color: #ffffff; box-shadow: inset 3px 0 #73b7ff; }
.nn-theme-oled .nn-sidebar-row:selected { background: #211d38; color: #ffffff; box-shadow: inset 3px 0 #b1a0ff; }
```

- [ ] **Step 4: Verify GTK CSS parsing and semantic tests GREEN**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test design_system --locked
```

Expected: all design-system tests pass, including GTK CSS parsing.

- [ ] **Step 5: Commit the semantic foundation**

```bash
git add apps/noor-notes/resources/design-system.css apps/noor-notes/tests/design_system.rs
git commit -m "style: refine light mode semantic foundation"
```

---

### Task 2: Clarify sidebar, note-list, and pane hierarchy

**Files:**
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/src/ui/library_sidebar.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs:147-218`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: Task 1 tokens `nn_sidebar_bg`, `nn_note_list_bg`, `nn_hover`, `nn_selected`, and `nn_text_muted`.
- Produces: 232-pixel expanded sidebar, 336-pixel note-list request, and semantic classes `nn-sidebar-icon`, `nn-sidebar-label`, `nn-sidebar-count`, and `nn-pane-separator`.

- [ ] **Step 1: Write failing sidebar structure assertions**

In `redesigned_library_uses_sidebar_virtualized_list_and_cards`, change the expanded-width assertion to `232` and add:

```rust
for index in 0..7 {
    let row = list.row_at_index(index).unwrap();
    let content = row.child().and_downcast::<gtk::Box>().unwrap();
    let icon = content.first_child().unwrap();
    let label = icon.next_sibling().unwrap();
    let count = label.next_sibling().unwrap();
    assert!(icon.has_css_class("nn-sidebar-icon"));
    assert!(label.has_css_class("nn-sidebar-label"));
    assert!(count.has_css_class("nn-sidebar-count"));
}
```

Add a CSS contract to `design_system.rs`:

```rust
#[test]
fn light_library_layers_sidebar_and_note_list_without_heavy_borders() {
    assert!(CSS.contains(".nn-sidebar { background: @nn_sidebar_bg;"));
    assert!(CSS.contains(".nn-note-list { background: @nn_note_list_bg;"));
    assert!(CSS.contains(".nn-sidebar-row { min-height: 42px;"));
    assert!(CSS.contains(".nn-pane-separator { background: @nn_border;"));
}
```

- [ ] **Step 2: Run focused UI tests and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test library_ui --test design_system --locked
```

Expected: FAIL at the 232-pixel width and missing semantic classes.

- [ ] **Step 3: Add sidebar semantics and pane sizing**

In `LibrarySidebar::new`, set expanded width to `232`. Add `nn-sidebar-icon` to each image, `nn-sidebar-label` to each label, and replace the count's generic caption-only styling with both `nn-caption` and `nn-sidebar-count`. In `set_collapsed`, keep `64` for collapsed and use `232` for expanded.

In `library_window.rs`, set `collection_stack.set_width_request(336)`, add `nn-pane-separator` to `sidebar_separator`, and set the wide pane position to `569` (`232 + 1 + 336`). Keep medium position at `336`.

Use these CSS rules:

```css
.nn-sidebar { background: @nn_sidebar_bg; border-right: 0; padding: 12px 8px; }
.nn-pane-separator { min-width: 1px; background: @nn_border; }
.nn-sidebar-row { min-height: 42px; margin: 2px 0; padding: 0 12px; border-radius: 8px; transition: 140ms ease; }
.nn-sidebar-row:hover { background: @nn_hover; }
.nn-sidebar-row:selected, .nn-sidebar-row:checked {
  background: @nn_selected;
  color: @nn_accent;
  font-weight: 600;
  box-shadow: inset 3px 0 @nn_accent;
}
.nn-sidebar-icon { color: @nn_text_secondary; -gtk-icon-size: 18px; }
.nn-sidebar-row:selected .nn-sidebar-icon { color: @nn_accent; }
.nn-sidebar-count { color: @nn_text_muted; font-weight: 400; }
.nn-note-list { background: @nn_note_list_bg; padding: 12px 10px; }
```

- [ ] **Step 4: Run focused tests GREEN**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test library_ui --test design_system --locked
```

Expected: all sidebar, component, and CSS tests pass.

- [ ] **Step 5: Commit pane hierarchy**

```bash
git add apps/noor-notes/src/ui/library_sidebar.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/design_system.rs
git commit -m "style: clarify light library pane hierarchy"
```

---

### Task 3: Redesign note cards and compact their actions

**Files:**
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: existing `NoteColor::css_class`, `CardAction`, and Task 1 tokens.
- Produces: `nn-note-card-preview`, `nn-note-status-icon`, and `nn-card-action` classes; centered 32-pixel actions; unchanged archive/menu callbacks.

- [ ] **Step 1: Write failing card hierarchy and action assertions**

After building the test card in `library_ui.rs`, add:

```rust
let color_rail = card.widget.first_child().expect("note color rail");
assert!(color_rail.has_css_class("nn-color-strip"));
assert_eq!(color_rail.width_request(), 4);
assert!(card.menu.has_css_class("nn-card-action"));
assert_eq!(card.menu.valign(), gtk::Align::Center);
let archive = card.archive.as_ref().expect("active card archive action");
assert!(archive.has_css_class("nn-card-action"));
assert_eq!(archive.valign(), gtk::Align::Center);
assert!(
    descendants(card.widget.clone().upcast())
        .iter()
        .any(|widget| widget.has_css_class("nn-note-card-preview"))
);
```

Add this design-system test:

```rust
#[test]
fn note_colors_are_identity_rails_and_selection_remains_calm() {
    for color in ["yellow", "cream", "blue", "green", "rose", "lavender"] {
        assert!(CSS.contains(&format!(".note-{color} .nn-color-strip")));
    }
    assert!(CSS.contains(".nn-card-action { min-width: 32px; min-height: 32px;"));
    let selected = CSS
        .split(".nn-note-list row:selected .nn-note-card")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("selected card rules");
    assert!(selected.contains("@nn_selected"));
    assert!(!selected.contains("color: white"));
}
```

- [ ] **Step 2: Run card tests and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test library_ui --test design_system --locked
```

Expected: FAIL because card action and preview classes are absent.

- [ ] **Step 3: Add compact semantic card widgets**

In `note_card.rs`:

```rust
preview.add_css_class("nn-note-card-preview");
preview.set_wrap_mode(gtk::pango::WrapMode::WordChar);
```

Add `nn-note-status-icon` to pin/favorite images before appending them. For both archive and menu controls:

```rust
button.add_css_class("flat");
button.add_css_class("nn-card-action");
button.set_valign(gtk::Align::Center);
```

Apply the equivalent three lines to `menu` after construction. Do not change action callbacks, visibility binding, right-click popover, or destructive confirmation.

- [ ] **Step 4: Replace card presentation rules without touching dark overrides**

Use these shared/light rules:

```css
.nn-note-list row { background: transparent; padding: 0; }
.nn-note-list row:selected { background: transparent; }
.nn-note-list row:selected .nn-note-card {
  background: @nn_selected;
  border-color: alpha(@nn_accent, .35);
  box-shadow: 0 0 0 1px alpha(@nn_accent, .14);
}
.nn-note-card {
  min-height: 92px;
  margin: 5px 4px;
  padding: 14px;
  background: @nn_surface;
  border: 1px solid @nn_border;
  border-radius: 10px;
  box-shadow: 0 1px 2px rgba(16,24,40,.03);
  transition: 150ms ease;
}
.nn-note-card:hover { border-color: #d6dae1; box-shadow: 0 2px 5px rgba(16,24,40,.05); }
.nn-note-card-content { padding: 0 2px; }
.nn-note-title { font-size: 16px; font-weight: 600; }
.nn-note-card-preview { font-size: 13px; color: @nn_text_secondary; }
.nn-note-card-tags { font-size: 12px; color: @nn_text_secondary; }
.nn-note-card-meta { font-size: 12px; color: @nn_text_muted; }
.nn-note-status-icon { color: @nn_text_secondary; }
.nn-card-action { min-width: 32px; min-height: 32px; padding: 0; border-radius: 8px; background: transparent; }
.nn-card-action:hover { background: @nn_hover; }
```

Keep the six existing rail colors and add only very faint Light Mode card tints:

```css
.nn-theme-light .note-yellow { background: #fffef5; }
.nn-theme-light .note-cream { background: #fffaf4; }
.nn-theme-light .note-blue { background: #f7f9ff; }
.nn-theme-light .note-green { background: #f7fcf8; }
.nn-theme-light .note-rose { background: #fff8fa; }
.nn-theme-light .note-lavender { background: #faf8ff; }
.nn-theme-light .nn-note-list row:selected .nn-note-card { background: @nn_selected; }
```

- [ ] **Step 5: Verify cards GREEN and lifecycle action tests unchanged**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test library_ui --test note_card_archive --test trash_actions --test design_system --locked
```

Expected: all card structure, archive, Trash, and CSS tests pass.

- [ ] **Step 6: Commit note-card redesign**

```bash
git add apps/noor-notes/src/ui/note_card.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/design_system.rs
git commit -m "style: refine light note cards and actions"
```

---

### Task 4: Improve preview typography and long-content safety

**Files:**
- Modify: `apps/noor-notes/tests/library_ui.rs`
- Modify: `apps/noor-notes/tests/editor_canvas.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/src/ui/editor_canvas.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: `NotePreview::show_note(&Note)` and existing 860-pixel `adw::Clamp`.
- Produces: semantic preview title/meta/body classes, `WordChar` wrapping, readable Pango and Rich Text line height, and unchanged Rich Text margins.

- [ ] **Step 1: Write failing long-content and Rich Text line-spacing tests**

Extend `library_ui.rs` after `preview.show_note(&note)`:

```rust
let preview_labels: Vec<gtk::Label> = descendants(preview.widget.clone().upcast())
    .into_iter()
    .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
    .collect();
let title = preview_labels
    .iter()
    .find(|label| label.has_css_class("nn-preview-title"))
    .expect("preview title");
let metadata = preview_labels
    .iter()
    .find(|label| label.has_css_class("nn-preview-metadata"))
    .expect("preview metadata");
let body = preview_labels
    .iter()
    .find(|label| label.has_css_class("nn-preview-body"))
    .expect("preview body");
assert_eq!(title.wrap_mode(), gtk::pango::WrapMode::WordChar);
assert_eq!(metadata.wrap_mode(), gtk::pango::WrapMode::WordChar);
assert_eq!(body.wrap_mode(), gtk::pango::WrapMode::WordChar);
assert!(body.is_selectable());
```

Use a note body containing at least 300 uninterrupted ASCII characters plus Bangla and Arabic text so the real Pango wrapping properties are exercised.

Add these assertions to `editor_canvas.rs` after the existing Rich Text margin assertions, and confirm source values remain zero:

```rust
assert_eq!(rich_editor.pixels_above_lines(), 2);
assert_eq!(rich_editor.pixels_below_lines(), 2);
assert_eq!(rich_editor.pixels_inside_wrap(), 1);
assert_eq!(source_editor.pixels_above_lines(), 0);
assert_eq!(source_editor.pixels_below_lines(), 0);
assert_eq!(source_editor.pixels_inside_wrap(), 0);
```

- [ ] **Step 2: Run the preview/editor tests and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test library_ui --test editor_canvas --locked
```

Expected: `library_ui` FAILS on missing preview classes and `editor_canvas` FAILS on the new line-spacing assertions while still reporting the correct 5/8 margins.

- [ ] **Step 3: Add semantic labels, safe wrapping, and Pango line height**

In `NotePreview::new`, apply:

```rust
title.add_css_class("nn-preview-title");
title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
let title_attributes = gtk::pango::AttrList::new();
title_attributes.insert(gtk::pango::AttrFloat::new_line_height(1.2));
title.set_attributes(Some(&title_attributes));
metadata.add_css_class("nn-preview-metadata");
metadata.set_wrap(true);
metadata.set_wrap_mode(gtk::pango::WrapMode::WordChar);
body.add_css_class("nn-preview-body");
body.set_wrap_mode(gtk::pango::WrapMode::WordChar);
let attributes = gtk::pango::AttrList::new();
attributes.insert(gtk::pango::AttrFloat::new_line_height(1.6));
body.set_attributes(Some(&attributes));
```

Retain `maximum_size(860)`, `tightening_threshold(720)`, selectable body content, and all current note strings.

Extend the Rich Text branch of `configure_editor_canvas` without changing its margin tuple:

```rust
if rich_mode {
    editor.set_pixels_above_lines(2);
    editor.set_pixels_below_lines(2);
    editor.set_pixels_inside_wrap(1);
} else {
    editor.set_pixels_above_lines(0);
    editor.set_pixels_below_lines(0);
    editor.set_pixels_inside_wrap(0);
}
```

Use these CSS rules:

```css
.nn-preview-surface { background: @nn_surface; }
.nn-preview { background: @nn_surface; padding: 36px 48px; }
.nn-preview-title { font-size: 30px; font-weight: 700; color: @nn_text; }
.nn-preview-metadata { font-size: 13px; color: @nn_text_muted; }
.nn-preview-body { font-size: 16px; color: @nn_text; }
```

- [ ] **Step 4: Run preview, editor-margin, source-palette, and CSS tests GREEN**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test library_ui --test editor_canvas --test source_palettes --test design_system --locked
```

Expected: all pass; Rich Text remains top/bottom 5 and left/right 8; source palette ownership remains unchanged.

- [ ] **Step 5: Commit preview ergonomics**

```bash
git add apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/src/ui/editor_canvas.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/library_ui.rs apps/noor-notes/tests/editor_canvas.rs
git commit -m "style: improve light preview reading ergonomics"
```

---

### Task 5: Unify application header, search, sort, and status chrome

**Files:**
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs:70-164`
- Modify: `apps/noor-notes/src/ui/editor_header.rs`
- Modify: `apps/noor-notes/src/ui/editor_status_bar.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: existing Libadwaita header, menu/action names, `AppearanceButton`, `GtkDropDown`, `GtkSearchBar`, and shared `nn-statusbar`.
- Produces: `nn-app-header`, `nn-header-control`, `nn-new-note`, `nn-sort-control`, and `nn-search-entry` classes without changing action destinations.

- [ ] **Step 1: Write failing header/status CSS contracts**

Add to `design_system.rs`:

```rust
#[test]
fn light_header_search_sort_and_status_share_compact_chrome() {
    for rule in [
        ".nn-app-header { min-height: 44px;",
        ".nn-header-control { min-width: 36px; min-height: 36px;",
        ".nn-new-note { min-height: 36px;",
        ".nn-sort-control { min-height: 36px;",
        ".nn-search-entry { min-height: 36px;",
        ".nn-statusbar { min-height: 30px;",
        ".nn-theme-light scrollbar slider {",
    ] {
        assert!(CSS.contains(rule), "missing compact chrome rule: {rule}");
    }
    assert!(CSS.contains(".nn-theme-light windowcontrols button"));
}
```

- [ ] **Step 2: Run the design-system test and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test design_system --locked
```

Expected: FAIL because the compact chrome classes do not exist.

- [ ] **Step 3: Assign semantic classes without changing control construction**

In `library_window.rs`:

```rust
header.add_css_class("nn-app-header");
new_button.add_css_class("nn-new-note");
search_button.add_css_class("flat");
search_button.add_css_class("nn-header-control");
search_button.add_css_class("nn-icon-neutral");
sort.add_css_class("nn-sort-control");
search.add_css_class("nn-search-entry");
```

Store the application `MenuButton` in a local variable before packing it, then add `flat`, `nn-header-control`, and `nn-icon-neutral`. Add `nn-header-control` to the existing appearance button. Keep every `action_name`, menu item, tooltip, binding, and sort index unchanged.

In `editor_header.rs`, add `nn-header-control` and the appropriate `nn-icon-neutral`/`nn-icon-active` classes to pin, favorite, appearance, archive, and Trash controls without changing their callbacks. Keep the existing accessible labels.

- [ ] **Step 4: Apply compact semantic chrome CSS**

```css
.nn-app-header { min-height: 44px; background: @nn_surface; border-bottom: 1px solid @nn_border_subtle; box-shadow: none; }
.nn-header-control { min-width: 36px; min-height: 36px; padding: 0 8px; border-radius: 8px; color: @nn_text_secondary; }
.nn-header-control:hover { background: @nn_hover; color: @nn_text; }
.nn-header-control:checked { background: @nn_accent_soft; color: @nn_accent; }
.nn-header-control image { -gtk-icon-size: 18px; }
.nn-new-note { min-height: 36px; padding: 0 12px; border-radius: 9px; }
.nn-sort-control { min-height: 36px; padding: 0 8px; border: 1px solid transparent; border-radius: 8px; background: @nn_hover; color: @nn_text; }
.nn-search-entry { min-height: 36px; border-radius: 8px; border-color: @nn_border; background: @nn_surface; color: @nn_text; }
.nn-statusbar { min-height: 30px; padding: 0 12px; background: @nn_note_list_bg; border-top: 1px solid @nn_border_subtle; font-size: 12px; color: @nn_text_muted; }
.nn-theme-light windowcontrols button { color: @nn_text_secondary; background: transparent; }
.nn-theme-light windowcontrols button:hover { color: @nn_text; background: @nn_hover; }
.nn-theme-light scrollbar slider { min-width: 6px; min-height: 6px; background: @nn_scrollbar; border-radius: 999px; }
.nn-theme-light scrollbar slider:hover { background: @nn_scrollbar_hover; }
```

Retain explicit dark header, toolbar, entry, and status overrides. Add dark `.nn-header-control` overrides only if visual parsing or screenshots show shared Light values winning through specificity.

- [ ] **Step 5: Run header, accessibility, editor, and CSS tests GREEN**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test design_system --test accessibility --test note_titles --test editor_status --locked
```

Expected: all tests pass with valid GTK CSS and unchanged editor title/status behavior.

- [ ] **Step 6: Commit unified chrome**

```bash
git add apps/noor-notes/src/ui/library_window.rs apps/noor-notes/src/ui/editor_header.rs apps/noor-notes/src/ui/editor_status_bar.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/design_system.rs
git commit -m "style: unify light application chrome"
```

---

### Task 6: Correct narrow-window responsiveness without hiding functionality

**Files:**
- Modify: `apps/noor-notes/tests/adaptive_layout.rs`
- Modify: `apps/noor-notes/src/ui/adaptive_layout.rs`
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/resources/design-system.css`

**Interfaces:**
- Consumes: `LibraryLayoutMode::for_window_width`, `LibraryLayoutMode::visibility`, existing Back navigation, and `NotePreview`.
- Produces: real post-map width detection, `NotePreview::set_compact(bool)`, compact preview padding below 760 pixels, and unchanged wide/medium/narrow visibility semantics.

- [ ] **Step 1: Write a failing regression test for the default-width masking bug**

Replace the final assertion in `width_breakpoints_choose_one_stable_library_mode` and add initial-allocation coverage:

```rust
assert_eq!(
    LibraryLayoutMode::for_window_width(720, 1_180),
    LibraryLayoutMode::Narrow
);
assert_eq!(
    LibraryLayoutMode::for_window_width(1, 1_180),
    LibraryLayoutMode::Wide
);
```

In `library_ui.rs`, add:

```rust
let preview = NotePreview::new();
preview.set_compact(true);
assert!(preview.widget.has_css_class("compact"));
preview.set_compact(false);
assert!(!preview.widget.has_css_class("compact"));
```

- [ ] **Step 2: Run responsive tests and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test adaptive_layout --test library_ui --locked
```

Expected: `adaptive_layout` FAILS because a real 720-pixel allocation is incorrectly replaced by the 1180-pixel default; `library_ui` fails because `set_compact` does not exist.

- [ ] **Step 3: Fix width selection at the source**

Implement:

```rust
pub const fn for_window_width(allocated: i32, configured_default: i32) -> Self {
    let width = if allocated <= 1 {
        configured_default
    } else {
        allocated
    };
    Self::for_width(width)
}
```

This preserves the pre-map sentinel behavior while allowing actual resized windows to become Medium or Narrow.

- [ ] **Step 4: Add compact preview state and wire it to layout**

Add to `NotePreview`:

```rust
pub fn set_compact(&self, compact: bool) {
    if compact {
        self.widget.add_css_class("compact");
    } else {
        self.widget.remove_css_class("compact");
    }
}
```

In `MainWindow::apply_layout`, call:

```rust
self.preview.set_compact(mode == LibraryLayoutMode::Narrow);
```

Use:

```css
.nn-preview-surface.compact .nn-preview { padding: 28px 32px; }
```

Keep sidebar-first collapse, list/preview medium mode, narrow list-to-preview switching, Back behavior, and all header controls. Do not hide sort, search, New Note, or menu functionality.

- [ ] **Step 5: Run responsive and component tests GREEN**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test adaptive_layout --test library_ui --test library_state --locked
```

Expected: all responsive breakpoints, note projection, and compact preview tests pass.

- [ ] **Step 6: Commit responsive behavior**

```bash
git add apps/noor-notes/src/ui/adaptive_layout.rs apps/noor-notes/src/ui/library_window.rs apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/adaptive_layout.rs apps/noor-notes/tests/library_ui.rs
git commit -m "fix: honor responsive library widths"
```

---

### Task 7: Complete Dark Mode regression, visual review, and production verification

**Files:**
- Modify only if a verified regression exists: files already changed in Tasks 1–6
- Track: `docs/superpowers/plans/2026-08-17-light-mode-ui-refresh.md`
- Create temporarily, then remove: `apps/noor-notes/examples/light_mode_review.rs`
- Create only under `/tmp`: `/tmp/noor-notes-light-review/*.png`

**Interfaces:**
- Consumes: completed production UI, existing public UI components, theme manager, GTK CSS parser, and repository verification scripts.
- Produces: visual evidence from synthetic notes without opening personal data; a clean tracked tree containing only production/test/docs changes.

- [ ] **Step 1: Run formatting, strict lint, focused UI suite, and release build**

Run each command and stop at the first failure:

```bash
cargo fmt --all -- --check
cargo +1.85.0 check --workspace --locked
cargo +1.85.0 clippy --workspace --all-targets --locked -- -D warnings
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test design_system --test library_ui --test adaptive_layout --test accessibility --test note_card_archive --test trash_actions --test editor_canvas --test source_palettes --locked
cargo +1.85.0 build --workspace --release --locked
```

Expected: every command exits zero. EGL/DRI3 warnings from Xvfb are acceptable only when tests still report zero failures.

- [ ] **Step 2: Create a bounded temporary visual-review harness**

Use `apply_patch` to create `apps/noor-notes/examples/light_mode_review.rs`. The harness must:

```rust
use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{Note, NoteColor};
use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore};
use noor_notes::ui::library_sidebar::LibrarySidebar;
use noor_notes::ui::note_collection::NoteCollection;
use noor_notes::ui::note_preview::NotePreview;

fn main() -> gtk::glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.LightReview")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(include_str!("../resources/design-system.css"));
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });
    app.connect_activate(|app| {
        let review_root = std::path::PathBuf::from("/tmp/noor-notes-light-review");
        let appearance = AppearanceManager::new(AppearanceStore::at(
            review_root.join("appearance.json"),
        ));
        let requested = std::env::var("NOOR_REVIEW_THEME").unwrap_or_else(|_| "light".into());
        let mode = if requested == "graphite" {
            AppearanceMode::Graphite
        } else {
            AppearanceMode::Light
        };
        appearance.set_mode(mode).expect("set isolated review theme");

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Noor Notes Light Review")
            .default_width(1180)
            .default_height(760)
            .build();
        appearance.register_window(&window);
        let sidebar = LibrarySidebar::new();
        let collection = NoteCollection::new(Rc::new(|_, _| {}));
        let preview = NotePreview::new();
        let mut notes = Vec::new();
        for (index, color) in [
            NoteColor::Blue,
            NoteColor::Yellow,
            NoteColor::Green,
            NoteColor::Lavender,
        ]
        .into_iter()
        .enumerate()
        {
            let mut note = Note::new(Utc::now());
            note.color = color;
            note.title = format!("Support workflow {}", index + 1);
            note.content = if index == 0 {
                format!("Client details\n{}\nবাংলা العربية", "A".repeat(300))
            } else {
                "A calm two-line note preview with representative content.".into()
            };
            if index == 0 {
                preview.show_note(&note);
            }
            notes.push(note);
        }
        collection.set_notes(&notes);
        let list_scroll = gtk::ScrolledWindow::builder()
            .width_request(336)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&collection.widget)
            .build();
        let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        navigation.append(&sidebar.widget);
        navigation.append(&list_scroll);
        let panes = gtk::Paned::new(gtk::Orientation::Horizontal);
        panes.set_start_child(Some(&navigation));
        panes.set_end_child(Some(&preview.widget));
        panes.set_position(569);
        window.set_content(Some(&panes));
        window.present();
    });
    app.run()
}
```

Build it with:

```bash
cargo +1.85.0 build -p noor-notes --example light_mode_review --locked
```

Expected: the temporary harness compiles without touching the normal database or appearance file.

- [ ] **Step 3: Capture and inspect isolated Light and Graphite windows**

Create `/tmp/noor-notes-light-review`, run the harness once with `NOOR_REVIEW_THEME=light` and once with `NOOR_REVIEW_THEME=graphite`, and capture only its active window through `org.gnome.Shell.Screenshot.ScreenshotWindow` to:

```bash
mkdir -p /tmp/noor-notes-light-review
NOOR_REVIEW_THEME=light target/debug/examples/light_mode_review >/tmp/noor-notes-light-review/light.log 2>&1 &
light_review_pid=$!
gdbus call --session --dest org.gnome.Shell.Screenshot --object-path /org/gnome/Shell/Screenshot --method org.gnome.Shell.Screenshot.ScreenshotWindow true false false /tmp/noor-notes-light-review/light.png
kill "$light_review_pid"
wait "$light_review_pid" || true
NOOR_REVIEW_THEME=graphite target/debug/examples/light_mode_review >/tmp/noor-notes-light-review/graphite.log 2>&1 &
graphite_review_pid=$!
gdbus call --session --dest org.gnome.Shell.Screenshot --object-path /org/gnome/Shell/Screenshot --method org.gnome.Shell.Screenshot.ScreenshotWindow true false false /tmp/noor-notes-light-review/graphite.png
kill "$graphite_review_pid"
wait "$graphite_review_pid" || true
```

Wait until the review window is visible and active before each `gdbus` call. Inspect both images with the local image viewer. Verify pane separation, selected/colored card restraint, icon contrast, long-string wrapping, preview padding, and no Light token leakage in Graphite. Exercise the real app at 1180, 900, 760, and 620 pixels without capturing personal note content; automated `adaptive_layout` and component tests remain the evidence for states that cannot be captured safely.

- [ ] **Step 4: Remove all capture-only material**

Remove only the temporary example through `apply_patch`, then move the exact review directory to the recoverable desktop trash:

```bash
gio trash /tmp/noor-notes-light-review
```

Confirm no example, PNG, appearance file, database, or log is tracked or staged.

- [ ] **Step 5: Run the full exact-tree verification**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test --workspace --locked
bash tests/install_ubuntu.sh
bash tests/screenshot_gallery.sh
git diff --check
git status --short
```

Expected: all tests and contracts pass. `git status --short` lists only intended tracked changes plus the two pre-existing untracked Snap artifacts.

- [ ] **Step 6: Commit any verified visual-regression fixes and the plan**

If Step 3 required a production adjustment, first add its focused failing test, verify RED, apply the minimal fix, and rerun the focused and full gates. Then stage only intended source/tests/CSS plus this plan:

```bash
git add apps/noor-notes/src/ui apps/noor-notes/resources/design-system.css apps/noor-notes/tests docs/superpowers/plans/2026-08-17-light-mode-ui-refresh.md
git commit -m "test: verify light mode UI refresh"
```

If no post-review production adjustment exists, stage and commit only the plan:

```bash
git add docs/superpowers/plans/2026-08-17-light-mode-ui-refresh.md
git commit -m "docs: record light mode verification"
```
