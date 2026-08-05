# Note Editor Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable note titles, complete bullet/numbered-list behavior, arbitrary positive custom font sizes, and exact compact note-window dimensions.

**Architecture:** Extend `Note` with backward-compatible title metadata and migrate existing JSON-backed SQLite rows on repository open. Keep list parsing and transformations inside `RichBuffer`, expose GTK controls through `ModernToolbar`, and connect persistence/UI behavior in `NoteWindow`. Scope compact styling to `.noor-note` so the library window is unchanged.

**Tech Stack:** Rust 2024, GTK4/libadwaita, serde JSON, SQLx/SQLite, Tokio, Xvfb, CSS.

## Global Constraints

- A note title is independent from body content and synchronizes inside the existing encrypted `Note` payload.
- Missing legacy titles derive from the first non-empty body line without modifying the body; otherwise display `Untitled note`.
- Repeated list actions never stack markers, list switching replaces markers, ordered lists increment, and empty Enter exits the list.
- Custom font size accepts any positive whole-number pixel value and rejects all other input.
- Note top bar/clickable height is exactly 28 px, icons exactly 12 px, and note radius exactly 3 px.
- Store permissions and packaging manifests must not change.

---

### Task 1: Durable note-title model and migration

**Files:**
- Modify: `crates/domain/src/note.rs`
- Modify: `crates/domain/tests/note_model.rs`
- Create: `crates/storage/migrations/0002_note_titles.sql`
- Modify: `crates/storage/src/repository.rs`
- Modify: `crates/storage/tests/repository.rs`
- Modify: `crates/storage/tests/lifecycle.rs`
- Modify: `crates/xpad-import/src/parser.rs`
- Modify: `crates/xpad-import/tests/import.rs`
- Modify: `crates/sync/tests/conflicts.rs`
- Modify: `crates/sync/tests/remote_download.rs`

**Interfaces:**
- Produces: `Note.title: String`, `Note::display_title(&self) -> &str`, and `Note::derive_title(content: &str) -> String`.
- Consumes: existing whole-note serde payloads used by storage, the change journal, encryption, and sync.

- [ ] **Step 1: Write failing domain compatibility tests**

Add tests proving `Note::new` starts with `Untitled note`, legacy JSON without `title` deserializes, title derivation uses the first trimmed non-empty line, and a whitespace-only body derives `Untitled note`.

- [ ] **Step 2: Run domain tests and verify RED**

```bash
cargo test -p noor-domain --test note_model
```

Expected: FAIL because `Note.title`, `display_title`, and `derive_title` do not exist.

- [ ] **Step 3: Implement the backward-compatible domain field**

Add `pub title: String`, set it in `Note::new`, normalize blank display values, and derive a title from at most the first 80 Unicode characters of the first non-empty trimmed body line. Implement backward-compatible deserialization through a private helper representation so a missing or blank legacy title is derived from that payload's content during deserialization rather than defaulting before content is available.

- [ ] **Step 4: Write failing storage migration and round-trip tests**

Create a legacy database/payload without a title, reopen it, and assert the title column and payload are migrated without changing `content`. Add persistence, archive/restore, permanent-delete isolation, and pending-change assertions for titles.

- [ ] **Step 5: Run storage tests and verify RED**

```bash
cargo test -p noor-storage --test repository --test lifecycle
```

Expected: FAIL because migration `0002_note_titles.sql` and repository normalization are missing.

- [ ] **Step 6: Implement idempotent SQLite migration**

Add a non-null `title` column with default `Untitled note`; execute migrations in order during `open`. In one transaction, normalize legacy payloads missing/blank titles, update both `notes.title` and `payload_json`, and preserve content/revision/timestamps. Bind `title` on every save/update.

- [ ] **Step 7: Extend import and sync regression coverage**

Assert Xpad imports derive titles, whole-note encryption round trips titles, remote application preserves titles, and conflict-copy naming does not discard them. No Supabase schema change is needed because the encrypted payload already contains the serialized `Note`.

- [ ] **Step 8: Run Task 1 verification**

```bash
cargo test -p noor-domain -p noor-storage -p noor-xpad-import -p noor-sync
git diff --check
```

Expected: PASS with no content mutation and no whitespace errors.

- [ ] **Step 9: Commit**

```bash
git add crates/domain crates/storage crates/xpad-import crates/sync
git commit -m "feat: add durable note titles"
```

### Task 2: Idempotent list formatting engine

