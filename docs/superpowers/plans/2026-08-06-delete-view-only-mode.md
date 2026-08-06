# Delete and View-Only Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (- [ ]) syntax for tracking.

**Goal:** Add three discoverable Move to Trash entry points and a persistent per-note minimal View-Only Mode.

**Architecture:** Persist View-Only Mode in the existing serialized EditorPreferences with a backward-compatible default. A focused EditorPresentation controller applies visibility and editability, while existing note_actions and autosave transactions remain responsible for safe Trash transitions.

**Tech Stack:** Rust 2024, GTK4, libadwaita, serde, SQLCipher-backed existing repository, Cargo tests.

## Global Constraints

- Preserve all existing notes and the current database schema.
- Do not change the application ID, package identity, Snap metadata, or Store revisions.
- Do not add dependencies, analytics, telemetry, or network behavior.
- Permanent deletion remains restricted to Trash and requires confirmation.
- Entering View-Only Mode and moving to Trash must not discard pending edits.
- Existing untracked Snap artifacts must remain untouched.

---

### Task 1: Persist the per-note View-Only preference

**Files:**
- Modify: crates/domain/src/note.rs
- Modify: crates/domain/tests/note_metadata.rs

**Interfaces:**
- Consumes: EditorPreferences serde representation and Note::duplicate.
- Produces: public EditorPreferences::view_only: bool with a false legacy default.

- [ ] **Step 1: Add failing compatibility and duplication assertions**

Extend the existing metadata tests with:

~~~rust
assert!(!restored.editor_preferences.view_only);
note.editor_preferences.view_only = true;
let encoded = serde_json::to_string(&note).unwrap();
assert!(serde_json::from_str::<Note>(&encoded)
    .unwrap()
    .editor_preferences
    .view_only);
assert!(!note.duplicate(Utc::now()).editor_preferences.view_only);
~~~

- [ ] **Step 2: Verify RED**

Run: PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-domain --test note_metadata

Expected: compilation fails because EditorPreferences has no view_only field.

- [ ] **Step 3: Implement the preference**

Add #[serde(default)] pub view_only: bool, initialize it to false in Default, and explicitly reset the duplicate copy to false after cloning editor preferences.

- [ ] **Step 4: Verify GREEN**

Run: PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-domain --test note_metadata

Expected: all note_metadata tests pass.

- [ ] **Step 5: Commit**

~~~bash
git add crates/domain/src/note.rs crates/domain/tests/note_metadata.rs
git commit -m "feat: persist per-note view-only preference"
~~~

---

### Task 2: Build a focused editor-presentation controller

**Files:**
- Create: apps/noor-notes/src/ui/editor_presentation.rs
- Modify: apps/noor-notes/src/ui/mod.rs
- Modify: apps/noor-notes/src/ui/editor_toolbar.rs
- Create: apps/noor-notes/tests/editor_presentation.rs

**Interfaces:**
- Consumes: GTK widgets already constructed by NoteWindow.
- Produces: EditorPresentation::new(elements), set_view_only(bool), and is_view_only().

- [ ] **Step 1: Write the failing presentation test**

Construct a header title box, header action group, toolbar, metadata row, find panel, status bar, and editable TextView. Assert set_view_only(true) hides every chrome element, keeps the body visible/selectable, sets editable false, and set_view_only(false) restores the elements.

- [ ] **Step 2: Verify RED**

Run: PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test editor_presentation

Expected: compilation fails because ui::editor_presentation does not exist.

- [ ] **Step 3: Implement EditorPresentation**

Store cloned widget references and a Cell<bool>. set_view_only applies visibility to all chrome groups, calls editor.set_editable(!view_only && !state_forces_read_only), closes the find panel, and focuses the editor when entering reading mode. Keep trashed-state read-only as a separate constructor flag.

- [ ] **Step 4: Add View Only to editor options**

Add pub view_only: gtk::Button to EditorToolbar. Place a labelled View Only button in the View popover with tooltip and accessible label. Do not add it to the permanent primary toolbar.

- [ ] **Step 5: Verify GREEN and accessibility**

Run:

~~~bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test editor_presentation
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test accessibility --test note_commands
~~~

Expected: all named tests pass.

- [ ] **Step 6: Commit**

~~~bash
git add apps/noor-notes/src/ui apps/noor-notes/tests/editor_presentation.rs apps/noor-notes/tests/accessibility.rs apps/noor-notes/tests/note_commands.rs
git commit -m "feat: add minimal view-only presentation"
~~~

---

### Task 3: Integrate persistent View-Only Mode and all Trash entry points

**Files:**
- Modify: apps/noor-notes/src/note_window.rs
- Create: apps/noor-notes/src/services/trash_command.rs
- Modify: apps/noor-notes/src/services/mod.rs
- Modify: apps/noor-notes/src/ui/editor_toolbar.rs
- Modify: apps/noor-notes/src/ui/note_card.rs
- Modify: apps/noor-notes/src/ui/library_window.rs
- Modify: apps/noor-notes/tests/toolbar_actions.rs
- Modify: apps/noor-notes/tests/trash_actions.rs
- Create: apps/noor-notes/tests/view_only_mode.rs

**Interfaces:**
- Consumes: EditorPresentation, EditorPreferences::view_only, AutosaveQueue::schedule/flush, SqliteNoteRepository::trash, and note_actions::trash.
- Produces: header trash button, More-menu trash button, active-card trash action, and persistent View/Edit transitions.

- [ ] **Step 1: Add failing discoverability tests**

Assert EditorToolbar exports distinct header_trash and trash controls, active note cards expose CardAction::Trash, trashed cards expose only Restore/DeletePermanently, and all destructive buttons have Move to Trash tooltips or labels.

- [ ] **Step 2: Verify RED**

Run: PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test trash_actions --test toolbar_actions

Expected: failure because header_trash and CardAction::Trash are absent.

- [ ] **Step 3: Add the three Trash entry points**

Create header_trash with user-trash-symbolic and destructive-hover styling; pack it into the editor header only for non-trashed notes. Keep toolbar.trash in More. Extend CardAction with Trash and show a compact Note actions menu on every card, selecting actions by NoteState.

- [ ] **Step 4: Unify safe Trash behavior**

Create services::trash_command with one confirm(parent: &impl IsA<gtk::Window>) async helper that owns the exact confirmation wording, and two transition functions: trash_open_note(note, autosave) for a pending editor draft and trash_saved_note(repository, id) for a library card. Connect header_trash and toolbar.trash to one Rc callback that invokes this shared service, refreshes the library, and closes only on success. On failure restore the previous Note, leave the window open, re-enable controls, and show the existing save error. MainWindow CardAction::Trash invokes the same confirmation helper and trash_saved_note, then refreshes on success or reports the error in the status bar.

- [ ] **Step 5: Integrate View-Only Mode**

Initialize EditorPresentation from current.editor_preferences.view_only. On View Only activation, flush pending edits, set the preference true, advance updated_at/revision using the existing note mutation pattern, save it, and only then apply the presentation. Escape and a two-press GtkGestureClick on the body set the preference false, save, restore Edit Mode, and focus the editor/title. Save failure keeps the previous presentation and exposes the existing failed state.

- [ ] **Step 6: Verify persistence and interactions**

Run:

~~~bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test view_only_mode --test trash_actions --test toolbar_actions
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-storage --test lifecycle
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test autosave --test save_status
~~~

Expected: all named tests pass.

- [ ] **Step 7: Commit**

~~~bash
git add apps/noor-notes/src/note_window.rs apps/noor-notes/src/ui apps/noor-notes/tests
git commit -m "feat: add discoverable trash and persistent view-only mode"
~~~

---

### Task 4: Documentation, installation, and final verification

**Files:**
- Modify: README.md

**Interfaces:**
- Consumes: completed behavior from Tasks 1-3.
- Produces: user-facing instructions for Trash and View-Only Mode.

- [ ] **Step 1: Update README**

Document all three Move to Trash locations, the confirmation/recovery behavior, View Only in the editor More menu, persistent per-note state, selectable/copyable body, and double-click/Escape exit.

- [ ] **Step 2: Run the complete gate**

~~~bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets --all-features -- -D warnings
PATH=/home/mamun/.cargo/bin:$PATH cargo test --workspace
PATH=/home/mamun/.cargo/bin:$PATH cargo build --release
bash tests/install_ubuntu.sh
bash tests/store_metadata.sh
git diff --check
~~~

Expected: every command exits zero.

- [ ] **Step 3: Install and manually verify**

Run PATH=/home/mamun/.cargo/bin:$PATH bash scripts/install-local.sh. Launch ~/.local/bin/noor-notes and verify active-note header/menu/card Trash, confirmation, Trash restore/permanent delete, View Only chrome, selectable/copyable body, Escape, double-click, restart persistence, Light, Graphite, Midnight, OLED, and narrow-window controls.

- [ ] **Step 4: Commit**

~~~bash
git add README.md
git commit -m "docs: explain trash and view-only mode"
~~~

- [ ] **Step 5: Report without publishing**

Report modified files, commits, verification output, manual results, branch, and Git status. Do not push, upload, release, or perform Snap Store actions without explicit user approval.
