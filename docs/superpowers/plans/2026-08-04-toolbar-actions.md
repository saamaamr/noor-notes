# Toolbar Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make New Note, Archive, and Delete fully functional, immediately persistent, and safely confirmed where destructive.

**Architecture:** Pure transition functions in `note_actions.rs` own note-state mutation. `NoteWindow` owns button wiring, the delete confirmation dialog, immediate autosave flush, error presentation, and window closure.

**Tech Stack:** Rust, GTK4, Libadwaita, Tokio, existing `AutosaveQueue` and domain model.

## Global Constraints

- Delete requires explicit confirmation; Cancel changes nothing.
- Archive and confirmed Delete persist before closing the note window.
- New Note reuses `app.new-note`.
- Existing toolbar behavior must remain unchanged.

---

### Task 1: State transitions

**Files:**
- Create: `apps/noor-notes/src/note_actions.rs`
- Modify: `apps/noor-notes/src/lib.rs`
- Test: `apps/noor-notes/tests/note_actions.rs`

**Interfaces:**
- Produces: `archive(&mut Note, DateTime<Utc>)` and `trash(&mut Note, DateTime<Utc>)`.

- [ ] Write tests proving archive/trash states, timestamps, revision updates, and content preservation.
- [ ] Run `cargo test -p noor-notes --test note_actions` and verify the missing-module failure.
- [ ] Implement the two pure transition functions using the domain state types.
- [ ] Rerun the focused test and commit.

### Task 2: Toolbar wiring

**Files:**
- Modify: `apps/noor-notes/src/note_window.rs`
- Test: `apps/noor-notes/tests/toolbar_actions.rs`

**Interfaces:**
- Consumes: `note_actions::{archive, trash}`, `AutosaveQueue::{schedule, flush}`, and `app.new-note`.
- Produces: connected New Note, Archive, and confirmation-gated Delete controls.

- [ ] Write a source contract test that fails until all three toolbar controls are wired.
- [ ] Run the focused test and verify the expected failure.
- [ ] Connect New Note to `app.new-note`; connect Archive to transition/save/flush/close; connect Delete to an Adwaita confirmation dialog and the same persistence path.
- [ ] Keep the window open and present an error dialog if flush fails.
- [ ] Rerun focused UI/action and autosave tests and commit.

### Task 3: Delivery

**Files:**
- Modify only if verification reveals a defect.

- [ ] Run formatting, strict Clippy, GTK tests, and the complete workspace suite.
- [ ] Install with `scripts/install-local.sh` and smoke-test the installed binary.
- [ ] Push `main` and verify local and remote hashes match.
