# README Stable Snap Installation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the README present the Snap Store's stable Noor Notes release as the default installation while retaining edge as an optional testing channel.

**Architecture:** This is a focused documentation-only change in `README.md`. Replace stale preview-only Store text with stable-first commands, add a compact edge opt-in path, and update the release-status paragraph from the live Snap Store channel map.

**Tech Stack:** Markdown, Snap/Snapcraft CLI, Git

## Global Constraints

- Change only Snap Store installation, channel-selection, refresh, and current Store-status text in `README.md`.
- Preserve downloaded Snap, Flatpak, Ubuntu source, local build, verification, removal, feature, and architecture documentation.
- Keep version 0.1.1 revision 2 and amd64 channel claims aligned with the live Store.
- Do not commit locally built `.snap` artifacts.

---

### Task 1: Publish stable-first README instructions

**Files:**
- Modify: `README.md:54-91`
- Modify: `README.md:267-271`

**Interfaces:**
- Consumes: Live `snapcraft status noor-notes` and `snap info noor-notes` channel metadata.
- Produces: Stable-first public installation instructions and an accurate release-status statement.

- [ ] **Step 1: Capture the live Store channel map**

Run:

```bash
snapcraft status noor-notes
snap info noor-notes
```

Expected: `latest/stable` and `latest/edge` both resolve to version `0.1.1`, revision `2`, for amd64; the default Store install is available without `--edge`.

- [ ] **Step 2: Replace preview-only installation copy with stable-first guidance**

In `README.md`, make the installation-method summary describe the Snap Store as the recommended stable packaged installation. Replace the current Store subsection with content that includes these exact primary commands:

```bash
sudo snap install noor-notes
noor-notes
```

Keep the existing verification commands:

```bash
snap info noor-notes
snap list noor-notes
```

Use this explicit stable refresh command:

```bash
sudo snap refresh noor-notes --stable
```

Add an `Optional edge channel` subsection explaining that edge is intended for testing newer builds and may change more frequently. Include both channel-switch commands:

```bash
sudo snap refresh noor-notes --edge
sudo snap refresh noor-notes --stable
```

Keep the existing removal command and Snap Store listing link. State that version 0.1.1 revision 2 is available on `latest/stable` and `latest/edge` for amd64.

- [ ] **Step 3: Update release automation and Store status**

Replace the stale statement that only edge is available with this meaning: Store publication remains a manual owner action; Noor Notes v0.1.1 revision 2 is published for amd64 on both `latest/stable` and `latest/edge`; no separate candidate or beta revision is published; Flathub submission remains outside the workflow.

- [ ] **Step 4: Run documentation contract checks**

Run:

```bash
rg -n "sudo snap install noor-notes$|sudo snap refresh noor-notes --stable|sudo snap refresh noor-notes --edge|latest/stable|latest/edge|no separate candidate or beta" README.md
git diff --check
git diff -- README.md
git status --short
```

Expected:

- The primary install command appears without `--edge`.
- Stable refresh and optional edge-switch commands are present.
- Store-status text names both live channels and does not claim candidate or beta has a separate revision.
- `git diff --check` exits successfully.
- The diff contains no unrelated README edits.
- The two local `.snap` files remain untracked and unstaged.

- [ ] **Step 5: Commit the README change**

```bash
git add README.md
git commit -m "docs: make stable Snap install the default"
```

- [ ] **Step 6: Run final verification and push**

Run:

```bash
snapcraft status noor-notes
snap info noor-notes
git diff --check HEAD^ HEAD
git status --short --branch
git push origin main
git rev-parse HEAD
git rev-parse origin/main
```

Expected: Store metadata remains stable/edge version 0.1.1 revision 2, the committed diff is clean, only the local `.snap` artifacts remain untracked, the push succeeds, and local `HEAD` equals `origin/main`.

