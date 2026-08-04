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

## Fix round 1

- Added the canonical GPL-3.0 license text as `LICENSE` and explicitly installs
  it to `/app/share/licenses/io.github.saamaamr.NoorNotes/noor-notes/`
  `GPL-3.0-or-later.txt`.
- Extended the manifest contract to require GNOME 50, the Rust SDK extension,
  the Rust extension path, `CARGO_HOME`, frozen offline Cargo, the canonical
  license source/install command, and exact Cargo source reconciliation.
  The test now compares every registry package and checksum in `Cargo.lock`
  with its generated archive and `.cargo-checksum.json` entry; it rejects
  missing, extra, duplicate, or mismatched entries.
- Extended the hosted workflow to export the bundle repository, install it in
  an isolated user Flatpak installation, verify the installed GPL path and
  `/usr/bin/secret-tool`, run `noor-notes --help`, and upload the bundle.

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
`flatpak-builder --show-manifest` parsed and expanded this manifest to 671
module sources, with the expected app ID, pinned commit, and exact permission
set.

## Hosted build, installation, and smoke evidence

GitHub Actions run `30906111940` completed successfully in 4m47s:

- Flatpak contracts passed.
- The GNOME 50 container installed the Rust 25.08 SDK extension, fetched all
  declared sources, ran `cargo build --frozen --offline --release --package
  noor-notes`, created the bundle, and uploaded it.
- The workflow added the generated repository to a separate user Flatpak
  installation, installed `io.github.saamaamr.NoorNotes//master`, and ran
  `flatpak --user run --command=noor-notes io.github.saamaamr.NoorNotes
  --help` successfully.
- Uploaded artifact: `8891219265`, `noor-notes-x86_64.flatpak`, 2,130,732
  bytes, available from the successful run and not expired at validation time.

Follow-up GitHub Actions run `30906525804` also completed successfully. Its
strengthened installed-package smoke asserted the deployed GPL file at
`/app/share/licenses/io.github.saamaamr.NoorNotes/noor-notes/`
`GPL-3.0-or-later.txt` and `/usr/bin/secret-tool` before running the packaged
`noor-notes --help` command.

The workflow was temporarily enabled for the package branch only because
GitHub does not offer workflow dispatch for a file absent from the default
branch. It was restored to its intended tag/manual-only trigger immediately
after the run; no release tag, GitHub release, store upload, or Flathub
submission was created.

## Remaining boundaries

- The local host still lacks disk space for the Rust SDK extension, but the
  complete hosted build/install/smoke gate above succeeded.
- The prior `secret-tool` concern was incorrect. The GNOME 50 runtime contains
  `/usr/bin/secret-tool`; the installed-package workflow now asserts that path.
  The app's actual Secret Service write remains an integration behavior for a
  desktop session with a configured keyring, rather than a safe CI smoke test.
- Flathub's current generative-AI policy is an external eligibility blocker for
  this agentic workstream. See `task-3-flathub-policy-blocker.md`; no agent may
  generate or open a Flathub submission PR from this work.
- The manifest intentionally pins the existing release-candidate commit. Task
  4 must update and revalidate it once the final `v0.1.0` release commit is
  created.

No credentials, signing keys, store tokens, uploads, pushes, or submissions
were used.