**Files:**
- Modify: `apps/noor-notes/src/rich_buffer.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/tests/rich_editor.rs`
- Create: `apps/noor-notes/tests/list_editing.rs`

**Interfaces:**
- Produces: `ListKind::{Bullet, Numbered}`, `RichBuffer::toggle_list`, `RichBuffer::continue_list`, and `RichBuffer::list_kind_at_cursor`.
- Consumes: GTK `TextBuffer` selection/current-line APIs and the existing bullet/number toolbar toggles.

- [ ] **Step 1: Write failing list transformation tests**

Cover current-line application, repeated-click removal, bullet-to-number conversion, number-to-bullet conversion, multi-line application once per line, and ordered numbering from `1` through every selected line.

- [ ] **Step 2: Run list tests and verify RED**

```bash
xvfb-run -a cargo test -p noor-notes --test list_editing
```

Expected: FAIL because list-aware APIs are absent and current code stacks prefixes.

- [ ] **Step 3: Implement marker parsing and target-line transforms**

Recognize only leading `• ` and ASCII ordered markers matching positive digits plus `. `. Compute complete selected line bounds, remove one existing marker, then add the requested kind only when it differs. Renumber all targeted ordered lines sequentially. Preserve body text, selection, and cursor logically after edits.

- [ ] **Step 4: Write failing Enter behavior tests**

Cover continuing a bullet, incrementing `9. item` to `10. `, and turning an empty `• ` or `3. ` item into a plain empty line.

- [ ] **Step 5: Implement Enter interception**

Connect `TextView::connect_key_pressed` through an event controller. On unmodified Return/KP_Enter, call `continue_list`; stop propagation only when list handling performed an edit. Leave Shift/Ctrl/Alt combinations and non-list lines unchanged.

- [ ] **Step 6: Synchronize toolbar state safely**

Update Bullet and Numbered toggle states when cursor/selection changes, blocking recursive action callbacks while reflecting `list_kind_at_cursor`.

- [ ] **Step 7: Run Task 2 verification**

```bash
xvfb-run -a cargo test -p noor-notes --test list_editing --test rich_editor --test toolbar_actions
git diff --check
```

Expected: PASS; repeated actions never duplicate markers.

- [ ] **Step 8: Commit**

```bash
git add apps/noor-notes/src/rich_buffer.rs apps/noor-notes/src/editor_actions.rs apps/noor-notes/tests
git commit -m "feat: complete rich-text list editing"
```

### Task 3: Title field and Rename action

**Files:**
- Modify: `apps/noor-notes/src/modern_toolbar.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/main_window.rs`
- Modify: `apps/noor-notes/src/autosave.rs`
- Create: `apps/noor-notes/tests/note_titles.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`

**Interfaces:**
- Consumes: `Note.title`, `Note::display_title`, and existing `AutosaveQueue` from Task 1.
- Produces: visible `gtk::Entry` title editor and toolbar/menu Rename action sharing one title-update path.

- [ ] **Step 1: Write failing UI contract and autosave tests**

Assert the note window creates a single-line title entry, title changes schedule `NoteDraft`, blank input displays `Untitled note`, the library row uses `display_title`, and Rename updates the same entry/model.

- [ ] **Step 2: Run tests and verify RED**

```bash
xvfb-run -a cargo test -p noor-notes --test note_titles --test autosave --test toolbar_actions
```

Expected: FAIL because title widgets/actions are absent.

- [ ] **Step 3: Add title editing and autosave**

Place the title entry between compact header and body, bind initial stored title, disable it for trashed notes, update the window's accessible title, and schedule the shared note through the existing debounce path on change.

- [ ] **Step 4: Add Rename action**

Expose Rename in the settings/menu controls. Present an `adw::AlertDialog` with an entry prefilled from the title; confirm updates the visible title entry so the same change handler performs normalization and autosave. Cancel makes no change.

- [ ] **Step 5: Replace body-derived library naming**

Remove `main_window::note_title` body parsing and use `Note::display_title` for active, archived, and trashed rows.

- [ ] **Step 6: Run Task 3 verification**

```bash
xvfb-run -a cargo test -p noor-notes --test note_titles --test autosave --test toolbar_actions --test trash_actions
cargo test -p noor-notes --test search --test import_flow
git diff --check
```

Expected: PASS with title persistence driven through one handler.

- [ ] **Step 7: Commit**

