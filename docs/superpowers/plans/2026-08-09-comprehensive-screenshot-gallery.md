# Comprehensive Screenshot Gallery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Capture a complete, truthful Noor Notes visual inventory from real GTK4/libadwaita widgets using isolated sample data, then publish individual 1248 x 702 screenshots, categorized contact sheets, and an indexed gallery under `data/screenshots/`.

**Architecture:** Run a temporary non-production Noor Notes harness with a distinct application ID, temporary XDG roots, an encrypted sample database, and deterministic notes. Drive the real UI through bounded AT-SPI actions, capture active windows through GNOME Shell, normalize individual screenshots without distortion, generate contact sheets only from approved captures, and verify the gallery through an index-driven shell contract. Remove every capture-only source, database, log, and raw image before the final commit.

**Tech Stack:** Rust 1.87, GTK4, libadwaita, GtkSourceView, Noor Notes workspace crates, SQLCipher temporary storage, Python 3 with PyGObject/AT-SPI/GdkPixbuf/cairo, GNOME Shell Screenshot D-Bus API, POSIX shell, ImageMagick only if already installed.

## Global Constraints

- Use the real current Noor Notes GTK widgets; do not mock or reconstruct controls in images.
- Use only deterministic isolated sample data; never open, copy, modify, or capture personal notes or normal settings.
- Keep the installed Noor Notes process untouched.
- Keep the existing seven root screenshot filenames stable.
- Save each individual screenshot as a nonempty 1248 x 702 RGB PNG without stretching.
- Treat “minimized” as a compact non-maximized window because a truly minimized window is not visible.
- Produce both individual screenshots and labeled category/master contact sheets.
- Do not change UI, storage behavior, application identity, packaging, Snap revisions, releases, analytics, network services, or machine-level dependencies.
- Do not stage the pre-existing `noor-notes_0.1.0_amd64.snap` or `noor-notes_0.1.1_amd64.snap` files.
- Do not push without a later explicit user request.

---

### Task 1: Add an index-driven gallery verification contract

**Files:**
- Create: `tests/screenshot_gallery.sh`
- Create later in Task 5: `data/screenshots/INDEX.md`

**Interfaces:**
- Consumes: Markdown image targets in `data/screenshots/INDEX.md` and PNG files under `data/screenshots/`.
- Produces: a deterministic validation command that requires every indexed image, validates individual dimensions/color format, validates contact-sheet presence, and detects unindexed PNG assets.

- [ ] **Step 1: Write the failing gallery test**

Create `tests/screenshot_gallery.sh` with this contract:

```sh
#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
gallery="$repo_root/data/screenshots"
index="$gallery/INDEX.md"

test -s "$index" || {
    printf 'Missing screenshot index: %s\n' "$index" >&2
    exit 1
}

indexed=$(sed -n 's/.*](\([^)]*\.png\)).*/\1/p' "$index" | sort -u)
test -n "$indexed" || {
    printf 'Screenshot index contains no PNG links\n' >&2
    exit 1
}

printf '%s\n' "$indexed" | while IFS= read -r relative; do
    image="$gallery/$relative"
    test -s "$image" || {
        printf 'Missing indexed screenshot: %s\n' "$relative" >&2
        exit 1
    }
    case "$relative" in
        contact-sheets/*) ;;
        *)
            file "$image" | grep -Fq 'PNG image data, 1248 x 702' || {
                printf 'Individual screenshot is not 1248 x 702: %s\n' "$relative" >&2
                exit 1
            }
            ;;
    esac
done

actual=$(find "$gallery" -type f -name '*.png' -printf '%P\n' | sort)
if test "$indexed" != "$actual"; then
    expected_file=$(mktemp)
    actual_file=$(mktemp)
    trap 'rm -f "$expected_file" "$actual_file"' EXIT HUP INT TERM
    printf '%s\n' "$indexed" > "$expected_file"
    printf '%s\n' "$actual" > "$actual_file"
    printf 'Screenshot index and PNG inventory differ\n' >&2
    diff -u "$expected_file" "$actual_file" >&2 || true
    exit 1
fi
```

- [ ] **Step 2: Run the test and confirm the missing-index failure**

Run: `sh tests/screenshot_gallery.sh`

Expected: FAIL with `Missing screenshot index`.

