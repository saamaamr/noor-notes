# Current-product README Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the root README with the current Noor Notes interface, working features, appearance system, and supported installation paths.

**Architecture:** Change documentation only. Consolidate repeated feature claims into product-oriented groups, separate editor-mode and appearance guidance, preserve verified packaging and recovery details, and add source-install launch troubleshooting.

**Tech Stack:** Markdown, Bash command examples, Git.

## Global Constraints

- Do not modify application source, database code, package identity, release artifacts, or Snap Store state.
- Preserve verified v0.1.1 release filenames, checksum instructions, data locations, and sandbox limitations.
- Describe unavailable release functionality as unavailable; do not imply future or unverified features work.
- Do not stage the existing untracked Snap artifacts.

---

### Task 1: Refresh and verify the root README

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: Current application behavior documented by tests, recent commits, install scripts, and package metadata.
- Produces: A self-contained installation and usage guide for source, Snap, and Flatpak users.

- [ ] **Step 1: Replace the repetitive feature list**

Organize current capabilities under Library, Editor, Appearance, Privacy and data safety, and Desktop integration. Include the adaptive three-pane library, rich/source modes, find and replace, formatting persistence, autosave state, Light/System/Graphite/Midnight/OLED modes, synchronized windows, adaptive symbolic icon colors, encrypted local storage, Xpad import, and supported window controls.

- [ ] **Step 2: Repair usage documentation**

Move the split Editor mode text into a complete `## Editor modes` section. Keep `## Appearance and dark palettes` independent and document the header switch, direct menu choices, Appearance Settings, persistence, system following, and semantic icon colors.

- [ ] **Step 3: Clarify installation paths**

Keep the published v0.1.1 Snap and Flatpak commands. Explain `./scripts/install-ubuntu.sh` for first source installation, `./scripts/install-local.sh` for rebuilding an existing checkout, and `~/.local/bin/noor-notes` for direct launch diagnostics.

- [ ] **Step 4: Improve troubleshooting**

Add source-install guidance to reinstall after pulling changes, quit older running processes before reopening, and run the local binary in a terminal to capture startup errors. Do not suggest deleting notes, configuration, keyring entries, or databases.

- [ ] **Step 5: Validate documentation**

Run:

```bash
rg -n '^## ' README.md
rg -n 'TODO|TBD|Choose \*\*Editor mode|adaptive symbolic icons|install-local' README.md
bash tests/install_ubuntu.sh
bash tests/store_metadata.sh
git diff --check
git status --short
```

Expected: coherent section order, no placeholders or split paragraph, both repository documentation contracts pass, no whitespace errors, and only `README.md` plus the plan are changed before commit.

- [ ] **Step 6: Commit documentation**
