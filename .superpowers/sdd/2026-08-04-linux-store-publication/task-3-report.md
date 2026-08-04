# Task 3 report — Reproducible Flathub package

Status: DONE_WITH_CONCERNS

## Implementation commit

- `0db9884798982cff65c888c9b2261a4bb34ba353` — reproducible Flatpak
  manifest, generated Cargo sources, CI workflow, documentation, and contracts.

## Delivered

- Replaced the development-only `type: dir` source with the public Noor Notes
  Git source pinned to the immutable `v0.1.0-rc.3` commit
  `39868d1a12ba4d2be9df8e7b67ef98a7599deba8`.
- Generated `packaging/flatpak/cargo-sources.json` from the unchanged release
  `Cargo.lock`: 334 checksummed crate archives, their checksums, and the
  vendored Cargo configuration. Cargo builds with `--frozen --offline` and
  `CARGO_HOME=/run/build/noor-notes/cargo`.
- Uses the supported GNOME 50 runtime and its Rust SDK extension. The only
  static runtime permissions are IPC, optional sync networking, Wayland/X11
  display, and Secret Service access. No filesystem-wide access or GNOME Shell
  extension is included.
- Added a tag/manual GitHub Actions build using the Flathub GNOME 50 builder
  image and `flatpak-builder@v6`, which uploads `noor-notes.flatpak` as an
  artifact without credentials or publication actions.
- Documented user-scoped local installation in `README.md`.

## TDD evidence

- The new manifest contract initially failed against the existing manifest:
  `finish-args must equal ...; got ... org.freedesktop.Notifications`.
- The workflow contract initially failed before the workflow was added:
  `Missing Flatpak workflow: .../.github/workflows/flatpak.yml`.
- After the implementation, a mutation that restored `type: dir` produced the
  expected failure: `Flatpak sources must not use the development-only type:
  dir source`. The immutable Git source was restored immediately afterward.

## Fresh validation

```text
tests/flatpak_manifest.sh                                            PASS
tests/flatpak_workflow.sh                                            PASS
sh -n tests/flatpak_manifest.sh tests/flatpak_workflow.sh            PASS
npx --yes prettier@3.5.3 --check README.md packaging/flatpak/io.github.saamaamr.NoorNotes.yml .github/workflows/flatpak.yml
                                                                    PASS
cargo build --locked --offline --release --package noor-notes        PASS
git diff --check                                                     PASS
```

Used Flathub's official `flatpak-cargo-generator.py` to regenerate the Cargo
manifest into a temporary file, then `cmp` verified it is byte-for-byte
identical to the committed `cargo-sources.json`. `git diff` also confirmed that
`Cargo.lock` and `Cargo.toml` are unchanged from the pinned source commit.

The official Ubuntu Flatpak 1.16.6 and flatpak-builder 1.4.8 packages were
downloaded and extracted only into `/tmp` (no system package installation).
`flatpak-builder --show-manifest` parsed and expanded this manifest to 670
module sources, with the expected app ID, pinned commit, and exact permission
set.

## Concerns / external boundary

- A full local Flatpak build, user-repository install, and smoke test could
  not run on this host. The isolated GNOME SDK installed successfully, but the
  required `org.freedesktop.Sdk.Extension.rust-stable//25.08` download failed:
  `Delta requires 1.7 GB free space, but only 1.7 GB available`. The CI
  workflow is therefore the next environment to perform the complete package
  build and smoke test.
- The inspected GNOME 50 SDK does not contain the `secret-tool` executable.
  Noor Notes currently invokes that executable for its optional credential
  store, so the packaged sync credential path needs runtime smoke testing (or
  a later application-level Secret Service implementation) before store
  submission.
- The repository currently has no `LICENSE` or `COPYING` file even though the
  Cargo metadata declares GPL-3.0-or-later. Current Flathub requirements call
  for installed license files; add the authoritative license text before a
  submission.
- The manifest intentionally pins the existing release-candidate commit. Task
  4 must update and revalidate it once the final `v0.1.0` release commit is
  created.

No credentials, signing keys, store tokens, uploads, pushes, or submissions
were used.
