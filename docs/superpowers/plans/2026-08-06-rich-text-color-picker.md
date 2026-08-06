# Rich Text Color Picker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add visible professional Rich Text color presets and native custom text/highlight pickers whose values persist safely across save, reopen, undo/redo, and theme changes.

**Architecture:** A new `rich_color` module owns semantic palettes, legacy compatibility, custom RGB validation, and tag encoding. A reusable `RichColorPalette` GTK component renders each preset/custom row, while `RichBuffer` remains the only layer that applies and serializes color marks. `NoteWindow` enables these controls only in Rich Text and updates adaptive preset tags when the appearance changes.

**Tech Stack:** Rust 1.85+, GTK4 4.14 via gtk4-rs 0.10, libadwaita, serde-backed `RichDocument`, existing SQLCipher repository and test framework.

## Global Constraints

- Rich Text mode only; Markdown, Plain Text, and Code retain GtkSourceView colors.
- Use GTK's native `ColorDialogButton`; add no new runtime dependency.
- Store custom colors as uppercase opaque `#RRGGBB`; alpha is unsupported.
- Preserve existing `charcoal`, `blue`, `green`, and `red` marks.
- Do not migrate or reset the database.
- Do not alter the application ID, package identity, Snap metadata, or Snap revisions.
- Do not build, upload, release, or modify any Snap.
- Do not add analytics, telemetry, remote assets, or network behavior.
- Keep the two root-level `.snap` files untracked and untouched.

---

### Task 1: Validated adaptive rich-color model

**Files:**
- Create: `apps/noor-notes/src/rich_color.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Modify: `apps/noor-notes/src/rich_buffer.rs`
- Test: `apps/noor-notes/tests/rich_colors.rs`
- Test: `apps/noor-notes/tests/rich_editor.rs`

**Interfaces:**
- Produces: `ColorRole::{Foreground, Highlight}`
- Produces: `ColorPreset { id, label, light, dark }`
- Produces: `presets(role) -> &'static [ColorPreset]`
- Produces: `normalize_stored(role, value) -> Option<String>`
- Produces: `rendered_color(role, value, theme) -> Option<String>`
- Produces: `tag_name(role, value) -> Option<String>`
- Produces: `stored_value_from_tag(role, name) -> Option<String>`
- Produces: `RichBuffer::{clear_foreground, clear_highlight, apply_color_theme}`

- [ ] **Step 1: Write failing palette and validation tests**

Create `apps/noor-notes/tests/rich_colors.rs` asserting:

```rust
use noor_notes::{
    appearance::EffectiveTheme,
    rich_color::{ColorRole, normalize_stored, presets, rendered_color, stored_value_from_tag, tag_name},
};

#[test]
fn professional_palettes_are_complete_and_custom_rgb_is_canonical() {
    assert_eq!(presets(ColorRole::Foreground).len(), 7);
    assert_eq!(presets(ColorRole::Highlight).len(), 7);
    assert_eq!(normalize_stored(ColorRole::Foreground, "#1a2b3c").as_deref(), Some("#1A2B3C"));
    assert_eq!(normalize_stored(ColorRole::Highlight, "not-a-color"), None);
    assert_eq!(normalize_stored(ColorRole::Foreground, "charcoal").as_deref(), Some("slate"));
}

#[test]
fn preset_rendering_is_theme_adaptive_and_custom_rgb_is_exact() {
    assert_eq!(rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Light).as_deref(), Some("#1D4ED8"));
    assert_eq!(rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Oled).as_deref(), Some("#93C5FD"));
    assert_eq!(rendered_color(ColorRole::Highlight, "#ABCDEF", EffectiveTheme::Midnight).as_deref(), Some("#ABCDEF"));
}

#[test]
fn tag_encoding_round_trips_without_embedding_untrusted_input() {
    let name = tag_name(ColorRole::Foreground, "#1A2B3C").unwrap();
    assert_eq!(name, "noor-fg-hex-1A2B3C");
    assert_eq!(stored_value_from_tag(ColorRole::Foreground, &name).as_deref(), Some("#1A2B3C"));
    assert!(tag_name(ColorRole::Highlight, "invalid value").is_none());
}
```

Extend `rich_editor.rs` with a selection that applies `#1A2B3C` foreground and `#F1E2D3` highlight, snapshots, reloads, and asserts identical marks.

- [ ] **Step 2: Run tests to verify RED**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_colors --test rich_editor
```

Expected: compilation fails because `rich_color` and custom-tag support do not exist.

- [ ] **Step 3: Implement the minimal color model**

Create immutable foreground and highlight preset tables matching the approved design. Normalize named presets, legacy aliases, and six-digit RGB only. Encode custom values as `noor-fg-hex-RRGGBB` or `noor-bg-hex-RRGGBB`; reject every other form.

Use `EffectiveTheme::Light` for light mappings and the dark mapping for Graphite, Midnight, and OLED. Keep a hidden legacy highlight-charcoal mapping and map legacy highlight-red to pink.

- [ ] **Step 4: Make RichBuffer create and reload dynamic tags**

Replace fixed-only color application with a helper that:

```rust
fn ensure_color_tag(
    buffer: &gtk::TextBuffer,
    role: ColorRole,
    stored: &str,
    theme: EffectiveTheme,
) -> Option<String>
```

The helper validates the value, creates the tag if absent, updates preset tag color for the current theme, and leaves custom RGB exact. `apply_marks` must call it before applying a foreground or highlight mark.

Wrap remove/apply operations in one `begin_user_action`/`end_user_action` pair. Add foreground-only and highlight-only reset methods. Preserve the public `foreground` and `highlight` entry points for existing callers.

- [ ] **Step 5: Run focused tests to verify GREEN**

Run the Task 1 command again. Expected: both test binaries pass with zero warnings.

- [ ] **Step 6: Commit Task 1**

```bash
git add apps/noor-notes/src/lib.rs apps/noor-notes/src/rich_color.rs apps/noor-notes/src/rich_buffer.rs apps/noor-notes/tests/rich_colors.rs apps/noor-notes/tests/rich_editor.rs
git commit -m "feat: add persistent adaptive rich colors"
```

### Task 2: Professional preset rows and native custom pickers

**Files:**
- Create: `apps/noor-notes/src/ui/rich_color_palette.rs`
- Modify: `apps/noor-notes/src/ui/mod.rs`
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/resources/design-system.css`
- Test: `apps/noor-notes/tests/rich_editor_ui.rs`
- Test: `apps/noor-notes/tests/accessibility.rs`
- Test: `apps/noor-notes/tests/design_system.rs`
- Test: `apps/noor-notes/tests/dark_palette_css.rs`

**Interfaces:**
- Consumes: Task 1 `ColorRole`, `ColorPreset`, `presets`, and RichBuffer color methods.
- Produces: `RichColorPalette::new(role) -> Self`
- Produces fields: `widget`, `reset`, `preset_buttons`, and `custom`
- Produces: `RichColorPalette::set_enabled(bool)`
- Produces: `EditorToolbar::set_rich_formatting_enabled(bool)`

- [ ] **Step 1: Write failing UI structure and accessibility tests**

Extend `rich_editor_ui.rs` to assert both palette components exist, each has seven preset toggle buttons plus reset and a `ColorDialogButton`, controls are focusable, and labels include “Blue text,” “Yellow highlight,” “Custom text color,” and “Custom highlight color.”

Extend `accessibility.rs` to check the picker/reset labels and insensitive state after `toolbar.set_rich_formatting_enabled(false)`.

Extend CSS tests to require swatch classes for slate, blue, teal, green, amber, red, purple, yellow, mint, peach, pink, and lavender under light and dark roots.

- [ ] **Step 2: Run tests to verify RED**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_editor_ui --test accessibility --test design_system --test dark_palette_css
```

Expected: compilation fails because `RichColorPalette` and rich-formatting sensitivity APIs do not exist.

- [ ] **Step 3: Build the reusable palette component**

Create a vertical component with a heading and wrapping `FlowBox`. Add:

- one reset toggle with reset icon and explicit accessible label;
- seven preset toggle buttons containing circular swatch widgets;
- one `gtk::ColorDialogButton` backed by `gtk::ColorDialog::builder().with_alpha(false)`;
- mutually exclusive selected-state synchronization;
- role-specific tooltips and accessible labels.

Use a maximum of four swatches per row at narrow width and allow the row to widen without overflowing the formatting popover.

- [ ] **Step 4: Replace anonymous toolbar color buttons**

Remove `color_buttons` and the old `foreground_buttons`/`highlight_buttons` fields. Add `foreground_palette` and `highlight_palette` fields and attach their widgets below typography/alignment in the formatting popover.

Implement `set_rich_formatting_enabled` to update every rich-only control: style toggles, list controls, font presets/custom entry, alignment, both palettes, and clear formatting. Disabled tooltips must state “Available in Rich Text mode.”

- [ ] **Step 5: Connect preset, custom, and reset actions**

In `editor_actions.rs`:

- preset activation calls `RichBuffer::foreground` or `RichBuffer::highlight`;
- reset calls the role-specific clear method;
- `connect_rgba_notify` converts the selected opaque RGB to uppercase `#RRGGBB`;
- no selection leaves marks and selected-state unchanged;
- successful actions return focus to the editor.