```bash
git add apps/noor-notes/src apps/noor-notes/tests
git commit -m "feat: add note naming and rename controls"
```

### Task 4: Arbitrary positive custom font sizes

**Files:**
- Modify: `apps/noor-notes/src/modern_toolbar.rs`
- Modify: `apps/noor-notes/src/editor_actions.rs`
- Modify: `apps/noor-notes/src/rich_buffer.rs`
- Create: `apps/noor-notes/tests/font_sizes.rs`
- Modify: `apps/noor-notes/tests/rich_editor.rs`
- Modify: `crates/domain/src/rich_text.rs`
- Modify: `crates/domain/tests/rich_text.rs`

**Interfaces:**
- Produces: preset controls plus custom positive whole-number entry; `RichBuffer::font_size` dynamically creates `noor-size-N` tags.
- Consumes: `TextMarks.font_size`, widened compatibly from `Option<u16>` to `Option<u32>`. The UI imposes no separate maximum; any positive whole number representable by the persisted type is accepted.

- [ ] **Step 1: Write failing validation and round-trip tests**

Cover presets, custom `1`, `37`, `65535`, and a value above the old `u16` ceiling, tag replacement, serialization round trip, and rejection of `0`, negatives, decimals, blank, non-numeric, and values beyond the persisted integer representation.

- [ ] **Step 2: Run tests and verify RED**

```bash
xvfb-run -a cargo test -p noor-notes --test font_sizes --test rich_editor
```

Expected: FAIL because the toolbar has presets only and tags are pre-created for five sizes.

- [ ] **Step 3: Implement dynamic size tags**

Create or reuse a `noor-size-{N}` tag on demand, remove every applied tag whose name begins `noor-size-`, and apply the dynamic tag to the selection. Ensure `load` also creates missing tags before applying serialized custom sizes.

- [ ] **Step 4: Add custom entry beside presets**

Keep one-click preset selection and add a numeric entry with an Apply affordance. Parse strictly as `u32`, require `> 0`, impose no smaller application-defined cap, show invalid state/tooltips without changing the buffer, and clear invalid state after valid application.

- [ ] **Step 5: Run Task 4 verification**

```bash
xvfb-run -a cargo test -p noor-notes --test font_sizes --test rich_editor
cargo test -p noor-domain --test rich_text
git diff --check
```

Expected: PASS for arbitrary representable positive whole-number values, including values above 65535.

- [ ] **Step 6: Commit**

```bash
git add apps/noor-notes/src apps/noor-notes/tests
git commit -m "feat: add custom font sizes"
```

### Task 5: Compact note-window styling and full verification

**Files:**
- Modify: `apps/noor-notes/resources/modern.css`
- Modify: `apps/noor-notes/resources/style.css`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/modern_toolbar.rs`
- Create: `apps/noor-notes/tests/compact_ui.rs`

**Interfaces:**
- Consumes: `.noor-note`, `.modern-toolbar`, and `.toolbar-button` CSS classes.
- Produces: note-only 28 px chrome/click targets, 12 px icons, and 3 px radius.

- [ ] **Step 1: Write failing exact-dimension tests**

Assert note-scoped CSS and widget code specify 28 px top bar/button height, 12 px icon size, and 3 px outer note/header radius, while the main library window selectors remain unaffected.

- [ ] **Step 2: Run tests and verify RED**

```bash
cargo test -p noor-notes --test compact_ui
```

Expected: FAIL against current 18 px header radius, 36–44 px controls, and default symbolic icon sizing.

- [ ] **Step 3: Apply compact scoped styling**

Set exact dimensions under `.noor-note`; use explicit 12 px icon widgets or note-scoped CSS, preserve focus/checked/destructive/hover states, reduce adjacent padding, and avoid modifying library-window controls.

- [ ] **Step 4: Run complete verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
xvfb-run -a cargo test -p noor-notes --test rich_editor --test list_editing --test note_titles --test font_sizes
xvfb-run -a cargo test -p noor-windowing
gjs -m extensions/gnome/tests/test-policy.js
bash tests/e2e/two_device_sync.sh
bash tests/snap_manifest.sh
bash tests/flatpak_manifest.sh
git diff --check
```

Expected: every command exits 0; no package-permission changes appear in the diff.

- [ ] **Step 5: Commit**

```bash
git add apps/noor-notes/resources apps/noor-notes/src apps/noor-notes/tests
git commit -m "style: compact note editor chrome"
```
