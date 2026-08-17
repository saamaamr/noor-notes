# Rich Text Canvas Spacing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Set the Rich Text writing area to 5-pixel top/bottom margins and 8-pixel left/right margins without changing other editor modes.

**Architecture:** Keep spacing centralized in the existing `configure_editor_canvas` function. Update its Rich Text tuple only, while the focused GTK integration test protects both the requested Rich Text values and the unchanged source-mode values.

**Tech Stack:** Rust 1.85, GTK4/Libadwaita, Cargo integration tests, Xvfb

## Global Constraints

- Rich Text uses 5 pixels at the top and bottom.
- Rich Text uses 8 pixels at the left and right.
- Markdown, Plain Text, and Code margins remain unchanged.
- Preserve the reading-width clamp, CSS classes, accessibility label, formatting, writing assistance, and persistence behavior.
- Add no dependency and perform no unrelated refactor.

---

### Task 1: Reduce Rich Text canvas margins

**Files:**
- Modify: `apps/noor-notes/tests/editor_canvas.rs:5-28`
- Modify: `apps/noor-notes/src/ui/editor_canvas.rs:3-18`

**Interfaces:**
- Consumes: `configure_editor_canvas(editor: &gtk::TextView, rich_mode: bool)`
- Produces: Rich Text `GtkTextView` margins of left/right 8 and top/bottom 5; the function signature remains unchanged.

- [x] **Step 1: Write the failing regression assertions**

Replace the Rich Text margin assertions in `apps/noor-notes/tests/editor_canvas.rs` with:

```rust
assert_eq!(rich_editor.left_margin(), 8);
assert_eq!(rich_editor.right_margin(), 8);
assert_eq!(rich_editor.top_margin(), 5);
assert_eq!(rich_editor.bottom_margin(), 5);
```

Keep the source-mode assertions at left/right 16 and top 16, and add:

```rust
assert_eq!(source_editor.bottom_margin(), 24);
```

- [x] **Step 2: Run the focused test and verify RED**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test editor_canvas --locked
```

Expected: FAIL because the current Rich Text left margin is 40 instead of 8.

- [x] **Step 3: Implement the minimal spacing change**

Change only the Rich Text tuple in `configure_editor_canvas`:

```rust
let (horizontal, top, bottom) = if rich_mode {
    (8, 5, 5)
} else {
    (16, 16, 24)
};
```

- [x] **Step 4: Run focused and regression verification**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo +1.85.0 test -p noor-notes --test editor_canvas --locked
cargo fmt --all -- --check
cargo +1.85.0 check --workspace --locked
git diff --check
```

Expected: all commands exit successfully with no formatting or diff errors.

- [x] **Step 5: Commit the implementation**

```bash
git add apps/noor-notes/src/ui/editor_canvas.rs apps/noor-notes/tests/editor_canvas.rs docs/superpowers/plans/2026-08-17-rich-text-canvas-spacing.md
git commit -m "fix: reduce rich text writing margins"
```
