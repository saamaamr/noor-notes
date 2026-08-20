# Professional UI Phase 2: Editor Commands and Transient Surfaces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the editor chrome and transient surfaces with the approved compact professional components while making every visible menu, toolbar, formatting, list, emoji, search, and mode control capability-aware and fully wired through the existing document mutation and autosave paths.

**Architecture:** Keep `RichBuffer`, the source adapters, `AutosaveController`, and note conversion as the functional authorities. Extend the existing `EditorCommand` metadata, use `EditorToolbar` as a compatibility facade around reusable command groups, and make `EditorMenuBar` proxy the same live controls instead of implementing duplicate mutations. `NotePreview` and `NoteWindow` consume the same shared components and capability rules.

**Tech Stack:** Rust 1.85, GTK4, Libadwaita, GtkSourceView, existing `RichBuffer`, existing editor adapters, and GTK integration tests under Xvfb.

**Spec:** `docs/superpowers/specs/2026-08-20-professional-product-ui-redesign-design.md`

## Global Constraints

- Do not add Link, Heading, Quote, Code Block, comment, indentation, language, or other commands without a real adapter implementation.
- Preserve font sizes 12, 14, 16, 18, and 24 plus the existing validated custom-size path.
- Preserve Rich Text internal margins of 8 pixels horizontally and 5 pixels vertically.
- Do not implement independent toolbar, menu, and shortcut mutation paths.
- Read-only state must block pointer and keyboard mutations.
- Every mutation must flow through the existing buffer-change, note snapshot, and autosave path.
- Keep existing source palettes and safe mode-conversion confirmation/recovery behavior.
- Do not modify or stage unrelated worktree changes.

---

### Task 1: Make Editor Command Metadata the Capability Authority

**Files:**
- Modify: `apps/noor-notes/src/editor_commands.rs`
- Modify: `apps/noor-notes/src/editor/adapter.rs`
- Modify: `apps/noor-notes/tests/editor_command_capabilities.rs`

**Interfaces:**
- Consumes: `EditorMode`, `AdapterCapabilities`, edit/read-only state.
- Produces: `EditorCommandSpec`, `spec(command)`, and `is_available(command, mode, capabilities, editable)` used by toolbar, menus, and shortcuts.

- [ ] **Step 1: Write failing command-contract tests**

```rust
#[test]
fn every_editor_command_has_one_real_capability_contract() {
    let bold = spec(EditorCommand::Bold);
    assert_eq!(bold.id, "bold");
    assert_eq!(bold.label, "Bold");
    assert_eq!(bold.shortcut, Some("Ctrl+B"));
    assert!(bold.mutates_document);

    let emoji = spec(EditorCommand::InsertEmoji);
    assert!(supports_command(&EditorMode::Rich, emoji.command));
    assert!(supports_command(&EditorMode::Markdown, emoji.command));
    assert!(!supports_command(&EditorMode::Code, emoji.command));
}

#[test]
fn read_only_blocks_all_document_mutations_but_not_view_commands() {
    let capabilities = AdapterCapabilities::all();
    assert!(!is_available(
        EditorCommand::Bold,
        &EditorMode::Rich,
        capabilities,
        false,
    ));
    assert!(!is_available(
        EditorCommand::Undo,
        &EditorMode::Rich,
        capabilities,
        false,
    ));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test editor_command_capabilities -- --nocapture
```

Expected: FAIL because command specifications and live availability do not exist.

- [ ] **Step 3: Add command metadata without inventing actions**

Add a compact descriptor for the current real commands:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorCommandSpec {
    pub command: EditorCommand,
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub mutates_document: bool,
}