- [ ] **Step 6: Add visible adaptive swatch CSS**

Define compact circular swatches with borders, focus rings, check indicators, and light/dark mappings. Do not style source-editor text with these rules. Keep destructive, disabled, and high-contrast states intact.

- [ ] **Step 7: Run focused tests to verify GREEN**

Run the Task 2 command again. Expected: all four test binaries pass with valid GTK CSS.

- [ ] **Step 8: Commit Task 2**

```bash
git add apps/noor-notes/src/ui/mod.rs apps/noor-notes/src/ui/rich_color_palette.rs apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/src/editor_actions.rs apps/noor-notes/resources/design-system.css apps/noor-notes/tests/rich_editor_ui.rs apps/noor-notes/tests/accessibility.rs apps/noor-notes/tests/design_system.rs apps/noor-notes/tests/dark_palette_css.rs
git commit -m "feat: add native rich color pickers"
```

### Task 3: Live themes, persistence, documentation, and delivery verification

**Files:**
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/tests/rich_formatting_persistence.rs`
- Modify: `apps/noor-notes/tests/editor_conversion.rs`
- Modify: `README.md`

**Interfaces:**
- Consumes: Task 1 `RichBuffer::apply_color_theme`
- Consumes: Task 2 `EditorToolbar::set_rich_formatting_enabled`

- [ ] **Step 1: Write failing persistence and mode-gating tests**

Extend `rich_formatting_persistence.rs` to save a note containing a custom foreground and custom highlight, close the repository, reopen it, load the rich buffer, snapshot again, and assert exact uppercase RGB values.

Add a note-window contract assertion that source modes call `set_rich_formatting_enabled(false)` through the `rich_mode` value rather than enabling only a partial list of buttons.
Extend `editor_conversion.rs` with custom foreground and highlight values and assert that conversion away from Rich Text reports their loss before applying the conversion.


- [ ] **Step 2: Run focused tests to verify RED**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test rich_formatting_persistence --test rich_editor_ui
```

Expected: persistence or mode-gating assertion fails before integration.

- [ ] **Step 3: Integrate live appearance and mode gating**

Replace the partial sensitivity loop in `NoteWindow` with:

```rust
toolbar.set_rich_formatting_enabled(rich_mode);
```

For Rich Text buffers, call `RichBuffer::apply_color_theme` at construction and subscribe a weak buffer reference to `AppearanceManager`. Closed windows must not be retained. Source buffers continue using `source_palette::apply`.

- [ ] **Step 4: Update README**

Document:

- professional adaptive presets;
- native custom text/highlight pickers;
- reset actions;
- Rich Text-only availability;
- custom color save/reopen persistence;
- source modes remaining controlled by GtkSourceView palettes.

- [ ] **Step 5: Run focused tests to verify GREEN**

Run the Task 3 focused command. Expected: both binaries pass.

- [ ] **Step 6: Run complete verification**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets --all-features -- -D warnings
PATH=/home/mamun/.cargo/bin:$PATH cargo test --workspace
PATH=/home/mamun/.cargo/bin:$PATH cargo build --release
git diff --check
```

Expected: every command exits 0 with no warnings or failures.

- [ ] **Step 7: Install the verified local build**

Confirm Noor Notes is not running, then run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh
sha256sum target/release/noor-notes /home/mamun/.local/bin/noor-notes
```

Expected: installer exits 0 and both hashes match.

- [ ] **Step 8: Perform manual verification**

In Rich Text mode:

1. Select text and apply every text preset.
2. Apply every highlight preset.
3. Choose one custom text RGB and one custom highlight RGB.
4. Undo and redo at least five color changes.
5. Wait for Saved, close, reopen, and confirm exact colors remain.
6. Verify Auto and None reset only their own mark.
7. Switch Light, Graphite, Midnight, and OLED and confirm presets adapt while custom RGB remains exact.
8. Resize to a narrow window and confirm swatches wrap without clipping.
9. Navigate every color control using keyboard only.
10. Switch to Markdown, Plain Text, and Code and confirm color controls are disabled.

- [ ] **Step 9: Commit integration and documentation**

```bash
git add apps/noor-notes/src/note_window.rs apps/noor-notes/tests/rich_formatting_persistence.rs apps/noor-notes/tests/editor_conversion.rs README.md
git commit -m "docs: explain rich text color controls"
```

Do not push unless the user explicitly asks after reviewing the verified result.
