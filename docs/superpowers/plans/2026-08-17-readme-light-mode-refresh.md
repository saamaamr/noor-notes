# README Light Mode Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refresh the GitHub README with accurate Light and Graphite overview screenshots and concise copy describing the completed Light Mode redesign, then verify and push the result to `main`.

**Architecture:** Keep the existing README structure, image links, and full screenshot gallery. Use a temporary Rust/GTK example that constructs the real `MainWindow` from an isolated encrypted repository containing synthetic notes, renders the production UI at 1248 × 702, and writes only the two stable overview PNGs. Remove the capture harness and all temporary data before the final commit.

**Tech Stack:** Rust 1.85, GTK4 0.10, Libadwaita 0.8, Cargo, Xvfb, POSIX shell, Markdown

## Global Constraints

- Never open or copy the user's normal Noor Notes database, appearance file, or personal notes.
- Use separate temporary XDG configuration/data roots for Light and Graphite captures.
- Preserve the existing screenshot destinations and exact 1248 × 702 dimensions.
- Preserve Rich Text canvas margins at exactly 5 pixels top/bottom and 8 pixels left/right.
- Do not regenerate unrelated screenshots or alter application behavior.
- Remove the temporary capture example, databases, keys, logs, and intermediate files before committing.
- Leave `noor-notes_0.1.0_amd64.snap` and `noor-notes_0.1.1_amd64.snap` untracked and untouched.

---

### Task 1: Capture current Light and Graphite product overviews

**Files:**
- Create temporarily, then remove: `apps/noor-notes/examples/readme_overview_capture.rs`
- Replace: `data/screenshots/noor-notes-library.png`
- Replace: `data/screenshots/noor-notes-dark.png`

**Interfaces:**
- Consumes: production `MainWindow`, stylesheet, appearance manager, encrypted repository, writing-assistance runtime, and fallback window controller.
- Produces: two stable 1248 × 702 PNGs containing synthetic product data only.

- [ ] **Step 1: Confirm the existing screenshot contract is green**

Run:

```bash
bash tests/screenshot_gallery.sh
bash tests/store_metadata.sh
```

Expected: both scripts pass before any image replacement.

- [ ] **Step 2: Add a temporary isolated capture example**

Create `apps/noor-notes/examples/readme_overview_capture.rs` using the repository's existing screenshot-rendering pattern:

- open a new encrypted SQLite repository only beneath `NOOR_CAPTURE_ROOT`;
- insert a small set of clearly synthetic notes with varied note colours, pin/favourite states, tags, and one safely wrapping long string;
- create the real `MainWindow` with the production CSS and reusable components;
- select `AppearanceMode::Light` or `AppearanceMode::Graphite` from `NOOR_CAPTURE_THEME` before presenting the window;
- size the window to exactly 1248 × 702;
- wait for data refresh and GTK allocation, then render the window through `gtk::WidgetPaintable` and `gsk::Renderer` to `NOOR_CAPTURE_OUTPUT`;
- close the window and unrealize the renderer cleanly.

Do not reference any path outside the explicit temporary capture root and output image.

- [ ] **Step 3: Build and run both isolated captures**

