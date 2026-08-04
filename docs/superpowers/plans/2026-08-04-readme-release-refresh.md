# README Release Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current incremental README with one accurate, user-centered guide for Noor Notes v0.1.0.

**Architecture:** Keep all public onboarding information in `README.md`, ordered from download and everyday use to technical development and publication details. Validate documentation against repository files and the published GitHub release rather than duplicating unverified claims.

**Tech Stack:** GitHub-flavored Markdown, Bash verification scripts, Cargo workspace tests, GitHub Releases.

## Global Constraints

- Preserve the project name and application ID `io.github.saamaamr.NoorNotes`.
- Use the released artifact names exactly: `noor-notes_0.1.0_amd64.snap`, `noor-notes.flatpak`, and `SHA256SUMS.txt`.
- Do not claim Snap Store or Flathub availability.
- State that sandboxed packages do not install the GNOME Shell extension.
- Do not include credentials, tokens, or private synchronization values.

---

### Task 1: Refresh and verify README

**Files:**
- Modify: `README.md`
- Test: `tests/store_metadata.sh`
- Test: `tests/snap_manifest.sh`
- Test: `tests/flatpak_manifest.sh`
- Test: `tests/release_workflow.sh`

**Interfaces:**
- Consumes: v0.1.0 GitHub release assets, repository installers, package manifests, application metadata, and release workflows.
- Produces: an accurate onboarding and reference document for users and contributors.

- [ ] **Step 1: Record the current documentation gaps**

Run searches that must initially fail to find the desired release download, checksum verification, feature inventory, Flatpak bundle-install command, troubleshooting, contribution, and license guidance in a coherent README.

```bash
rg -n "releases/tag/v0\.1\.0|sha256sum -c|flatpak install --user .*noor-notes\.flatpak|Rich text|Trash|Troubleshooting|Contributing|GPL-3\.0" README.md
```

- [ ] **Step 2: Rewrite README with verified information**

Update `README.md` with these sections in order: overview and v0.1.0 download, features, installation choices, artifact verification, first use/Xpad import, rich-text and trash behavior, window/sandbox limitations, encrypted sync, data/recovery, troubleshooting, development/build verification, release automation and Store status, contributing, and license.

- [ ] **Step 3: Verify documentation and package contracts**

```bash
bash tests/store_metadata.sh
bash tests/snap_manifest.sh
bash tests/flatpak_manifest.sh
bash tests/snap_workflow.sh
bash tests/flatpak_workflow.sh
bash tests/release_workflow.sh
cargo test --workspace
git diff --check
```

Expected: every command exits 0, the Cargo suite reports no failed tests, and `git diff --check` prints nothing.

- [ ] **Step 4: Verify published release references**

```bash
gh release view v0.1.0 --json assets,url --jq '{url,assets:[.assets[].name]}'
```

Expected: the release URL ends in `/releases/tag/v0.1.0` and lists all three names from Global Constraints.

- [ ] **Step 5: Commit and push**

```bash
git add README.md docs/superpowers/specs/2026-08-04-readme-release-refresh-design.md docs/superpowers/plans/2026-08-04-readme-release-refresh.md
git commit -m "docs: refresh v0.1.0 README"
git push origin main
```
