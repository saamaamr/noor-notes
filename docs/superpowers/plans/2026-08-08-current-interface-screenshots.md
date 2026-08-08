# Current Interface Screenshot Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the obsolete Noor Notes screenshots and publish a seven-image gallery of the real current GTK interface using isolated, non-personal sample data.

**Architecture:** Add metadata contracts for the complete gallery, then run a temporary uncommitted screenshot harness with its own application ID, encrypted temporary database, and sample notes. Drive real GTK controls through AT-SPI, capture only the active application window through GNOME Shell's screenshot D-Bus API, and normalize each capture to a 1248 x 702 RGB PNG before visual review.

**Tech Stack:** Rust 1.87, GTK4/libadwaita, Noor Notes workspace crates, Python 3 with GI AT-SPI/GdkPixbuf and cairo, GNOME Shell Screenshot D-Bus API, AppStream XML, shell tests.

## Global Constraints

- Every final image is a nonempty 1248 x 702 RGB PNG under `data/screenshots/`.
- Use real running Noor Notes GTK widgets; do not compose or add interface controls after capture.
- Use a temporary database and non-personal sample notes; never open or modify the normal notes database.
- Do not add dependencies, alter storage, change application identity, modify Snap metadata, upload packages, or publish a release.
- Exclude desktop panels, docks, notifications, terminals, cursors, and unrelated windows.
- Temporary harnesses, scripts, databases, logs, and raw captures must not be committed.
- Preserve the existing untracked Snap artifacts without staging or modifying them.

---

### Task 1: Define the seven-image store and documentation contract

**Files:**
- Modify: `tests/store_metadata.sh`
- Modify: `data/io.github.saamaamr.NoorNotes.metainfo.xml`

**Interfaces:**
- Consumes: the approved filenames in `docs/superpowers/specs/2026-08-08-current-interface-screenshots-design.md`.
- Produces: an AppStream gallery and a shell test requiring all seven exact image paths and dimensions.

- [ ] **Step 1: Extend the store test before adding metadata**

Replace the two hard-coded screenshot variables with the exact gallery list:

```sh
screenshots='noor-notes-editor.png
noor-notes-library.png
noor-notes-dark.png
noor-notes-formatting.png
noor-notes-find-replace.png
noor-notes-trash.png
noor-notes-responsive.png'
```

For each name, require the raw GitHub URL in AppStream and validate the local file:

```sh
for name in $screenshots; do
    require "https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/$name" "$metadata"
    screenshot="$repo_root/data/screenshots/$name"
    test -s "$screenshot" || {
        printf 'Missing required screenshot: %s\n' "$screenshot" >&2
        exit 1
    }
    file "$screenshot" | grep -Fq 'PNG image data, 1248 x 702' || {
        printf 'Screenshot is not a 1248 x 702 PNG: %s\n' "$screenshot" >&2
        exit 1
    }
done
```

- [ ] **Step 2: Run the contract and confirm the new files are missing**

Run: `bash tests/store_metadata.sh`

Expected: FAIL mentioning `noor-notes-dark.png` or its missing AppStream URL.

- [ ] **Step 3: Add accurate AppStream screenshot entries**

Keep Editor as the default screenshot and add entries with these exact captions and paths:

```xml
<screenshot><caption>Browse and preview private notes.</caption><image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-library.png</image></screenshot>
<screenshot><caption>Use a focused dark appearance.</caption><image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-dark.png</image></screenshot>
<screenshot><caption>Apply persistent rich text and colours.</caption><image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-formatting.png</image></screenshot>
<screenshot><caption>Find and replace text inside a note.</caption><image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-find-replace.png</image></screenshot>
<screenshot><caption>Restore notes safely from Trash.</caption><image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-trash.png</image></screenshot>
<screenshot><caption>Work comfortably in a narrow window.</caption><image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-responsive.png</image></screenshot>
```

- [ ] **Step 4: Validate the XML structure while image files are still pending**

Run: `appstreamcli validate --no-net data/io.github.saamaamr.NoorNotes.metainfo.xml`

