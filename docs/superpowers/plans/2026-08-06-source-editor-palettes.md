# Theme-Matched Source Editor Palettes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace GtkSourceView's conflicting `classic` colors with Noor Notes Light, Graphite, Midnight, and OLED source-editor palettes that update live without changing note content.

**Architecture:** Compile four GtkSourceView XML schemes into the application GResource, register them once, and resolve them through a focused `source_palette` module. Source buffers receive the effective Noor Notes theme at creation and through weak-reference appearance subscriptions; Rich Text keeps its existing CSS path.

**Tech Stack:** Rust 1.87, GTK4, libadwaita, GtkSourceView 5, GLib GResource, Cargo integration tests

## Global Constraints

- Preserve notes, editor modes, source languages, cursor, selection, undo/redo history, and database compatibility.
- Markdown and Code retain syntax highlighting; Plain Text remains uniform.
- Use no remote assets, analytics, telemetry, or runtime network access.
- Add only the standard `glib-build-tools` build-time helper.
- Do not modify Snap metadata or perform Snap build, upload, release, or Store actions.
- Do not touch the existing untracked Snap artifacts.

---

### Task 1: Embed and register four source-editor schemes

**Files:**
- Create: `apps/noor-notes/build.rs`
- Create: `apps/noor-notes/resources/styles/noor-light.xml`
- Create: `apps/noor-notes/resources/styles/noor-graphite.xml`
- Create: `apps/noor-notes/resources/styles/noor-midnight.xml`
- Create: `apps/noor-notes/resources/styles/noor-oled.xml`
- Create: `apps/noor-notes/src/editor/source_palette.rs`
- Create: `apps/noor-notes/tests/source_palettes.rs`
- Modify: `apps/noor-notes/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `apps/noor-notes/resources/noor-notes.gresource.xml`
- Modify: `apps/noor-notes/src/editor/mod.rs`

**Interfaces:**
- Produces: `source_palette::register() -> bool`
- Produces: `source_palette::scheme_id(EffectiveTheme) -> &'static str`
- Produces: `source_palette::apply(&sourceview5::Buffer, EffectiveTheme) -> Option<glib::GString>`
- Produces scheme IDs: `noor-light`, `noor-graphite`, `noor-midnight`, `noor-oled`

- [ ] **Step 1: Write the failing palette discovery and mapping test**

Create `source_palettes.rs` with a single GTK-initialized test that calls `register()`, asserts the four mapping results, asks the default `StyleSchemeManager` for all four scheme IDs, and verifies each scheme provides `text`, `cursor`, `line-numbers`, `current-line`, `selection`, and `search-match` styles.

The test must also parse the `text` foreground/background colors and calculate WCAG relative luminance:

```rust
fn contrast_ratio(foreground: &str, background: &str) -> f64 {
    let luminance = |hex: &str| {
        let channel = |offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap() as f64 / 255.0;
        let linear = |value: f64| if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        };
        0.2126 * linear(channel(1)) + 0.7152 * linear(channel(3)) + 0.0722 * linear(channel(5))
    };
    let (a, b) = (luminance(foreground), luminance(background));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
```

Require `contrast_ratio >= 4.5` for every scheme.

- [ ] **Step 2: Run the test and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test source_palettes
```

Expected: compilation fails because `source_palette` does not exist.

- [ ] **Step 3: Add resource compilation**

Add to `apps/noor-notes/Cargo.toml`:

```toml
[build-dependencies]
glib-build-tools = "0.21"
```

Create `build.rs`:

```rust
fn main() {
    glib_build_tools::compile_resources(
        &[`resources`],
        `resources/noor-notes.gresource.xml`,
        `noor-notes.gresource`,
    );
}
```

Add the four style XML files to the existing GResource under `styles/`.

- [ ] **Step 4: Define the four XML palettes**

Each file uses GtkSourceView style-scheme version 1.0, inherits `Adwaita` for Light or `Adwaita-dark` for dark palettes, and overrides these exact style IDs:

```xml
<style name=`text` foreground=`BODY` background=`CANVAS`/>
<style name=`cursor` foreground=`ACCENT`/>
<style name=`line-numbers` foreground=`SECONDARY` background=`GUTTER`/>
<style name=`current-line-number` foreground=`ACCENT` background=`CURRENT` bold=`true`/>
<style name=`current-line` background=`CURRENT`/>
<style name=`selection` foreground=`SELECTION_TEXT` background=`SELECTION`/>
<style name=`search-match` foreground=`SEARCH_TEXT` background=`SEARCH`/>
<style name=`bracket-match` foreground=`ACCENT` bold=`true`/>
<style name=`def:comment` foreground=`SECONDARY` italic=`true`/>
<style name=`def:keyword` foreground=`ACCENT` bold=`true`/>
<style name=`def:string` foreground=`STRING`/>
<style name=`def:number` foreground=`CONSTANT`/>
<style name=`def:constant` foreground=`CONSTANT`/>
<style name=`def:type` foreground=`TYPE`/>
<style name=`def:function` foreground=`FUNCTION`/>
<style name=`def:error` foreground=`ERROR` underline=`error`/>
```

Use the base colors from the approved specification and accessible palette-specific secondary, string, constant, type, function, selection, and search colors. Include `<property name=`variant`>light</property>` for Light and `dark` for the other three.

- [ ] **Step 5: Implement registration and application**

In `source_palette.rs`, use a `OnceLock<bool>` to register `include_bytes!(concat!(env!(`OUT_DIR`), `/noor-notes.gresource`))` through `gio::Resource::from_data` and `gio::resources_register` exactly once. Prepend `resource:///io/github/saamaamr/NoorNotes/styles` to the default style manager and force a rescan.

`apply` selects the Noor ID first, then `Adwaita` for Light or `Adwaita-dark` for dark themes. It calls `buffer.set_style_scheme(Some(&scheme))` and returns the applied ID. If neither lookup succeeds, return `None` without changing text.

- [ ] **Step 6: Run the focused test and verify GREEN**

Run the Step 2 command. Expected: all four schemes are discoverable and every base contrast assertion passes.

- [ ] **Step 7: Commit embedded palettes**

```bash
git add Cargo.lock apps/noor-notes/Cargo.toml apps/noor-notes/build.rs apps/noor-notes/resources/noor-notes.gresource.xml apps/noor-notes/resources/styles apps/noor-notes/src/editor/mod.rs apps/noor-notes/src/editor/source_palette.rs apps/noor-notes/tests/source_palettes.rs
git commit -m `feat: add theme-matched source palettes`
```

---

### Task 2: Apply palettes to every source mode and live theme changes

**Files:**
- Modify: `apps/noor-notes/src/editor/source_adapter.rs`
- Modify: `apps/noor-notes/src/note_window.rs:40-170`
- Modify: `apps/noor-notes/tests/source_editor.rs`
- Modify: `apps/noor-notes/tests/appearance_manager.rs`

**Interfaces:**
- Consumes: `source_palette::apply(&Buffer, EffectiveTheme)`
- Produces: `SourceEditorAdapter::new_with_theme(text, language, theme) -> Self`
- Produces: `SourceEditorAdapter::apply_theme(theme) -> Option<glib::GString>`
- Preserves: `SourceEditorAdapter::new(text, language) -> Self` with Light as its readable default

- [ ] **Step 1: Write failing source-mode preservation tests**

Extend `source_editor.rs` to construct Markdown, Plain Text, and Code adapters. Assert none uses `classic`, Plain Text has no language, Markdown resolves `markdown`, and Code resolves the selected language.

For a Unicode buffer, make five edits, select text, place the cursor, capture text and undo availability, call `apply_theme(EffectiveTheme::Midnight)` then `apply_theme(EffectiveTheme::Oled)`, and assert text, selection, cursor, and undo availability are unchanged while scheme IDs change.