pub const fn spec(command: EditorCommand) -> EditorCommandSpec {
    match command {
        EditorCommand::Undo => EditorCommandSpec {
            command, id: "undo", label: "Undo", shortcut: Some("Ctrl+Z"), mutates_document: true,
        },
        EditorCommand::Redo => EditorCommandSpec {
            command, id: "redo", label: "Redo", shortcut: Some("Ctrl+Shift+Z"), mutates_document: true,
        },
        EditorCommand::Bold => EditorCommandSpec {
            command,
            id: "bold",
            label: "Bold",
            shortcut: Some("Ctrl+B"),
            mutates_document: true,
        },
        EditorCommand::Italic => EditorCommandSpec {
            command, id: "italic", label: "Italic", shortcut: Some("Ctrl+I"), mutates_document: true,
        },
        EditorCommand::Underline => EditorCommandSpec {
            command, id: "underline", label: "Underline", shortcut: Some("Ctrl+U"), mutates_document: true,
        },
        EditorCommand::Strikethrough => EditorCommandSpec {
            command, id: "strikethrough", label: "Strikethrough", shortcut: None, mutates_document: true,
        },
        EditorCommand::ToggleBulletList => EditorCommandSpec {
            command, id: "bullet-list", label: "Bullet List", shortcut: None, mutates_document: true,
        },
        EditorCommand::ToggleNumberedList => EditorCommandSpec {
            command, id: "numbered-list", label: "Numbered List", shortcut: None, mutates_document: true,
        },
        EditorCommand::ClearFormatting => EditorCommandSpec {
            command, id: "clear-formatting", label: "Clear Formatting", shortcut: None, mutates_document: true,
        },
        EditorCommand::InsertEmoji => EditorCommandSpec {
            command, id: "insert-emoji", label: "Emoji", shortcut: None, mutates_document: true,
        },
        EditorCommand::FontSize => EditorCommandSpec {
            command, id: "font-size", label: "Font Size", shortcut: None, mutates_document: true,
        },
    }
}
```

Implement `is_available` from `supports_command`, adapter capability fields, and edit state. Keep view/search/application actions outside this document-mutation enum unless they already have a real adapter command.

- [ ] **Step 4: Run the capability tests and verify GREEN**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test editor_command_capabilities
```

- [ ] **Step 5: Commit the command contracts**

```bash
git add apps/noor-notes/src/editor_commands.rs apps/noor-notes/src/editor/adapter.rs apps/noor-notes/tests/editor_command_capabilities.rs
git commit -m "editor: centralize command capability metadata"
```

### Task 2: Replace Toolbar Construction with Reusable Compact Groups

**Files:**
- Create: `apps/noor-notes/src/ui/toolbar_primitives.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/tests/rich_editor_ui.rs`
- Modify: `apps/noor-notes/tests/editor_presentation.rs`

**Interfaces:**
- Consumes: current `EditorToolbar` public controls and the semantic control classes from Phase 1.
- Produces: `ToolbarGroup`, shared icon/text toggle builders, and a single-line content-fit toolbar with priority classes for compact layouts.

- [ ] **Step 1: Write failing presentation tests**

```rust
#[test]
fn professional_toolbar_is_content_fit_grouped_and_never_wraps() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    assert!(toolbar.widget.has_css_class("nn-command-bar"));
    assert!(!toolbar.widget.hexpands());
    assert_eq!(toolbar.group_count(), 5);
    assert_eq!(toolbar.format.icon_name().as_deref(), Some("format-text-rich-symbolic"));
    assert_eq!(toolbar.format.tooltip_text().as_deref(), Some("Formatting"));

    let (_, narrow_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 420);
    let (_, wide_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 1000);
    assert_eq!(narrow_height, wide_height);
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_editor_ui professional_toolbar -- --nocapture
```

- [ ] **Step 3: Introduce reusable toolbar primitives**

Create builders that enforce a 32–36 pixel interaction target, accessible label, tooltip, focusability, and one active-state class. Recompose `EditorToolbar::new` into five groups:

```text
History | Typography | Inline formatting | Lists/Insert | More
```

Keep public widget fields until both `NotePreview` and `NoteWindow` are migrated. Remove the unused visible Style control; the current `RichDocument` has no block-style model. The formatting control remains icon-only.

- [ ] **Step 4: Add priority behavior without multi-row wrapping**

Add CSS priority classes so secondary controls hide or move behind `More` at compact allocations. Do not duplicate commands in two simultaneously visible groups.