Expected: XML parses successfully; remote screenshot reachability is not checked because `--no-net` is used.

- [ ] **Step 5: Keep the failing contract pending until the images exist**

Do not commit a deliberately failing store contract. Keep the two reviewed text changes unstaged and include them in Task 3's screenshot commit after all seven images pass the contract.

---

### Task 2: Build an isolated temporary capture environment

**Files:**
- Temporarily create, then remove: `apps/noor-notes/examples/screenshot_harness.rs`
- Temporarily create, then remove: `/tmp/noor-notes-screenshot-capture.py`
- Create during capture only: `/tmp/noor-notes-screenshot-refresh/`

**Interfaces:**
- Consumes: `SqliteNoteRepository::open_encrypted`, `SqliteNoteRepository::save_note`, `MainWindow::new`, `NoteWindow::new`, and the current compiled application CSS.
- Produces: a running non-unique screenshot application backed only by `/tmp/noor-notes-screenshot-refresh/data/notes.db`, plus deterministic accessible sample states.

- [ ] **Step 1: Record the real-data safety baseline**

Record the path, size, modification time, and hash when present without opening the database:

```bash
real_db="${XDG_DATA_HOME:-$HOME/.local/share}/noor-notes/notes.db"
stat --printf='%n %s %Y\n' "$real_db" > /tmp/noor-notes-real-db-before.txt
sha256sum "$real_db" >> /tmp/noor-notes-real-db-before.txt
```

If the file does not exist, record `normal database absent` instead. Do not hash WAL/SHM files while a normal Noor Notes process is running; first check `pgrep -x noor-notes` and stop for user direction if one exists.

- [ ] **Step 2: Create a temporary Rust harness with a distinct application ID**

Create `apps/noor-notes/examples/screenshot_harness.rs` with a Tokio main that:

1. removes and recreates `/tmp/noor-notes-screenshot-refresh/data`;
2. generates an in-memory `DatabaseKey` and opens `notes.db` with `SqliteNoteRepository::open_encrypted`;
3. creates seven deterministic `Note::new` values with titles including `Release planning`, `Design system notes`, `Markdown handbook`, `Keyboard workflow`, and two Trash recovery examples;
4. sets sample tags, colours, pinned/favorite flags, rich content, and `NoteState::Trashed` directly before `save_note`;
5. installs the current CSS and appearance manager;
6. creates `adw::Application` with application ID `io.github.saamaamr.NoorNotes.ScreenshotHarness` and `gio::ApplicationFlags::NON_UNIQUE`;
7. presents `MainWindow::new` with `AutosaveQueue` and `FallbackWindowController`.

The visible title, widgets, CSS, and note content remain Noor Notes; only the private D-Bus application identity differs to avoid activating a running production instance.

- [ ] **Step 3: Compile the harness without changing Cargo manifests**

Run: `PATH=/home/mamun/.cargo/bin:$PATH cargo build -p noor-notes --example screenshot_harness`

Expected: PASS and create `target/debug/examples/screenshot_harness`.

- [ ] **Step 4: Create the temporary AT-SPI capture controller**

Create `/tmp/noor-notes-screenshot-capture.py` using `gi.repository.Atspi` and `subprocess.run`. It must expose these bounded helpers:

```python
import subprocess
import time
from gi.repository import Atspi

def walk(root):
    yield root
    for index in range(root.get_child_count()):
        yield from walk(root.get_child_at_index(index))

def application(name: str, timeout_seconds: int):
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        desktop = Atspi.get_desktop(0)
        for index in range(desktop.get_child_count()):
            candidate = desktop.get_child_at_index(index)
            if candidate.get_name() == name:
                return candidate
        time.sleep(0.1)
    raise RuntimeError(f"application not found: {name}")

def descendant(root, role_name: str | None = None, name: str | None = None):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        for candidate in walk(root):
            role_matches = role_name is None or candidate.get_role_name() == role_name
            name_matches = name is None or candidate.get_name() == name
            if role_matches and name_matches:
                return candidate
        time.sleep(0.1)
    raise RuntimeError(f"accessible not found: role={role_name!r}, name={name!r}")

def click(node):
    action = node.get_action_iface()
    if action is None or not action.do_action(0):
        raise RuntimeError(f"cannot activate accessible: {node.get_name()!r}")

def set_text(node, text: str):
    editable = node.get_editable_text_iface()
    if editable is None or not editable.set_text_contents(text):
        raise RuntimeError(f"cannot edit accessible: {node.get_name()!r}")

def capture_active_window(raw_path: str):
    subprocess.run([
        "gdbus", "call", "--session", "--dest", "org.gnome.Shell.Screenshot",
        "--object-path", "/org/gnome/Shell/Screenshot",
        "--method", "org.gnome.Shell.Screenshot.ScreenshotWindow",
        "true", "false", "false", raw_path,
    ], check=True)
```

Every lookup must use a 10-second deadline and fail with the missing role/name instead of clicking by screen coordinates. The script must never search for or activate an application other than the screenshot harness.

- [ ] **Step 5: Launch and prove isolation**

Run the harness with temporary configuration and cache roots:

```bash
XDG_DATA_HOME=/tmp/noor-notes-screenshot-refresh/xdg-data \
XDG_CONFIG_HOME=/tmp/noor-notes-screenshot-refresh/xdg-config \
XDG_CACHE_HOME=/tmp/noor-notes-screenshot-refresh/xdg-cache \
target/debug/examples/screenshot_harness \
>/tmp/noor-notes-screenshot-refresh/harness.log 2>&1 &
```

Expected: AT-SPI sees `Noor Notes`, the temporary encrypted database exists, and no normal database path appears in `harness.log`.

---

### Task 3: Capture and normalize all seven real application states

**Files:**
- Replace: `data/screenshots/noor-notes-library.png`
- Replace: `data/screenshots/noor-notes-editor.png`
- Create: `data/screenshots/noor-notes-dark.png`
- Create: `data/screenshots/noor-notes-formatting.png`
- Create: `data/screenshots/noor-notes-find-replace.png`
- Create: `data/screenshots/noor-notes-trash.png`
- Create: `data/screenshots/noor-notes-responsive.png`

**Interfaces:**
- Consumes: the real harness windows and the bounded AT-SPI/D-Bus capture controller from Task 2.
- Produces: seven visually reviewed 1248 x 702 RGB PNG assets using the approved filenames.

- [ ] **Step 1: Capture the light library state**

Wait until the footer reports the populated All Notes count, select `Release planning`, focus its card, and capture the active library window to `/tmp/noor-notes-screenshot-refresh/raw-library.png`.

- [ ] **Step 2: Capture the rich editor state**

Activate `Design system notes`, confirm the editor title and `Saved` state are visible, and capture `/tmp/noor-notes-screenshot-refresh/raw-editor.png`.

- [ ] **Step 3: Capture the formatting state**

Select representative body text, activate the `Formatting` toolbar button, confirm the compact text-style and colour controls are visible, and capture `/tmp/noor-notes-screenshot-refresh/raw-formatting.png`.

- [ ] **Step 4: Capture find and replace**

Close the formatting popover with Escape, activate Replace with Ctrl+H through AT-SPI key synthesis, enter `design` in Find and `interface` in Replace, confirm a nonzero result count, and capture `/tmp/noor-notes-screenshot-refresh/raw-find-replace.png`.

- [ ] **Step 5: Capture the dark state**

Close the editor, use the library Appearance action to select `Midnight`, wait until the CSS class and icon palette update, select `Markdown handbook`, and capture `/tmp/noor-notes-screenshot-refresh/raw-dark.png`.

- [ ] **Step 6: Capture Trash**

Return to Light appearance, activate the `Trash` sidebar row, select `Meeting scratchpad`, confirm Restore is available while permanent deletion remains in the row menu, and capture `/tmp/noor-notes-screenshot-refresh/raw-trash.png`.

- [ ] **Step 7: Capture the responsive state**

Set the harness library window default size to 720 x 620 from the harness process, wait for the preview pane to hide below the 920-pixel breakpoint, and capture `/tmp/noor-notes-screenshot-refresh/raw-responsive.png`.

- [ ] **Step 8: Normalize raw captures without distorting them**

Use a temporary Python script with cairo and `GdkPixbuf.Pixbuf` to create a 1248 x 702 RGB canvas. Scale each raw capture down proportionally only when it exceeds the canvas, center it, and fill unused space with the sampled native outer background colour. Save with PNG compression and no alpha channel.

Map raw inputs to the seven approved output files exactly. Never upscale a narrow responsive window to fill the canvas.

- [ ] **Step 9: Validate dimensions and inspect every image**

Run:

```bash
for image in data/screenshots/*.png; do
    file "$image"
done
bash tests/store_metadata.sh
```

Expected: all seven report `PNG image data, 1248 x 702` and the store test passes. Load each image for visual inspection and reject captures with clipping, hidden popovers, personal data, a cursor, or unrelated desktop UI.

- [ ] **Step 10: Commit the screenshot assets and their passing contract**

```bash
git add data/screenshots tests/store_metadata.sh data/io.github.saamaamr.NoorNotes.metainfo.xml
git commit -m "docs: refresh current interface screenshots"
```

---

### Task 4: Publish the expanded README gallery and clean the harness

**Files:**
- Modify: `README.md`
- Remove before commit: `apps/noor-notes/examples/screenshot_harness.rs`
- Remove before commit: `/tmp/noor-notes-screenshot-capture.py`
- Remove before commit: `/tmp/noor-notes-screenshot-refresh/`

**Interfaces:**
- Consumes: seven verified screenshot assets from Task 3.
- Produces: an accessible README gallery with truthful captions and a repository containing no capture-only source or data.

- [ ] **Step 1: Replace the two-column README screenshot table**

Use three concise rows:

```markdown
| Library and preview | Focused rich editor |
| --- | --- |
| ![Noor Notes library with navigation, note cards, and selected-note preview](data/screenshots/noor-notes-library.png) | ![Noor Notes rich editor with compact toolbar, writing canvas, and status bar](data/screenshots/noor-notes-editor.png) |

| Dark appearance | Rich formatting and colours |
| --- | --- |
| ![Noor Notes library using the Midnight dark appearance](data/screenshots/noor-notes-dark.png) | ![Noor Notes rich formatting popover with text and highlight colours](data/screenshots/noor-notes-formatting.png) |

| Find and replace | Trash recovery | Narrow layout |
| --- | --- | --- |
| ![Noor Notes inline find and replace panel](data/screenshots/noor-notes-find-replace.png) | ![Noor Notes Trash view with recoverable notes](data/screenshots/noor-notes-trash.png) | ![Noor Notes adaptive narrow-window layout](data/screenshots/noor-notes-responsive.png) |
```

- [ ] **Step 2: Remove all temporary capture material**

Stop only `target/debug/examples/screenshot_harness`, remove the temporary Rust example, controller, raw captures, temporary database, config, cache, and logs. Do not delete broad directories and do not touch the normal Noor Notes process or data.

- [ ] **Step 3: Prove the normal database was unchanged**

Repeat the exact `stat` and `sha256sum` commands from Task 2 into `/tmp/noor-notes-real-db-after.txt`, then run:

```bash
diff -u /tmp/noor-notes-real-db-before.txt /tmp/noor-notes-real-db-after.txt
```

Expected: no differences, or both files record `normal database absent`.

- [ ] **Step 4: Run final metadata and repository checks**

Run:

```bash
bash tests/store_metadata.sh
appstreamcli validate --no-net data/io.github.saamaamr.NoorNotes.metainfo.xml
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo test --workspace --quiet
git diff --check
git status --short
```

Expected: all commands pass. Git lists only the intended README change plus the two pre-existing untracked Snap artifacts; no example, raw screenshot, database, log, credential, or temporary file appears.

- [ ] **Step 5: Commit documentation and report without publishing**

```bash
git add README.md
git commit -m "docs: present current interface gallery"
```

Report every screenshot path, validation results, database safety comparison, current branch, commit hashes, and the unchanged untracked Snap files. Do not push unless the user explicitly requests it.