Build the temporary example:

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo build -p noor-notes --example readme_overview_capture --locked
```

Run the Light capture under Xvfb with isolated XDG directories:

```bash
env XDG_CONFIG_HOME=/tmp/noor-notes-readme-refresh/light/config XDG_DATA_HOME=/tmp/noor-notes-readme-refresh/light/data NOOR_CAPTURE_ROOT=/tmp/noor-notes-readme-refresh/light/runtime NOOR_CAPTURE_THEME=light NOOR_CAPTURE_OUTPUT=/home/mamun/Documents/noor-notes/data/screenshots/noor-notes-library.png xvfb-run -a target/debug/examples/readme_overview_capture
```

Run the Graphite capture with a separate isolated root:

```bash
env XDG_CONFIG_HOME=/tmp/noor-notes-readme-refresh/graphite/config XDG_DATA_HOME=/tmp/noor-notes-readme-refresh/graphite/data NOOR_CAPTURE_ROOT=/tmp/noor-notes-readme-refresh/graphite/runtime NOOR_CAPTURE_THEME=graphite NOOR_CAPTURE_OUTPUT=/home/mamun/Documents/noor-notes/data/screenshots/noor-notes-dark.png xvfb-run -a target/debug/examples/readme_overview_capture
```

- [ ] **Step 4: Inspect both captures at original resolution**

Verify with `file` that both images are 1248 × 702 PNGs. Open each at original resolution and confirm:

- all displayed notes are synthetic;
- the Light screenshot shows layered panes, restrained note colours, compact card actions, and readable preview wrapping;
- the Graphite screenshot has readable preview text, hover-independent controls, and balanced pane contrast;
- no clipped content, black rendering region, modal, tooltip, cursor artifact, or personal data is visible.

If a capture fails any criterion, adjust only the temporary sample data or capture timing and recapture it.

- [ ] **Step 5: Remove the capture harness and temporary state**

Delete the exact temporary example through `apply_patch`, remove only `/tmp/noor-notes-readme-refresh`, then confirm:

```bash
test ! -e apps/noor-notes/examples/readme_overview_capture.rs
test ! -e /tmp/noor-notes-readme-refresh
```

Expected: both checks pass; only the two stable PNGs remain as intended screenshot changes.

---

### Task 2: Update GitHub-facing product copy and gallery provenance

**Files:**
- Modify: `README.md`
- Modify: `data/screenshots/INDEX.md`

- [ ] **Step 1: Refine the Product overview copy**

Immediately after the existing overview introduction, add one concise paragraph explaining that the refreshed library uses:

- clearer sidebar, note-list, and editor hierarchy;
- restrained colour rails rather than saturated full-card colour;
- compact controls and a readable preview surface;
- adaptive behavior for narrower windows.

Keep the existing overview tables and image destinations unchanged.

- [ ] **Step 2: Update the appearance/editor feature description**

Revise the existing relevant feature bullet instead of adding duplicate marketing copy. State accurately that Light Mode has calm semantic surfaces while Graphite and the other dark appearances remain supported. Add a compact editor-ergonomics sentence stating that Rich Text preserves 5-pixel top/bottom and 8-pixel left/right canvas margins.

- [ ] **Step 3: Clarify screenshot capture dates and safety**

In `data/screenshots/INDEX.md`, retain the broader gallery's 9 August 2026 provenance and add that the stable Light and Graphite overview images were refreshed on 17 August 2026 after the Light Mode update. State that the refreshed captures use isolated synthetic data and did not open a personal database.

- [ ] **Step 4: Review the documentation diff**

Confirm the copy is concise, grammatical, verifiable from the repository, and does not modify installation, privacy, security, release, limitation, or contribution guidance.

---

### Task 3: Verify, commit, and publish `main`

**Files:**
- Verify: `README.md`
- Verify: `data/screenshots/INDEX.md`
- Verify: `data/screenshots/noor-notes-library.png`
- Verify: `data/screenshots/noor-notes-dark.png`
- Verify: `docs/superpowers/specs/2026-08-17-readme-light-mode-refresh-design.md`
- Verify: `docs/superpowers/plans/2026-08-17-readme-light-mode-refresh.md`

- [ ] **Step 1: Run formatting and screenshot/document contracts**

Run:

```bash
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
bash tests/screenshot_gallery.sh
bash tests/store_metadata.sh
git diff --check
```

Expected: all commands pass.

- [ ] **Step 2: Re-run the focused Light Mode regression suite**

Run:

```bash
GDK_BACKEND=x11 xvfb-run -a cargo test -p noor-notes --test design_system --test library_ui --test rich_editor --locked
```

Expected: all tests pass, including the exact Rich Text 5/8 spacing contract and dark-theme style contracts.

- [ ] **Step 3: Inspect final scope and repository state**

Run:

```bash
git diff --stat
git diff -- README.md data/screenshots/INDEX.md docs/superpowers/plans/2026-08-17-readme-light-mode-refresh.md
git status --short
```

Expected: no temporary harness or capture state; only the intended docs/screenshots are tracked changes; the two existing Snap packages are still the only unrelated untracked files.

- [ ] **Step 4: Commit the README refresh**

Stage only the intended files:

```bash
git add README.md data/screenshots/INDEX.md data/screenshots/noor-notes-library.png data/screenshots/noor-notes-dark.png docs/superpowers/plans/2026-08-17-readme-light-mode-refresh.md
git commit -m "docs: refresh light mode product overview"
```

The already committed design specification remains part of the branch history; never add either `.snap` file.

- [ ] **Step 5: Push and prove remote synchronization**

Run:

```bash
git push origin main
git rev-parse HEAD
git rev-parse origin/main
```

Expected: the push succeeds and both commit IDs are identical.