- [ ] **Step 3: Make the test portable if `/dev/fd` is unavailable**

Replace the final diagnostic-only `diff` block with two `mktemp` files and a cleanup trap if the host shell reports missing `/dev/fd`. Do not weaken the equality assertion.

- [ ] **Step 4: Commit the failing contract**

```bash
git add tests/screenshot_gallery.sh
git commit -m "test: define comprehensive screenshot gallery"
```

---

### Task 2: Build the temporary isolated capture harness

**Files:**
- Temporarily create: `apps/noor-notes/examples/comprehensive_screenshot_harness.rs`
- Temporarily create: `/tmp/noor-notes-gallery/capture.py`
- Temporarily create: `/tmp/noor-notes-gallery/normalize.py`
- Temporarily create: `/tmp/noor-notes-gallery/contact_sheets.py`
- Temporarily create: `/tmp/noor-notes-gallery/manifest.tsv`
- Remove all of the above before Task 7 commits final assets.

**Interfaces:**
- Consumes: `SqliteNoteRepository::open_encrypted`, `Note::new`, `MainWindow::new`, `NoteWindow::new`, `AppearanceManager`, `AppearanceStore`, and current application CSS/resources.
- Produces: a distinct `io.github.saamaamr.NoorNotes.ComprehensiveScreenshotHarness` process backed only by `/tmp/noor-notes-gallery/xdg-data/noor-notes/notes.db`, plus accessible deterministic UI states.

- [ ] **Step 1: Record data-safety and process baselines**

Run:

```bash
pgrep -af '/noor-notes($| )' > /tmp/noor-notes-gallery-running-before.txt || true
normal_db="${XDG_DATA_HOME:-$HOME/.local/share}/noor-notes/notes.db"
if test -e "$normal_db" && ! pgrep -x noor-notes >/dev/null; then
    stat --printf='%n %s %Y\n' "$normal_db" > /tmp/noor-notes-gallery-normal-db-before.txt
    sha256sum "$normal_db" >> /tmp/noor-notes-gallery-normal-db-before.txt
else
    printf 'normal database not hashed: absent or application running\n' > /tmp/noor-notes-gallery-normal-db-before.txt
fi
```

Expected: only read-only baseline files under `/tmp`; the installed process is not stopped.

- [ ] **Step 2: Create the temporary Rust harness**

The harness must:

1. refuse to start unless `NOOR_SCREENSHOT_ROOT` begins with `/tmp/noor-notes-gallery/`;
2. generate a fresh in-memory `DatabaseKey` and call `SqliteNoteRepository::open_encrypted` on `$NOOR_SCREENSHOT_ROOT/xdg-data/noor-notes/notes.db`;
3. seed deterministic notes named `Release planning`, `Design system notes`, `Markdown handbook`, `Unicode field notes`, `Rust command palette`, `Keyboard workflow`, `Long-form draft`, `Archived research`, `Meeting scratchpad`, and `Old checklist`;
4. assign Rich Text, Markdown, Plain Text, and Code modes, all supported note colors, tags, pinned/favorite flags, and active/archived/trashed states;
5. include the searchable phrases `Design system`, `design system`, `interface`, and `release-2026` for exact case, whole-word, replacement, and regex states;
6. install the production CSS and use production `MainWindow` and `NoteWindow` constructors;
7. create `adw::Application` with application ID `io.github.saamaamr.NoorNotes.ComprehensiveScreenshotHarness` and `gio::ApplicationFlags::NON_UNIQUE`;
8. expose only capture-only `gio::SimpleAction` state transitions needed to set window size, theme, section, selected note, editor mode, popover, dialog, find options, zoom, word wrap, and View-Only Mode;
9. never reference the normal application data/config/cache paths.

- [ ] **Step 3: Compile the temporary harness**

Run: `cargo build -p noor-notes --example comprehensive_screenshot_harness`

Expected: PASS and create `target/debug/examples/comprehensive_screenshot_harness`.

- [ ] **Step 4: Create a bounded accessibility controller**

`/tmp/noor-notes-gallery/capture.py` must provide these concrete bounded helpers:

```python
import os
import subprocess
import time
from collections.abc import Sequence
from gi.repository import Atspi

def walk(root):
    yield root
    for index in range(root.get_child_count()):
        yield from walk(root.get_child_at_index(index))

def wait_for_application(name: str, timeout: float = 10.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        desktop = Atspi.get_desktop(0)
        for index in range(desktop.get_child_count()):
            candidate = desktop.get_child_at_index(index)
            if candidate.get_name() == name:
                return candidate
        time.sleep(0.1)
    raise RuntimeError(f"application not found: {name!r}")

def find(root, *, role: str | None = None, name: str | None = None, timeout: float = 10.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        for candidate in walk(root):
            role_ok = role is None or candidate.get_role_name() == role
            name_ok = name is None or candidate.get_name() == name
            if role_ok and name_ok:
                return candidate
        time.sleep(0.1)
    raise RuntimeError(f"accessible not found: role={role!r}, name={name!r}")

def click(node):
    actions = node.get_action_iface()
    if actions is None or not actions.do_action(0):
        raise RuntimeError(f"cannot activate accessible: {node.get_name()!r}")

def set_text(node, value: str):
    editable = node.get_editable_text_iface()
    if editable is None or not editable.set_text_contents(value):
        raise RuntimeError(f"cannot edit accessible: {node.get_name()!r}")

def press(keysym: str, modifiers: Sequence[str] = ()):
    for modifier in modifiers:
        Atspi.generate_keyboard_event(0, modifier, Atspi.KeySynthType.PRESS)
    Atspi.generate_keyboard_event(0, keysym, Atspi.KeySynthType.PRESSRELEASE)
    for modifier in reversed(modifiers):
        Atspi.generate_keyboard_event(0, modifier, Atspi.KeySynthType.RELEASE)

def wait_for_name(root, name: str, timeout: float = 10.0):
    return find(root, name=name, timeout=timeout)

def screenshot_window(raw_path: str):
    if os.path.exists(raw_path):
        os.unlink(raw_path)
    completed = subprocess.run([
        "gdbus", "call", "--session",
        "--dest", "org.gnome.Shell.Screenshot",
        "--object-path", "/org/gnome/Shell/Screenshot",
        "--method", "org.gnome.Shell.Screenshot.ScreenshotWindow",
        "true", "false", "false", raw_path,
    ], check=True, capture_output=True, text=True)
    if "true" not in completed.stdout.lower() or not os.path.isfile(raw_path):
        raise RuntimeError(f"window screenshot failed: {completed.stdout.strip()}")
```

Every tree walk must be scoped to the harness application, use a monotonic deadline, and raise an error containing the missing role/name. `screenshot_window` must call `org.gnome.Shell.Screenshot.ScreenshotWindow(true, false, false, raw_path)` and verify the returned success flag and output file.

- [ ] **Step 5: Launch and prove isolation**

Run:

```bash
NOOR_SCREENSHOT_ROOT=/tmp/noor-notes-gallery \
XDG_DATA_HOME=/tmp/noor-notes-gallery/xdg-data \
XDG_CONFIG_HOME=/tmp/noor-notes-gallery/xdg-config \
XDG_CACHE_HOME=/tmp/noor-notes-gallery/xdg-cache \
target/debug/examples/comprehensive_screenshot_harness \
>/tmp/noor-notes-gallery/harness.log 2>&1 &
```

Expected: the temporary encrypted database exists, AT-SPI exposes the harness UI, the log contains no normal Noor Notes data path, and any installed Noor Notes process remains unchanged.

---

### Task 3: Capture the library, lifecycle, and menu inventory

**Files:**
- Create: `data/screenshots/library/*.png`
- Create: `data/screenshots/menus/*.png`
- Replace later in Task 6: the applicable root screenshots.

**Interfaces:**
- Consumes: the running harness and capture controller from Task 2.
- Produces: normalized individual screenshots for every library navigation, search, sorting, card, lifecycle, and primary menu state.

- [ ] **Step 1: Define the exact library manifest rows**

Add these targets to `/tmp/noor-notes-gallery/manifest.tsv`, with a tab-separated category, filename, window state, theme, and UI state:

```text
library	library/maximized-all-notes.png	maximized	light	All Notes with selected card and preview
library	library/restored-all-notes.png	restored	light	All Notes restored window
library	library/compact-all-notes.png	compact	light	compact non-maximized window
library	library/selected-card-preview.png	maximized	light	selected card and preview
library	library/pinned.png	maximized	light	Pinned
library	library/favorites.png	maximized	light	Favorites
library	library/recent.png	maximized	light	Recent
library	library/tags.png	maximized	light	Tags
library	library/archived.png	maximized	light	Archived
library	library/trash.png	maximized	light	Trash
library	library/search-results.png	maximized	light	search design
library	library/search-no-results.png	maximized	light	search no-match-2026
library	library/empty-pinned.png	maximized	light	empty Pinned state
library	library/empty-archive.png	maximized	light	empty Archived state
library	library/empty-trash.png	maximized	light	empty Trash state
library	library/sort-updated.png	maximized	light	sort Recently updated
library	library/sort-created.png	maximized	light	sort Recently created
library	library/sort-title-az.png	maximized	light	sort Title A-Z
library	library/sort-title-za.png	maximized	light	sort Title Z-A
library	library/active-card-archive-action.png	maximized	light	selected active card quick Archive
library	library/active-card-menu.png	maximized	light	active card context menu
library	library/archived-card-menu.png	maximized	light	archived card context menu
library	library/trash-card-menu.png	maximized	light	trash card context menu
library	library/trash-restore-selected.png	maximized	light	selected trash Restore action
library	library/permanent-delete-confirmation.png	maximized	light	permanent delete confirmation
menus	menus/application-menu.png	maximized	light	main application menu
menus	menus/appearance-submenu.png	maximized	light	Appearance submenu
menus	menus/sort-menu.png	maximized	light	sort menu
menus	menus/active-card-context.png	maximized	light	active card actions
menus	menus/archived-card-context.png	maximized	light	archived card actions
menus	menus/trash-card-context.png	maximized	light	trash card actions
menus	menus/keyboard-shortcuts.png	maximized	light	keyboard shortcut reference
```

- [ ] **Step 2: Capture each manifest state from the real UI**

For every row, reset the harness to its deterministic seed, apply the requested theme/window state, activate the named navigation/action through AT-SPI, wait for the expected accessible label, capture only the active application window to `/tmp/noor-notes-gallery/raw/<basename>`, and record success in `/tmp/noor-notes-gallery/capture.log`.

- [ ] **Step 3: Normalize the category images**

Run `normalize.py` so each raw image is proportionally placed on a 1248 x 702 RGB canvas. Do not upscale `compact-all-notes.png`.

- [ ] **Step 4: Inspect the category**

Use an image viewer/contact proof to reject images with missing selections, closed menus, clipped rows, cursor, desktop elements, or private data. Recapture rejected states before continuing.

---

### Task 4: Capture editor, formatting, search, modes, and View-Only inventory

**Files:**
- Create: `data/screenshots/editor/*.png`
- Create: `data/screenshots/formatting/*.png`
- Create: `data/screenshots/search/*.png`
- Create: `data/screenshots/modes/*.png`
- Create: `data/screenshots/view-only/*.png`

**Interfaces:**
- Consumes: the running harness, deterministic note IDs, capture controller, and normalization script.
- Produces: individual real-UI screenshots for editor productivity, formatting, find/replace, editor modes, source features, and reading mode.

- [ ] **Step 1: Add the exact editor and View-Only rows**

```text
editor	editor/maximized-rich-editor.png	maximized	light	Design system notes
editor	editor/restored-rich-editor.png	restored	light	Design system notes
editor	editor/compact-rich-editor.png	compact	light	Design system notes
editor	editor/narrow-toolbar-wrap.png	narrow	light	wrapped toolbar
editor	editor/short-multicolumn-more.png	short	light	multi-column More menu
editor	editor/header-archive-delete.png	restored	light	header lifecycle actions
editor	editor/undo-redo-enabled.png	restored	light	five edits then two undo steps
editor	editor/saved-status.png	restored	light	Saved
editor	editor/unsaved-status.png	restored	light	Unsaved before debounce
editor	editor/note-colour-menu.png	restored	light	note color menu
editor	editor/window-settings.png	restored	light	window settings menu
editor	editor/export-menu.png	restored	light	export menu
editor	editor/editor-mode-menu.png	restored	light	editor mode controls
editor	editor/go-to-line-dialog.png	restored	light	Go to line
editor	editor/zoom-125.png	restored	light	125 percent zoom
editor	editor/word-wrap-on.png	restored	light	word wrap enabled
editor	editor/trash-confirmation.png	restored	light	Move to Trash confirmation
view-only	view-only/menu-entry.png	restored	light	View Only action in More
view-only	view-only/full-reading-window.png	maximized	light	View-Only Mode
view-only	view-only/compact-reading-window.png	compact	light	compact View-Only Mode
view-only	view-only/library-card-and-preview.png	maximized	light	selected card reading preview
```

- [ ] **Step 2: Add the exact formatting rows**

```text
formatting	formatting/popover-overview.png	restored	light	formatting overview
formatting	formatting/bold-italic-selected.png	restored	light	bold and italic selected
formatting	formatting/underline-strikethrough.png	restored	light	underline and strikethrough selected
formatting	formatting/bullet-list.png	restored	light	bullet list
formatting	formatting/numbered-list.png	restored	light	numbered list
formatting	formatting/checklist.png	restored	light	checklist
formatting	formatting/alignment-controls.png	restored	light	paragraph alignment
formatting	formatting/font-size-presets.png	restored	light	font-size presets
formatting	formatting/custom-font-size.png	restored	light	custom font size
formatting	formatting/text-colour-presets.png	restored	light	text color presets
formatting	formatting/custom-text-colour.png	restored	light	custom text color picker
formatting	formatting/highlight-presets.png	restored	light	highlight presets
formatting	formatting/custom-highlight-colour.png	restored	light	custom highlight picker
formatting	formatting/clear-formatting.png	restored	light	Clear Formatting
formatting	formatting/emoji-picker.png	restored	light	emoji picker
```

- [ ] **Step 3: Add the exact search and mode rows**

```text
search	search/find-panel.png	restored	light	Find design
search	search/replace-panel.png	restored	light	Replace design with interface
search	search/result-count.png	restored	light	nonzero result count
search	search/no-results.png	restored	light	no result state
search	search/match-case.png	restored	light	Match case
search	search/whole-word.png	restored	light	Whole word
search	search/regex.png	restored	light	regex release-[0-9]+
search	search/replace-result.png	restored	light	replacement applied
modes	modes/rich-text.png	restored	light	Rich Text
modes	modes/markdown.png	restored	light	Markdown syntax
modes	modes/plain-text.png	restored	light	uniform Plain Text
modes	modes/code-rust.png	restored	light	Rust Code syntax
modes	modes/source-line-numbers.png	restored	light	line numbers
modes	modes/source-current-line.png	restored	light	current-line highlight
modes	modes/source-bookmark.png	restored	light	bookmark
modes	modes/conversion-confirmation.png	restored	light	mode conversion confirmation
```

- [ ] **Step 4: Capture and normalize all editor-related rows**

Reset the seed before destructive/lifecycle states. Select real text before activating formatting states. Use real keyboard shortcuts for find, replace, go-to-line, undo, redo, zoom, and Escape. Wait for the correct accessible state and ensure source-mode controls truthfully match capabilities. Normalize each accepted raw capture to 1248 x 702 RGB.

- [ ] **Step 5: Inspect all editor-related captures**

Reject captures with a truncated title, detached transient, hidden selection, incorrect mode palette, formatting controls enabled in unsupported source modes, missing status values, or menu columns clipped by the screen edge.

---

### Task 5: Capture themes, responsive states, settings, and remaining menus

**Files:**
- Create: `data/screenshots/themes/*.png`
- Create: `data/screenshots/responsive/*.png`
- Create: `data/screenshots/settings/*.png`
- Create additional: `data/screenshots/menus/*.png`

**Interfaces:**
- Consumes: production appearance manager, real responsive breakpoints, and the same deterministic capture environment.
- Produces: complete theme/library/editor pairs, adaptive state proof, and all remaining settings/menu surfaces.

- [ ] **Step 1: Add the exact theme rows**

```text
themes	themes/light-library.png	maximized	light	library
themes	themes/light-editor.png	restored	light	editor
themes	themes/graphite-library.png	maximized	graphite	library
themes	themes/graphite-editor.png	restored	graphite	editor
themes	themes/midnight-library.png	maximized	midnight	library
themes	themes/midnight-editor.png	restored	midnight	editor
themes	themes/oled-library.png	maximized	oled	library
themes	themes/oled-editor.png	restored	oled	editor
themes	themes/midnight-markdown.png	restored	midnight	Markdown palette
themes	themes/oled-code.png	restored	oled	Code palette
```

