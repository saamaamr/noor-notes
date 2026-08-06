# Responsive Editor Toolbar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep every editor action usable in narrow and short note windows by wrapping the toolbar into rows and the More popover into columns.

**Architecture:** Replace the editor toolbar's fixed horizontal `gtk::Box` with a non-selecting horizontal `gtk::FlowBox`. Replace the More popover's tall action box with a vertical `gtk::FlowBox` capped at six rows per column, while retaining a separate wrapping editor-mode footer.

**Tech Stack:** Rust 1.87, GTK4, libadwaita, Cargo integration tests, GTK CSS

## Global Constraints

- Preserve every existing editor action, shortcut, tooltip, enabled state, and signal connection.
- Keep `View Only` directly visible in `More note actions`.
- Do not change note data, database schemas, application identity, dependencies, Snap packaging, or Snap Store state.
- Do not touch the existing untracked Snap artifacts.

---

### Task 1: Wrap the primary editor toolbar

**Files:**
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs:4-330`
- Modify: `apps/noor-notes/tests/rich_editor_ui.rs:1-30`

**Interfaces:**
- Produces: `EditorToolbar::widget: gtk::FlowBox`
- Produces: `EditorToolbar::more: gtk::MenuButton`
- Preserves: every existing public action field on `EditorToolbar`

- [ ] **Step 1: Write the failing responsive-layout test**

Extend `compact_toolbar_exposes_only_frequent_actions_at_top_level`:

```rust
assert_eq!(toolbar.widget.selection_mode(), gtk::SelectionMode::None);
assert_eq!(toolbar.widget.max_children_per_line(), 9);
assert_eq!(toolbar.widget.observe_children().n_items(), 9);
assert!(toolbar.more.is_visible());

let (_, narrow_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 190);
let (_, wide_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 900);
assert!(narrow_height > wide_height, "narrow toolbars must wrap into rows");
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_editor_ui
```

Expected: compilation fails because `widget` is a `gtk::Box` and `more` is not exposed, or the wrapping-height assertion fails.

- [ ] **Step 3: Replace the fixed Box with FlowBox**

In `EditorToolbar`, change the widget type and expose the More button:

```rust
pub widget: gtk::FlowBox,
pub more: gtk::MenuButton,
```

Construct the container with native wrapping:

```rust
let widget = gtk::FlowBox::builder()
    .selection_mode(gtk::SelectionMode::None)
    .min_children_per_line(1)
    .max_children_per_line(9)
    .column_spacing(2)
    .row_spacing(2)
    .hexpand(true)
    .build();
widget.add_css_class("nn-editor-toolbar");
```

Remove the non-wrapping `left`, `center`, spacer, `right`, and toolbar separators. After the More button is constructed, insert each frequent action directly in order:

```rust
for action in [
    undo.upcast_ref::<gtk::Widget>(),
    redo.upcast_ref(),
    find.upcast_ref(),
    bold.upcast_ref(),
    italic.upcast_ref(),
    bullets.upcast_ref(),
    format.upcast_ref(),
    emoji.upcast_ref(),
    more.upcast_ref(),
] {
    widget.insert(action, -1);
}
```

Return `more` from `EditorToolbar::new` without changing any signal wiring.

- [ ] **Step 4: Run the focused test and verify GREEN**

Run the Step 2 command. Expected: PASS, with narrow measured height greater than wide height.

- [ ] **Step 5: Commit the responsive toolbar**

```bash
git add apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/tests/rich_editor_ui.rs
git commit -m "fix: wrap editor actions in narrow windows"
```

---

### Task 2: Flow More actions into columns

**Files:**
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs:230-330`
- Modify: `apps/noor-notes/tests/view_only_mode.rs:16-30`

**Interfaces:**
- Produces: `EditorToolbar::more_actions: gtk::FlowBox`
- Consumes: existing note-action buttons and nested `export` and `view` menu buttons
- Preserves: `EditorToolbar::view_only: gtk::Button` as a direct main-More action

- [ ] **Step 1: Write the failing multi-column contract test**

Add a GTK test using the real toolbar:

```rust
#[test]
fn more_actions_are_height_bounded_and_can_flow_into_columns() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    assert_eq!(toolbar.more_actions.orientation(), gtk::Orientation::Vertical);
    assert_eq!(toolbar.more_actions.selection_mode(), gtk::SelectionMode::None);
    assert_eq!(toolbar.more_actions.max_children_per_line(), 6);
    assert!(toolbar.more_actions.observe_children().n_items() >= 9);
    assert!(toolbar.view_only.is_ancestor(&toolbar.more_actions));
}
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test view_only_mode
```

Expected: compilation fails because `more_actions` does not exist.

- [ ] **Step 3: Implement the bounded vertical action flow**

Expose and construct the action flow:

```rust
pub more_actions: gtk::FlowBox,

let more_actions = gtk::FlowBox::builder()
    .orientation(gtk::Orientation::Vertical)
    .selection_mode(gtk::SelectionMode::None)
    .min_children_per_line(1)
    .max_children_per_line(6)
    .column_spacing(6)
    .row_spacing(4)
    .build();
more_actions.add_css_class("nn-more-actions");
```

Insert `new_note`, `rename`, `duplicate`, `pin`, `view_only`, `archive`, `trash`, `restore`, `permanent_delete`, `export`, and `view` in that order. Place this flow at the top of the existing More popover.

Build the editor-mode footer as its own non-selecting horizontal FlowBox with four items per line so it can wrap independently. Keep the mode label and separator above it. Return `more_actions` from the toolbar.

- [ ] **Step 4: Run focused GTK tests and verify GREEN**

Run:
