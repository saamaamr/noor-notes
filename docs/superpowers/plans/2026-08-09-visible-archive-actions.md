# Visible Archive Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add discoverable, accessible Archive buttons to active-note editor headers and selected library cards while reusing Noor Notes’ existing reversible archive persistence.

**Architecture:** Extend the existing editor toolbar with a dedicated header control and connect it together with the More-menu control through one archive handler. Extend note cards with an active-note-only quick action whose visibility follows the owning `GtkListItem:selected` property, then dispatch `CardAction::Archive` through the existing transactional repository lifecycle API.

**Tech Stack:** Rust 1.87, GTK4, libadwaita, Tokio, SQLCipher-backed `SqliteNoteRepository`, Cargo integration tests.

## Global Constraints

- Preserve the current database and all existing notes.
- Add no database migration or dependency.
- Do not change the application ID, packaging, Snap metadata, or release state.
- Archive is reversible and must not show a confirmation dialog.
- The visible action uses `folder-symbolic`, tooltip and accessible label “Archive note.”
- Active notes expose Archive; archived and trashed notes do not.
- Failures leave the note active and visible and use existing error feedback.
- Keep the existing More-menu Archive action as a fallback.

---

### Task 1: Add the editor-header Archive action

**Files:**
- Modify: `apps/noor-notes/src/ui/editor_toolbar.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/tests/accessibility.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`

**Interfaces:**
- Consumes: `note_actions::archive(&mut Note, DateTime<Utc>)`, `AutosaveQueue::schedule`, and `AutosaveQueue::flush`.
- Produces: `EditorToolbar::header_archive: gtk::Button` and one shared editor archive connection path.

- [ ] **Step 1: Write failing editor-header tests**

Add these assertions to the existing GTK accessibility test:

```rust
assert_eq!(
    toolbar.header_archive.tooltip_text().as_deref(),
    Some("Archive note")
);
assert!(toolbar.header_archive.can_focus());
```

Extend `every_primary_toolbar_button_is_wired` with:

```rust
assert!(
    NOTE_WINDOW.contains("[&toolbar.header_archive, &toolbar.archive]")
        && NOTE_WINDOW.contains("connect_archive_button"),
    "header and More-menu Archive controls must share one persistence path"
);
```

- [ ] **Step 2: Run tests and verify the missing field fails compilation**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test accessibility --test toolbar_actions
```

Expected: FAIL because `EditorToolbar` has no `header_archive` field.

- [ ] **Step 3: Add the minimal editor-header control**

In `EditorToolbar`, add:

```rust
pub header_archive: gtk::Button,
```

Construct it beside the existing archive button:

```rust
let archive = icon_button("folder-symbolic", "Archive note");
let header_archive = icon_button("folder-symbolic", "Archive note");
```

Return both fields from `EditorToolbar::new`.

- [ ] **Step 4: Show the control only for active notes**

In `NoteWindow::new`, calculate:

```rust
let is_active = matches!(current.state, NoteState::Active);
let is_trashed = matches!(current.state, NoteState::Trashed { .. });
toolbar.archive.set_visible(is_active);
toolbar.header_archive.set_visible(is_active);
```

Pack `header_archive` beside `header_trash` and add it to the `EditorPresentation` chrome widgets so View-Only mode hides it.

- [ ] **Step 5: Share one archive handler between both buttons**

Extract the existing closure into:

```rust
fn connect_archive_button(
    button: &gtk::Button,
    note: Rc<RefCell<Note>>,
    autosave: AutosaveQueue,
    window: adw::ApplicationWindow,
)
```

Connect both controls:

```rust
for button in [&toolbar.header_archive, &toolbar.archive] {
    connect_archive_button(button, note.clone(), autosave.clone(), window.clone());
}
```

The helper must keep the existing rollback, immediate flush, library refresh, close-on-success, and save-error behavior unchanged.

- [ ] **Step 6: Verify editor tests pass**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test accessibility --test toolbar_actions --test note_actions
```

Expected: PASS.

- [ ] **Step 7: Commit the editor action**

```bash
git add apps/noor-notes/src/ui/editor_toolbar.rs apps/noor-notes/src/note_window.rs apps/noor-notes/tests/accessibility.rs apps/noor-notes/tests/toolbar_actions.rs
git commit -m "feat: expose Archive in note headers"
```

---

### Task 2: Add the selected-card Archive quick action

**Files:**
- Modify: `apps/noor-notes/src/ui/note_card.rs`
- Modify: `apps/noor-notes/src/ui/note_collection.rs`
- Create: `apps/noor-notes/tests/note_card_archive.rs`

**Interfaces:**
- Consumes: `CardAction::Archive` and the existing card action callback `Rc<dyn Fn(NoteId, CardAction)>`.
- Produces: `NoteCard { widget: gtk::Box, archive: Option<gtk::Button> }`; `NoteCollection` binds `GtkListItem:selected` to the quick action’s `visible` property.

- [ ] **Step 1: Write a failing active-card test**

Create `apps/noor-notes/tests/note_card_archive.rs`:

```rust
use std::cell::RefCell;
use std::rc::Rc;

use chrono::Utc;
use gtk::prelude::*;
use noor_domain::{Note, NoteState};
use noor_notes::ui::note_card::{self, CardAction};

#[test]
fn archive_quick_action_exists_only_for_active_notes() {
    gtk::init().unwrap();
    let observed = Rc::new(RefCell::new(None));
    let sink = observed.clone();
    let active = Note::new(Utc::now());
    let card = note_card::build(
        &active,
        Rc::new(move |_, action| *sink.borrow_mut() = Some(action)),
    );
    let archive = card.archive.expect("active note Archive action");
    assert_eq!(archive.tooltip_text().as_deref(), Some("Archive note"));
    archive.emit_clicked();
    assert_eq!(*observed.borrow(), Some(CardAction::Archive));

    let mut archived = Note::new(Utc::now());
    archived.state = NoteState::Archived;
    assert!(note_card::build(&archived, Rc::new(|_, _| {}))
        .archive
        .is_none());

    let mut trashed = Note::new(Utc::now());
    trashed.state = NoteState::Trashed {
        deleted_at: Utc::now(),
    };
    assert!(note_card::build(&trashed, Rc::new(|_, _| {}))
        .archive
        .is_none());
}
```

- [ ] **Step 2: Run the test and verify the missing API fails compilation**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test note_card_archive
```

Expected: FAIL because `CardAction::Archive` and `NoteCard::archive` do not exist.

- [ ] **Step 3: Add the card component and Archive action**

In `note_card.rs`, add:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardAction {
    Archive,
    Trash,
    Restore,
    DeletePermanently,
}

pub struct NoteCard {
    pub widget: gtk::Box,
    pub archive: Option<gtk::Button>,
}
```

For `NoteState::Active`, construct a compact `folder-symbolic` button with tooltip “Archive note,” initially hidden, and connect it to `CardAction::Archive`. Add Archive to the active card’s popover before Move to Trash. Return `archive: None` for Archived and Trashed notes.

- [ ] **Step 4: Bind quick-action visibility to selection**

Update `NoteCollection` binding:

```rust
let card = note_card::build(note, action.clone());
if let Some(archive) = card.archive.as_ref() {
    item.bind_property("selected", archive, "visible")
        .sync_create()
        .build();
}
item.set_child(Some(&card.widget));
```

This keeps the action outside keyboard focus while hidden and reveals it immediately when the card becomes selected.

- [ ] **Step 5: Verify the card test passes**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test note_card_archive --test accessibility
```

Expected: PASS.

- [ ] **Step 6: Commit the card interaction**

```bash
git add apps/noor-notes/src/ui/note_card.rs apps/noor-notes/src/ui/note_collection.rs apps/noor-notes/tests/note_card_archive.rs
git commit -m "feat: show Archive on selected note cards"
```

---

### Task 3: Persist library Archive actions and verify end to end

**Files:**
- Modify: `apps/noor-notes/src/ui/library_window.rs`
- Modify: `apps/noor-notes/tests/toolbar_actions.rs`

**Interfaces:**
- Consumes: `CardAction::Archive`, `SqliteNoteRepository::archive(NoteId, DateTime<Utc>)`, and `MainWindow::refresh`.
- Produces: library Archive dispatch with existing status-error feedback.

- [ ] **Step 1: Write a failing dispatch contract**

Add to `toolbar_actions.rs`:

```rust
const LIBRARY_WINDOW: &str = include_str!("../src/ui/library_window.rs");

#[test]
fn library_archive_action_uses_transactional_repository_lifecycle() {
    assert!(LIBRARY_WINDOW.contains("CardAction::Archive"));
    assert!(LIBRARY_WINDOW.contains("repository.archive(id, Utc::now()).await"));
}
```

- [ ] **Step 2: Run the test and verify the Archive match arm is missing**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-notes --test toolbar_actions
```

Expected: FAIL because `library_window.rs` does not dispatch `CardAction::Archive`.

- [ ] **Step 3: Implement the library dispatch**

Extend `MainWindow::handle_card_action`:

```rust
let result = match action {
    CardAction::Archive => this.repository.archive(id, Utc::now()).await,
    CardAction::Trash => trash_command::trash_saved_note(&this.repository, id).await,
    CardAction::Restore => this.repository.restore(id, Utc::now()).await,
    CardAction::DeletePermanently => this.repository.delete_permanently(id).await,
};
```

Leave the existing confirmation gates limited to Trash and Delete Permanently. Keep the existing success refresh and failure status message.

- [ ] **Step 4: Run focused lifecycle and UI tests**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH xvfb-run -a cargo test -p noor-notes --test toolbar_actions --test note_card_archive --test accessibility
PATH=/home/mamun/.cargo/bin:$PATH cargo test -p noor-storage --test lifecycle
```

Expected: PASS.

- [ ] **Step 5: Manually verify the real GTK flows**

Launch the application against isolated temporary XDG data/config/cache roots. Create two active notes, select one card, confirm only its Archive quick action is visible, activate it, and confirm the Archived sidebar count increases. Open the second note, activate the editor-header Archive button, and confirm the editor closes and the note appears in Archived. Open an Archived note and confirm neither visible Archive button is offered.

- [ ] **Step 6: Run full verification**

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets -- -D warnings
PATH=/home/mamun/.cargo/bin:$PATH cargo test --workspace
PATH=/home/mamun/.cargo/bin:$PATH cargo build --release
git diff --check
git status --short
```

Expected: all commands pass and only the planned source, test, spec, and plan changes are tracked.

- [ ] **Step 7: Commit the library dispatch**

```bash
git add apps/noor-notes/src/ui/library_window.rs apps/noor-notes/tests/toolbar_actions.rs
git commit -m "feat: archive notes from the library"
```