- [ ] **Step 2: Add responsive, settings, and remaining menu rows**

```text
responsive	responsive/maximized-three-pane.png	maximized	light	three-pane layout
responsive	responsive/restored-three-pane.png	restored	light	three-pane layout
responsive	responsive/compact-library.png	compact	light	compact non-maximized
responsive	responsive/narrow-list.png	narrow	light	preview hidden
responsive	responsive/narrow-editor.png	narrow	light	toolbar wraps
responsive	responsive/short-editor-more.png	short	light	multi-column More
responsive	responsive/compact-view-only.png	compact	light	reading mode
settings	settings/appearance-settings.png	restored	light	Appearance Settings
settings	settings/appearance-settings-dark.png	restored	midnight	Appearance Settings
settings	settings/keyboard-shortcuts.png	restored	light	Keyboard Shortcuts
menus	menus/editor-more.png	restored	light	More note actions
menus	menus/editor-more-multicolumn.png	short	light	multi-column More
menus	menus/editor-view-options.png	restored	light	Editor view options
menus	menus/editor-mode.png	restored	light	Editor mode
menus	menus/note-colour.png	restored	light	note color
menus	menus/export.png	restored	light	export
menus	menus/emoji.png	restored	light	emoji
menus	menus/move-to-trash-confirmation.png	restored	light	trash confirmation
menus	menus/permanent-delete-confirmation.png	maximized	light	permanent delete confirmation
```

- [ ] **Step 3: Capture and normalize all rows**

For each theme pair, verify the root theme CSS class, icon contrast, selected state, paper contrast, and source palette before capture. For each responsive row, set the exact harness size and wait for layout reflow before capture. Normalize all individual outputs to 1248 x 702 RGB without upscaling compact captures.

- [ ] **Step 4: Inspect cross-theme and responsive consistency**

Compare theme pairs side by side. Reject images with stale icon colors, illegible text, a light transient on a dark root, incorrect source palette, preview visible below the narrow breakpoint, toolbar clipping, or missing menu columns.

---

### Task 6: Refresh root images, write the index, and generate contact sheets

**Files:**
- Replace: `data/screenshots/noor-notes-library.png`
- Replace: `data/screenshots/noor-notes-editor.png`
- Replace: `data/screenshots/noor-notes-dark.png`
- Replace: `data/screenshots/noor-notes-formatting.png`
- Replace: `data/screenshots/noor-notes-find-replace.png`
- Replace: `data/screenshots/noor-notes-trash.png`
- Replace: `data/screenshots/noor-notes-responsive.png`
- Create: `data/screenshots/INDEX.md`
- Create: `data/screenshots/contact-sheets/library.png`
- Create: `data/screenshots/contact-sheets/editor.png`
- Create: `data/screenshots/contact-sheets/formatting.png`
- Create: `data/screenshots/contact-sheets/search.png`
- Create: `data/screenshots/contact-sheets/modes.png`
- Create: `data/screenshots/contact-sheets/view-only.png`
- Create: `data/screenshots/contact-sheets/menus.png`
- Create: `data/screenshots/contact-sheets/themes.png`
- Create: `data/screenshots/contact-sheets/responsive.png`
- Create: `data/screenshots/contact-sheets/settings.png`
- Create: `data/screenshots/contact-sheets/all-features.png`

**Interfaces:**
- Consumes: all approved individual screenshots from Tasks 3–5 and `/tmp/noor-notes-gallery/manifest.tsv`.
- Produces: stable root gallery images, a complete indexed inventory, ten category sheets, one master sheet, and a passing screenshot-gallery contract.

- [ ] **Step 1: Map approved captures to root filenames**

Copy only these normalized approved images:

```text
library/maximized-all-notes.png -> noor-notes-library.png
editor/maximized-rich-editor.png -> noor-notes-editor.png
themes/midnight-library.png -> noor-notes-dark.png
formatting/popover-overview.png -> noor-notes-formatting.png
search/replace-panel.png -> noor-notes-find-replace.png
library/trash.png -> noor-notes-trash.png
responsive/narrow-list.png -> noor-notes-responsive.png
```

- [ ] **Step 2: Generate `INDEX.md` from the reviewed manifest**

The index must contain:

- capture date and current Git commit;
- statement that all content uses isolated sample data;
- explanation of compact versus truly minimized windows;
- one heading per category;
- a Markdown image link and one-sentence visible-state description for every PNG;
- a limitations section naming any state that could not be captured truthfully;
- a contact-sheet section linking all eleven sheets.

- [ ] **Step 3: Generate category contact sheets**

Use `contact_sheets.py` to read only indexed individual images. Create a 4-column grid with a neutral category-appropriate background, 24-pixel outer padding, 16-pixel gaps, proportional thumbnails, and a 24-pixel text label below each thumbnail. Never crop application content.

- [ ] **Step 4: Generate the master contact sheet**

Build `contact-sheets/all-features.png` from the ten category contact sheets, with a visible category title above each section. Keep labels legible at full resolution.

- [ ] **Step 5: Run the gallery contract**

Run: `sh tests/screenshot_gallery.sh`

Expected: PASS with every indexed PNG present, every individual image exactly 1248 x 702, contact sheets present, and no unindexed PNGs.

- [ ] **Step 6: Run the established store contract**

Run: `sh tests/store_metadata.sh`

Expected: PASS for the seven stable root assets.

---

### Task 7: Visual audit, cleanup, final verification, and focused commits

**Files:**
- Remove: `apps/noor-notes/examples/comprehensive_screenshot_harness.rs`
- Remove: `/tmp/noor-notes-gallery/`
- Commit: `data/screenshots/**`, `tests/screenshot_gallery.sh`, and the plan/spec documentation only.

**Interfaces:**
- Consumes: the complete indexed gallery and temporary capture environment.
- Produces: a clean repository containing only reviewed documentation assets and their validation contract.

- [ ] **Step 1: Visually inspect every individual image**

Open every PNG at original resolution and check: correct state, readable text, no clipping, no cursor, no terminal/desktop clutter, no personal data, correct theme, correct window size, correct transient placement, and truthful enabled/selected controls.

- [ ] **Step 2: Visually inspect all contact sheets**

Confirm every thumbnail is legible, correctly labeled, uncropped, and belongs to the stated category. Confirm the master sheet contains all ten categories.

- [ ] **Step 3: Recheck normal-data safety**

Repeat the Task 2 baseline command into `/tmp/noor-notes-gallery-normal-db-after.txt`. Compare before/after when hashes were permitted. Confirm the installed Noor Notes process list is unchanged except for natural PID changes not caused by this workflow.

- [ ] **Step 4: Stop only the harness and remove capture-only material**

Resolve the exact harness PID from its distinct executable path, terminate only that PID, then remove the explicit temporary files and `/tmp/noor-notes-gallery/`. Remove `apps/noor-notes/examples/comprehensive_screenshot_harness.rs` through `apply_patch`. Do not use a broad recursive target, and do not touch the installed application or normal XDG directories.

- [ ] **Step 5: Run final validation**

Run:

```bash
sh tests/screenshot_gallery.sh
sh tests/store_metadata.sh
git diff --check
git status --short
```

Expected: both tests pass; the diff has no whitespace errors; only intended screenshot, index, test, spec, and plan changes appear, plus the two pre-existing untracked Snap artifacts.

- [ ] **Step 6: Confirm no temporary or sensitive files are staged**

Run:

```bash
git diff --cached --name-only | rg '(notes\.db|\.db-wal|\.db-shm|harness\.log|raw/|capture\.py|normalize\.py|contact_sheets\.py|\.snap$)' && exit 1 || true
```

Expected: no match.

- [ ] **Step 7: Commit the gallery assets**

```bash
git add data/screenshots tests/screenshot_gallery.sh
git commit -m "docs: add comprehensive interface gallery"
```

- [ ] **Step 8: Commit the execution notes if the plan changed during capture**

If actual capture limitations required plan annotations, stage only this plan and commit:

```bash
git add docs/superpowers/plans/2026-08-09-comprehensive-screenshot-gallery.md
git commit -m "docs: record screenshot gallery verification"
```

- [ ] **Step 9: Report without pushing**

Report exact directories, individual and contact-sheet counts, root filenames, validation results, isolation evidence, truthful capture limitations, commit hashes, current branch, and Git status. State explicitly that no personal note data, Snap action, release, package installation, or GitHub push occurred.
