# Linux Store Publication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce validated Snap Store and Flathub release packages for Noor Notes 0.1.0 and submit them as far as current account access allows.

**Architecture:** One immutable `v0.1.0` source release feeds two sandboxed package definitions. Shared desktop metadata and screenshots describe the same application, while store-specific manifests request only the interfaces each sandbox needs.

**Tech Stack:** Rust 1.85+, GTK4, Libadwaita, Snapcraft 9, Flatpak/flatpak-builder, AppStream, GitHub Actions.

## Global Constraints

- App ID remains `io.github.saamaamr.NoorNotes`; Snap name target remains `noor-notes`.
- Both packages use strict sandboxing and no filesystem-wide access.
- No account credentials, signing keys, or store tokens enter Git.
- The GNOME Shell extension is not bundled or installed by sandboxed store packages.
- A Git tag is created only after manifests reference and build the final tested revision.

---

### Task 1: Store metadata and validation

**Files:**
- Modify: `data/io.github.saamaamr.NoorNotes.metainfo.xml`
- Modify: `data/io.github.saamaamr.NoorNotes.desktop`
- Create: `data/screenshots/noor-notes-editor.png`
- Create: `data/screenshots/noor-notes-library.png`
- Create: `tests/store_metadata.sh`

**Interfaces:**
- Produces validated AppStream metadata, desktop entry, icon references, release notes, URLs, and screenshots consumed by both stores.

- [ ] Write `tests/store_metadata.sh` to fail unless required AppStream IDs, URLs, developer identity, release `0.1.0`, screenshots, desktop categories, icon, and executable are present; run it and confirm failure.
- [ ] Complete the AppStream and desktop metadata and capture truthful application screenshots at store-safe dimensions.
- [ ] Run `appstreamcli validate`, `desktop-file-validate`, XML parsing, SVG validation, and `tests/store_metadata.sh`; commit the green metadata unit.

### Task 2: Strict Snap package

**Files:**
- Create: `snap/snapcraft.yaml`
- Create: `tests/snap_manifest.sh`
- Create: `.github/workflows/snap.yml`
- Modify: `README.md`

**Interfaces:**
- Produces a strictly confined `noor-notes_0.1.0_amd64.snap` and a tag-triggered CI artifact.

- [ ] Write a manifest contract test requiring `base: core24`, `confinement: strict`, GNOME desktop integration, Wayland/X11, network, desktop, and password-manager-service plugs; verify it fails.
- [ ] Add a Snapcraft manifest that builds the Cargo workspace and installs the binary, desktop file, AppStream XML, and scalable icon without the Shell extension.
- [ ] Add a GitHub Actions build using Canonical's supported Snap build action and artifact upload; document local installation and the Wayland pinning limitation.
- [ ] Install Snapcraft if needed, run lint/build, install the local Snap with `--dangerous`, and smoke-test its command; commit the green Snap unit.

### Task 3: Reproducible Flathub package

**Files:**
- Modify: `packaging/flatpak/io.github.saamaamr.NoorNotes.yml`
- Create: `packaging/flatpak/cargo-sources.json`
- Create: `tests/flatpak_manifest.sh`
- Create: `.github/workflows/flatpak.yml`
- Modify: `README.md`

**Interfaces:**
- Produces a Flathub-compatible manifest using an immutable release source and offline Cargo dependencies.

- [ ] Write a contract test rejecting `type: dir`, requiring an immutable source URL/commit, Cargo sources, minimal finish arguments, and installed metadata; verify it fails.
- [ ] Generate vendored Cargo source declarations from `Cargo.lock` and update the manifest to build offline against the supported GNOME runtime.
- [ ] Add a GitHub Actions Flatpak build and artifact workflow; document local Flatpak installation.
- [ ] Install Flatpak tooling/runtime if needed, run manifest validation, build, install into a user test repository, and smoke-test; commit the green Flatpak unit.

### Task 4: Release and submissions

**Files:**
- Create: `.github/workflows/release.yml`
- Modify: `README.md`

**Interfaces:**
- Produces GitHub release `v0.1.0`, a Flathub submission PR, and a Snap ready for store registration/upload after Ubuntu One authentication.

- [ ] Run Rust formatting, strict Clippy, full workspace tests, metadata validation, Snap validation/build, and Flatpak validation/build on the final tree.
- [ ] Add a release workflow that builds both artifacts for `v*` tags and attaches checksummed outputs without embedded credentials.
- [ ] Commit and push `main`, create signed/annotated tag `v0.1.0`, push it, and verify GitHub Actions artifacts.
- [ ] Fork the Flathub submission repository, add `io.github.saamaamr.NoorNotes`, run Flathub lint, and open a submission pull request.
- [ ] Stop only at Ubuntu One authentication; after the user signs in, register `noor-notes`, upload the Snap to `edge`, verify review status, and request stable release when approved.