- [ ] **Step 5: Run toolbar and presentation tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_editor_ui --test editor_presentation
```

- [ ] **Step 6: Commit the compact command bar**

```bash
git add apps/noor-notes/src/ui/toolbar_primitives.rs apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/tests/rich_editor_ui.rs apps/noor-notes/tests/editor_presentation.rs
git commit -m "ui: replace editor toolbar with compact command groups"
```

### Task 3: Synchronize Menu Proxies with Live Command State

**Files:**
- Modify: `apps/noor-notes/src/ui/editor_menu_bar.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`
- Create: `apps/noor-notes/tests/editor_menu_bar.rs`

**Interfaces:**
- Consumes: real toolbar controls and their visible, sensitive, active, and tooltip state.
- Produces: live menu proxies that invoke the same source control and close their own popover before modal or nested transient work.

- [ ] **Step 1: Write failing proxy synchronization tests**

```rust
#[test]
fn menu_items_follow_source_command_availability_and_checked_state() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    let menu = EditorMenuBar::new(&toolbar);
    toolbar.set_editor_mode(EditorMode::PlainText);
    assert!(!menu.item("format.bold").is_visible());

    toolbar.undo.set_sensitive(false);
    assert!(!menu.item("edit.undo").is_sensitive());

    toolbar.word_wrap.set_active(false);
    menu.item("view.word-wrap").emit_clicked();
    assert!(toolbar.word_wrap.is_active());
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test editor_menu_bar -- --nocapture
```

- [ ] **Step 3: Add keyed reusable menu items**

Replace anonymous cloned buttons with a keyed `EditorMenuItem` registry. Bind `visible` and `sensitive` from the real source widget. For toggles, bind or refresh checked state and expose it semantically. Menu activation must emit the real source click/toggle, then close the current popover.

Keep only supported contents:

- File: New Note, Duplicate, real Export choices, lifecycle-appropriate Trash/Delete.
- Edit: Undo, Redo, Find.
- View: Word Wrap, Zoom controls, View Only where the host supports them.
- Insert: Emoji outside Code mode.
- Format: real Rich formatting/list controls.
- Tools: Go to Line, Editor Mode, and real More actions in the standalone editor.

- [ ] **Step 4: Add mutual-exclusion behavior**

Opening one editor menu/popover must close any already-open editor transient. Escape closes the active transient and restores focus to its trigger/editor.

- [ ] **Step 5: Run menu and action tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test editor_menu_bar --test toolbar_actions
```

- [ ] **Step 6: Commit live menu proxies**

```bash
git add apps/noor-notes/src/ui/editor_menu_bar.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/tests/editor_menu_bar.rs apps/noor-notes/tests/toolbar_actions.rs
git commit -m "ui: synchronize editor menus with command state"
```

### Task 4: Centralize Selection Preservation and Active Formatting State

**Files:**
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/src/rich_buffer.rs`
- Modify: `apps/noor-notes/tests/editor_history.rs`
- Modify: `apps/noor-notes/tests/rich_editor.rs`
- Modify: `apps/noor-notes/tests/rich_editor_ui.rs`

**Interfaces:**
- Consumes: `gtk::TextBuffer` selection/cursor, toolbar trigger focus changes, and `TextMarks` at the cursor.
- Produces: `SavedTextRange`, `restore_range`, and one command dispatch/synchronization path for formatting actions.

- [ ] **Step 1: Write failing selection/history tests**

```rust
#[test]
fn toolbar_focus_does_not_collapse_the_formatting_target() {
    let buffer = prepared("alpha beta");
    buffer.select_range(&buffer.iter_at_offset(0), &buffer.iter_at_offset(5));
    let saved = SavedTextRange::capture(&buffer);
    buffer.place_cursor(&buffer.end_iter());
    saved.restore(&buffer);
    execute(EditorCommand::Bold, &buffer, None);
    assert!(snapshot_marks(&buffer, 1).bold);
}