- [ ] **Step 2: Run the test and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test source_editor
```

Expected: compilation fails because `new_with_theme` and `apply_theme` do not exist.

- [ ] **Step 3: Integrate palette application in the adapter**

Implement `new_with_theme` by constructing the existing buffer/view/search objects and applying the passed theme after language assignment. Keep `new` as:

```rust
pub fn new(text: &str, language: &SourceLanguage) -> Self {
    Self::new_with_theme(text, language, EffectiveTheme::Light)
}
```

Implement `apply_theme` by delegating to `source_palette::apply(&self.buffer, theme)`. Do not recreate the buffer or view.

- [ ] **Step 4: Wire the current and live theme in NoteWindow**

Capture `let effective_theme = appearance.effective_theme();` before editor creation and use `SourceEditorAdapter::new_with_theme` for non-Rich notes.

After extracting the source buffer, create a GLib weak reference:

```rust
let weak_buffer = source_buffer.downgrade();
appearance.subscribe(move |_, theme| {
    if let Some(buffer) = weak_buffer.upgrade() {
        source_palette::apply(&buffer, theme);
    }
});
```

Register this subscription only for Markdown, Plain Text, and Code. Do not capture the window or a strong buffer reference.

- [ ] **Step 5: Verify focused adapter and appearance tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test source_editor --test appearance_manager --test editor_history --test note_find
```

Expected: all pass, including Unicode state preservation and live scheme changes.

- [ ] **Step 6: Commit source-mode integration**

```bash
git add apps/noor-notes/src/editor/source_adapter.rs apps/noor-notes/src/note_window.rs apps/noor-notes/tests/source_editor.rs apps/noor-notes/tests/appearance_manager.rs
git commit -m `fix: synchronize source colors with appearance`
```

---

### Task 3: Separate Rich Text CSS and complete verification

**Files:**
- Modify: `apps/noor-notes/resources/design-system.css:60-220`
- Modify: `apps/noor-notes/src/note_window.rs:145-160`
- Modify: `apps/noor-notes/tests/dark_palette_css.rs`
- Modify: `apps/noor-notes/tests/design_system.rs`
- Modify: `README.md`

**Interfaces:**
- Produces CSS class: `.nn-rich-writing-canvas`
- Preserves CSS class: `.nn-writing-canvas` for shared padding, typography, caret, and zoom
- Consumes the GtkSource scheme as authoritative for source-editor inner colors

- [ ] **Step 1: Write the failing CSS separation test**

Require `.nn-rich-writing-canvas` light and dark surface rules. Assert `.nn-writing-canvas` contains layout and typography but does not set `background` or `color`. Assert each dark palette targets `.nn-rich-writing-canvas` rather than the shared source class.

- [ ] **Step 2: Run the CSS tests and verify RED**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test design_system --test dark_palette_css
```

Expected: FAIL because canvas color rules are still shared.

- [ ] **Step 3: Separate the CSS classes**

Keep padding, font size, and caret defaults in `.nn-writing-canvas`. Move light background/foreground and all Graphite, Midnight, and OLED canvas colors to `.nn-rich-writing-canvas`. Add that class only when `current.editor_mode == EditorMode::Rich`.

Update README with the four theme-matched source palettes and live theme switching.

- [ ] **Step 4: Run focused visual-contract tests**

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test source_palettes --test source_editor --test design_system --test dark_palette_css --test rich_editor
```

Expected: all pass.

- [ ] **Step 5: Run the complete verification gate**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets --all-features -- -D warnings
PATH=/home/mamun/.cargo/bin:$PATH cargo test --workspace
PATH=/home/mamun/.cargo/bin:$PATH cargo build --release
git diff --check
```

Expected: every command exits 0.

- [ ] **Step 6: Install the verified build**

```bash
PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh
```

If Noor Notes is already running, do not terminate it automatically; report that all windows must be closed and reopened to load the new binary. Perform no Snap action.

- [ ] **Step 7: Commit CSS and documentation**

```bash
git add README.md apps/noor-notes/resources/design-system.css apps/noor-notes/src/note_window.rs apps/noor-notes/tests/dark_palette_css.rs apps/noor-notes/tests/design_system.rs
git commit -m `style: unify source editor appearance`
```
