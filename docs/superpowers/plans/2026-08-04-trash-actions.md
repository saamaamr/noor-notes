# Trash Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Restore and confirmed Permanent Delete through Trash rows, row context menus, and trashed-note windows.

**Architecture:** Extend the repository with transactional permanent deletion, centralize trash operations, and reuse them from `MainWindow` and `NoteWindow`. All successful operations refresh lists; failures preserve visible UI state.

**Tech Stack:** Rust, GTK4, Libadwaita, Tokio, SQLx/SQLite.

## Global Constraints

- Restore and Permanent Delete appear only for trashed notes.
- Permanent Delete always requires confirmation.
- Permanent deletion removes only local database data for the selected note.
- Storage failures keep controls and content available.

---

### Task 1: Storage and transitions

**Files:**
- Modify: `crates/storage/src/repository.rs`
- Modify: `apps/noor-notes/src/note_actions.rs`
- Test: `crates/storage/tests/lifecycle.rs`
- Test: `apps/noor-notes/tests/note_actions.rs`

**Interfaces:**
- Produces: `SqliteNoteRepository::delete_permanently(NoteId)` and `note_actions::restore(&mut Note, DateTime<Utc>)`.

- [ ] Add failing tests for content-preserving Restore and isolated permanent deletion.
- [ ] Verify focused tests fail for missing APIs.
- [ ] Implement the minimal transition and transactional delete operation.
- [ ] Verify focused tests pass and commit.

### Task 2: Trash row actions

**Files:**
- Modify: `apps/noor-notes/src/main_window.rs`
- Test: `apps/noor-notes/tests/trash_actions.rs`

**Interfaces:**
- Consumes: repository restore and permanent-delete methods.
- Produces: visible Restore/Delete buttons and a right-click popover for Trash rows.

- [ ] Add a failing UI contract test for both row and context-menu actions.
- [ ] Implement shared row action callbacks, destructive confirmation, refresh, disabled states, and errors.
- [ ] Verify the focused UI contract and storage tests pass; commit.

### Task 3: Trashed-note toolbar

**Files:**
- Modify: `apps/noor-notes/src/modern_toolbar.rs`
- Modify: `apps/noor-notes/src/note_window.rs`
- Modify: `apps/noor-notes/src/managed_app.rs`
- Test: `apps/noor-notes/tests/trash_actions.rs`

**Interfaces:**
- Produces: trash-state-aware toolbar controls and repository access in note windows.

- [ ] Extend the failing contract for trash-only window controls.
- [ ] Pass repository access into note windows and connect Restore/Delete with flush/error/close behavior.
- [ ] Verify focused tests pass; commit.

### Task 4: Delivery

- [ ] Run formatting, strict Clippy, GTK tests, and the full workspace suite.
- [ ] Merge to `main`, rerun the full suite, install locally, and smoke-test the binary.
- [ ] Push `main` and verify local/remote commit hashes match.