#[test]
fn formatting_undo_and_redo_use_the_native_buffer_history() {
    let buffer = prepared("alpha");
    select_all(&buffer);
    execute(EditorCommand::Underline, &buffer, None);
    execute(EditorCommand::Undo, &buffer, None);
    assert!(!snapshot_marks(&buffer, 1).underline);
    execute(EditorCommand::Redo, &buffer, None);
    assert!(snapshot_marks(&buffer, 1).underline);
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test editor_history --test rich_editor toolbar_focus -- --nocapture
```

- [ ] **Step 3: Implement a single selection guard**

Capture selection before a toolbar/menu/popover grabs focus, restore it immediately before a formatting mutation, and restore the logical cursor/focus after execution. Keep offsets clamped to the current buffer. Do not duplicate offset handling per button.

- [ ] **Step 4: Synchronize all checked controls from the buffer**

On selection/cursor mark changes, update Bold, Italic, Underline, Strikethrough, list kind, size, alignment, foreground, and highlight controls under one `syncing` guard. A mixed selection uses a neutral state.

- [ ] **Step 5: Run rich editor and history tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test editor_history --test rich_editor --test rich_editor_ui
```

- [ ] **Step 6: Commit selection and state synchronization**

```bash
git add apps/noor-notes/src/editor_actions.rs apps/noor-notes/src/rich_buffer.rs apps/noor-notes/tests/editor_history.rs apps/noor-notes/tests/rich_editor.rs apps/noor-notes/tests/rich_editor_ui.rs
git commit -m "editor: preserve selection and synchronize format state"
```

### Task 5: Professionalize and Fully Wire the Formatting Popover

**Files:**
- Modify: `apps/noor-notes/src/ui/formatting_popover.rs`
- Modify: `apps/noor-notes/src/ui/rich_color_palette.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/tests/rich_colors.rs`
- Modify: `apps/noor-notes/tests/rich_formatting_persistence.rs`
- Modify: `apps/noor-notes/tests/rich_editor_ui.rs`

**Interfaces:**
- Consumes: real Rich formatting commands, existing preset/custom color tags, current selection guard.
- Produces: grouped Typography, Formatting, Alignment, Colors, Lists, and Clear Formatting controls with no dead widgets.

- [ ] **Step 1: Write failing popover behavior tests**

```rust
#[test]
fn formatting_popover_contains_only_supported_functional_groups() {
    gtk::init().unwrap();
    let popover = FormattingPopover::new();
    assert_eq!(popover.section_names(), [
        "Typography", "Formatting", "Alignment", "Colors", "Lists"
    ]);
    assert_eq!(popover.font_size.model().unwrap().n_items(), 5);
    assert_eq!(popover.foreground_palette.automatic.tooltip_text().as_deref(), Some("Automatic text color"));
    assert_eq!(popover.highlight_palette.automatic.tooltip_text().as_deref(), Some("No highlight"));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_editor_ui formatting_popover -- --nocapture
```

- [ ] **Step 3: Recompose supported controls**

Use compact section labels and grouped rows. Keep the preset sizes and validated custom entry. Give all swatches accessible color-role names, a non-color selected indicator, and explicit Automatic/No Highlight controls. Move numbered/bullet controls into the Lists group while retaining their existing command wiring.

- [ ] **Step 4: Verify mutations persist through `RichDocument`**

Test text size, alignment, foreground, highlight, lists, and clear formatting by snapshotting, serializing, loading a new buffer, and comparing marks/blocks. Do not write storage from popover handlers.

- [ ] **Step 5: Run formatting tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_colors --test rich_formatting_persistence --test rich_editor_ui
```

- [ ] **Step 6: Commit the formatting surface**

```bash
git add apps/noor-notes/src/ui/formatting_popover.rs apps/noor-notes/src/ui/rich_color_palette.rs apps/noor-notes/src/editor_actions.rs apps/noor-notes/tests/rich_colors.rs apps/noor-notes/tests/rich_formatting_persistence.rs apps/noor-notes/tests/rich_editor_ui.rs
git commit -m "ui: complete professional rich formatting popover"
```

### Task 6: Make Lists, Emoji, and More Actions Reliable

**Files:**
- Modify: `apps/noor-notes/src/rich_buffer.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/tests/list_editing.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`
- Create: `apps/noor-notes/tests/emoji_insertion.rs`

**Interfaces:**
- Consumes: current list markers/continuation logic, emoji set, More actions, and editor focus.
- Produces: list toggle/continue/exit behavior, cursor-correct undoable emoji insertion, and modal-safe More actions.

- [ ] **Step 1: Add failing list and emoji tests**

```rust
#[test]
fn emoji_inserts_at_preserved_cursor_and_is_undoable() {
    let buffer = prepared("Hello  world");
    buffer.place_cursor(&buffer.iter_at_offset(6));
    execute(EditorCommand::InsertEmoji, &buffer, Some("😊"));
    assert_eq!(text(&buffer), "Hello 😊 world");
    assert_eq!(buffer.iter_at_mark(&buffer.get_insert()).offset(), 7);
    execute(EditorCommand::Undo, &buffer, None);
    assert_eq!(text(&buffer), "Hello  world");
}

#[test]
fn empty_list_item_exits_the_current_list() {
    let buffer = prepared("• item\n• ");
    buffer.place_cursor(&buffer.end_iter());
    assert!(RichBuffer::continue_list(&buffer));
    assert_eq!(text(&buffer), "• item\n");
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test list_editing --test emoji_insertion -- --nocapture
```

- [ ] **Step 3: Group each list/emoji mutation into one native user action**

Keep the cursor immediately after inserted content. Close emoji popover after selection and return focus to the editor. Continue and exit list behavior must use the existing marker model and preserve selected text.

- [ ] **Step 4: Remove dead More entries and prevent stuck grabs**

Build More from host-supported actions only. Pop down More before rename, export, mode confirmation, archive/trash confirmation, appearance, or secondary-window presentation. Disable lifecycle actions while one is already running.

- [ ] **Step 5: Run focused interaction tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test list_editing --test emoji_insertion --test toolbar_actions
```

- [ ] **Step 6: Commit list, emoji, and More behavior**

```bash
git add apps/noor-notes/src/rich_buffer.rs apps/noor-notes/src/editor_actions.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/tests/list_editing.rs apps/noor-notes/tests/emoji_insertion.rs apps/noor-notes/tests/toolbar_actions.rs
git commit -m "editor: harden lists emoji and more actions"
```

### Task 7: Preserve Saved Editor Modes in the Main Workspace

**Files:**
- Modify: `apps/noor-notes/src/ui/note_editor_surface.rs`
- Modify: `apps/noor-notes/src/ui/note_preview.rs`
- Modify: `apps/noor-notes/src/editor/session.rs`
- Modify: `apps/noor-notes/tests/note_preview_edit.rs`
- Modify: `apps/noor-notes/tests/preview_editor_surface.rs`
- Modify: `apps/noor-notes/tests/source_editor.rs`

**Interfaces:**
- Consumes: selected note's saved `EditorMode`, rich/source adapters, current edit/read transition, and preview autosave callback.
- Produces: a mode-aware primary workspace that reopens Rich, Markdown, Plain Text, and Code notes without forcing Rich mode.

- [ ] **Step 1: Write failing saved-mode tests**

```rust
#[test]
fn preview_reopens_a_code_note_in_its_saved_mode() {
    let mut note = fixture_note();
    note.editor_preferences.mode = EditorMode::Code;
    note.source_language = SourceLanguage::Rust;
    let preview = fixture_preview();
    preview.show_note(&note);
    assert_eq!(preview.active_mode(), EditorMode::Code);
    assert!(preview.source_view().is_visible());
    assert!(!preview.toolbar().bold.is_visible());
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test note_preview_edit preview_reopens -- --nocapture
```

- [ ] **Step 3: Route preview mode through the existing session/adapter boundary**

Reuse `NoteEditorSurface` and the existing rich/source adapter setup. Do not convert content merely by selecting a note. Mode conversion remains an explicit confirmed command. Preserve source language, word wrap, line behavior, and source palette.

- [ ] **Step 4: Keep read/edit title and body transitions coherent**

Reading mode uses a label title and selectable body. Edit mode uses the title entry and the active mode's real editor. Done snapshots through the current session and autosave callback. Trashed notes remain non-editable.

- [ ] **Step 5: Run preview and source tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test note_preview_edit --test preview_editor_surface --test source_editor
```

- [ ] **Step 6: Commit mode-aware primary editing**

```bash
git add apps/noor-notes/src/ui/note_editor_surface.rs apps/noor-notes/src/ui/note_preview.rs apps/noor-notes/src/editor/session.rs apps/noor-notes/tests/note_preview_edit.rs apps/noor-notes/tests/preview_editor_surface.rs apps/noor-notes/tests/source_editor.rs
git commit -m "editor: preserve saved modes in the main workspace"
```

### Task 8: Migrate the Standalone Editor to Shared Chrome

**Files:**
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/ui/editor_header.rs`
- Modify: `apps/noor-notes/src/ui/editor_status_bar.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`
- Modify: `apps/noor-notes/tests/editor_status.rs`
- Modify: `apps/noor-notes/tests/shortcuts.rs`

**Interfaces:**
- Consumes: shared command bar/menu/formatting components and existing standalone find/replace, zoom, export, lifecycle, conversion, and status handlers.
- Produces: one professional visual/command system across `NotePreview` and legacy `NoteWindow`.

- [ ] **Step 1: Write failing shared-chrome tests**

```rust
#[test]
fn standalone_editor_uses_shared_document_chrome() {
    assert!(NOTE_WINDOW.contains("EditorHeader::new"));
    assert!(NOTE_WINDOW.contains("EditorMenuBar::new"));
    assert!(NOTE_WINDOW.contains("EditorToolbar::new"));
    assert!(!NOTE_WINDOW.contains("FormattingPopover::new()"));
}
```

- [ ] **Step 2: Run and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test toolbar_actions standalone_editor -- --nocapture
```

- [ ] **Step 3: Replace standalone presentation without changing handlers**

Use the same document header, menu bar, command bar, canvas container, and status primitives. Move existing connections to the shared controls; do not duplicate export, lifecycle, mode conversion, find/replace, zoom, writing assistance, or window settings logic.

- [ ] **Step 4: Verify shortcuts call the same live controls/commands**

Ctrl+Z/Shift+Z/Y, Ctrl+B/I/U, Ctrl+F/H/G, zoom, F11, Escape, and read-only blocking must use the same command/action path as the toolbar/menu.

- [ ] **Step 5: Run standalone tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test toolbar_actions --test editor_status --test shortcuts
```

- [ ] **Step 6: Commit shared standalone chrome**

```bash
git add apps/noor-notes/src/note_window.rs apps/noor-notes/src/ui/editor_header.rs apps/noor-notes/src/ui/editor_status_bar.rs apps/noor-notes/tests/toolbar_actions.rs apps/noor-notes/tests/editor_status.rs apps/noor-notes/tests/shortcuts.rs
git commit -m "ui: share professional editor chrome across windows"
```

### Task 9: Run the Phase 2 Functional Gate

**Files:**
- Verify only; fix failures in the owning Phase 2 files.

- [ ] **Step 1: Format and check**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo check -p noor-notes
```

- [ ] **Step 2: Run command/editor tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes \
  --test editor_command_capabilities \
  --test editor_history \
  --test rich_editor \
  --test rich_editor_ui \
  --test rich_formatting_persistence \
  --test rich_colors \
  --test list_editing \
  --test emoji_insertion \
  --test editor_menu_bar \
  --test toolbar_actions \
  --test note_preview_edit \
  --test preview_editor_surface \
  --test source_editor \
  --test shortcuts
```

- [ ] **Step 3: Run persistence regressions**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test autosave --test library_preview_autosave
```

- [ ] **Step 4: Verify the diff**

```bash
git diff --check
git status --short
```

Expected: all Phase 2 checks pass; unrelated screenshot/Snap worktree changes remain untouched.
