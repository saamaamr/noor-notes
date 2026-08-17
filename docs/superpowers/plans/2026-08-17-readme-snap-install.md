# README Snap Store Installation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the published Noor Notes Snap Store `latest/edge` package the primary README installation path while preserving packaged and source alternatives.

**Architecture:** This is a focused documentation-only change in `README.md`. Add one Store-first subsection inside the existing Installation section, update two stale Store-availability statements, and verify the published channel plus the final Markdown and git diff.

**Tech Stack:** Markdown, POSIX shell verification, Snapcraft/Snap Store CLI, Git.

## Global Constraints

- Present `latest/edge` as a preview channel, not as stable, candidate, or beta.
- Keep downloadable Snap, Flatpak, source installation, checksum verification, Xpad limitations, and troubleshooting guidance.
- Do not modify application code, packaging configuration, release automation, or stored-data behavior.
- Link the installation section to `https://snapcraft.io/noor-notes`.
- Document install, launch, inspect, refresh, and removal commands exactly once in the primary Store subsection.

---

### Task 1: Publish Store-first installation guidance

**Files:**
- Modify: `README.md:29-32`
- Modify: `README.md:54-93`
- Modify: `README.md:238-240`

**Interfaces:**
- Consumes: Snap Store package `noor-notes`, channel `latest/edge`, published version `0.1.1`, revision `2`.
- Produces: One README installation path that users can copy directly, plus accurate Store-status language elsewhere in the README.

- [ ] **Step 1: Run the documentation contract to verify it fails before editing**

Run:

```bash
test "$(grep -Fc 'sudo snap install noor-notes --edge' README.md)" -eq 1
test "$(grep -Fc 'sudo snap refresh noor-notes --edge' README.md)" -eq 1
test "$(grep -Fc 'sudo snap remove noor-notes' README.md)" -eq 1
test "$(grep -Fc 'https://snapcraft.io/noor-notes' README.md)" -ge 1
! grep -Fq 'Store publication is not automated' README.md
```

Expected: FAIL on the first assertion because the Store install command is not documented.

- [ ] **Step 2: Update the Engineering highlights Store statement**

Replace:

```markdown
- Snap, Flatpak, local, and Ubuntu installation paths are documented without claiming unsupported store availability.
```

With:

```markdown
- Snap Store, downloadable Snap, Flatpak, local, and Ubuntu installation paths are documented with their supported channels and sandbox limitations.
```

- [ ] **Step 3: Add the Store-first installation subsection**

In `## Installation`, replace the current method summary and insert this content before `### Release packages`:

```markdown
Choose one installation method:

- **Snap Store** for the recommended packaged installation from the current preview channel.
- **Downloaded Snap or Flatpak release** for a locally verified v0.1.1 package.
- **Ubuntu source installer** for the current repository version and host Xpad import.
- **Local rebuild** when this repository and its dependencies are already installed.

### Install from Snap Store

Noor Notes is currently published on the Snap Store's `latest/edge` preview channel. Install it, then launch it from the application grid or terminal:

```bash
sudo snap install noor-notes --edge
noor-notes
```

View the Store channels and confirm the installed revision:

```bash
snap info noor-notes
snap list noor-notes
```

Refresh an existing installation to the newest edge revision:

```bash
sudo snap refresh noor-notes --edge
```

Remove the Snap installation when it is no longer needed:

```bash
sudo snap remove noor-notes
```

See the [Noor Notes Snap Store listing](https://snapcraft.io/noor-notes) for current channel information. The edge channel is intended for preview builds; no stable, candidate, or beta release is currently published.
```

- [ ] **Step 4: Correct the release automation and Store status paragraph**

Replace the stale Store paragraph with:

```markdown
Snap Store publication remains a manual owner action rather than part of the tag workflow. Noor Notes v0.1.1 revision 2 is currently published on `latest/edge`; no stable, candidate, or beta channel release is available. A Flathub submission is not created by this project's workflow.
```

- [ ] **Step 5: Run the focused documentation checks**

Run:

```bash
test "$(grep -Fc 'sudo snap install noor-notes --edge' README.md)" -eq 1
test "$(grep -Fc 'sudo snap refresh noor-notes --edge' README.md)" -eq 1
test "$(grep -Fc 'sudo snap remove noor-notes' README.md)" -eq 1
test "$(grep -Fc 'https://snapcraft.io/noor-notes' README.md)" -ge 1
! grep -Fq 'Store publication is not automated' README.md
rg -n 'latest/(stable|candidate|beta)' README.md
```

Expected: The contract assertions pass. The final `rg` may show only explanatory text stating that those channels are not published; it must not show installation commands or availability claims for them.

- [ ] **Step 6: Verify the live Store channel**

Run:

```bash
snapcraft status noor-notes
snap info noor-notes
```

Expected: `latest/edge` reports version `0.1.1`, revision `2`; stable, candidate, and beta report no release.

- [ ] **Step 7: Inspect documentation quality and repository scope**

Run:

```bash
git diff --check
git diff -- README.md
git status --short
```

Expected: No whitespace errors; the README diff contains only the Store-first installation and accuracy updates; the two local `.snap` artifacts remain untracked and unchanged.

- [ ] **Step 8: Commit the README update**

Run:

```bash
git add README.md
git commit -m "docs: add Snap Store installation steps"
```

Expected: One documentation commit containing only `README.md`.
