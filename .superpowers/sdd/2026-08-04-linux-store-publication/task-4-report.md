# Task 4 report — Release and submissions

Status: PENDING_HOSTED_RC_VALIDATION

## Release candidate

- Candidate orchestration commit: `69fe55c741c99966ef24d722ea5c63e0767afa96`
- Annotated candidate tag: `v0.1.0-rc.5`
- Consolidated hosted validation: [run 30907910334](https://github.com/saamaamr/noor-notes/actions/runs/30907910334)

The candidate tag starts only `.github/workflows/release.yml`. Its Snap and
Flatpak jobs build, lint/install/smoke-test, and retain their artifacts. The
GitHub-release job is deliberately limited to the exact final `v0.1.0` tag, so
the candidate cannot publish a release.

At this report's checkpoint the RC run is still in progress: the Snap build is
running and the Flatpak container is initializing. It is not final-release
evidence. Do not create or push `v0.1.0` until both jobs complete successfully.

The earlier `v0.1.0-rc.4` validation is superseded because it preceded the
permission and workflow-consolidation fix. Its standalone Flatpak validation
run [30907660196](https://github.com/saamaamr/noor-notes/actions/runs/30907660196)
completed successfully; the redundant in-progress RC.4 jobs were cancelled.

## Safe source strategy

The Flatpak manifest pins immutable payload commit
`37c1ddd55867e8c520b0596b9c9c09fb7250b12b`. That commit is an ancestor of the
orchestration candidate and its application/package inputs are unchanged
through `69fe55c`. The following orchestration commit can therefore pin the
already-existing payload without a self-reference; `v0.1.0-rc.5` tests that
exact orchestration commit and its pinned payload.

When the RC is green, create an **annotated** `v0.1.0` tag at
`69fe55c741c99966ef24d722ea5c63e0767afa96` and push it. This starts the same
fresh package builds and then creates the GitHub release with the `.snap`,
`.flatpak`, and `SHA256SUMS.txt` assets. A signed tag cannot be created from
the current environment: no GPG signing key is configured or available. Do not
claim a signed tag or fabricate one.

## Local evidence

The final candidate tree completed these commands successfully before the RC
tag was pushed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
xvfb-run -a cargo test -p noor-windowing
gjs -m extensions/gnome/tests/test-policy.js
bash tests/e2e/two_device_sync.sh
tests/store_metadata.sh
appstreamcli validate --no-net data/io.github.saamaamr.NoorNotes.metainfo.xml
desktop-file-validate data/io.github.saamaamr.NoorNotes.desktop
tests/snap_manifest.sh
tests/snap_workflow.sh
tests/flatpak_manifest.sh
tests/flatpak_workflow.sh
tests/release_workflow.sh
sh -n tests/store_metadata.sh tests/snap_manifest.sh tests/snap_workflow.sh tests/flatpak_manifest.sh tests/flatpak_workflow.sh tests/release_workflow.sh
npx --yes prettier@3.5.3 --check README.md .github/workflows/snap.yml .github/workflows/flatpak.yml .github/workflows/release.yml snap/snapcraft.yaml packaging/flatpak/io.github.saamaamr.NoorNotes.yml
git diff --check
```

This host has `appstreamcli` and `desktop-file-validate`, but no local
`snapcraft`, `flatpak`, or `flatpak-builder`; hosted RC validation is the
package build/install/smoke gate.

## Submission boundaries

### Flathub — blocked, no PR attempted

Flathub's current generative-AI policy prohibits AI-assisted application
content and AI-generated submission pull requests. This agentic workstream is
therefore ineligible for an agent-created Flathub submission. No fork,
manifest submission, lint submission, or pull request was created. The owner
must first establish compliant application provenance and make any eligible
human-led submission. See `task-3-flathub-policy-blocker.md`.

### Snap Store — human authentication required

No Ubuntu One/Snapcraft credentials were used or invented, and no Snap Store
name registration or upload was attempted. After the owner creates or signs in
to Ubuntu One, accepts the Snap Store terms, and authorizes `noor-notes`, the
next human step is to upload the verified release `.snap` to `edge`, wait for
review, then request stable promotion if Canonical approves it.

## Final-release checklist

- [x] Pushed release orchestration commits and annotated RC tags.
- [x] Completed local Rust, metadata, workflow-contract, formatting, syntax,
      and diff checks.
- [ ] Verify run `30907910334` has successful Snap and Flatpak jobs.
- [ ] Create/push annotated `v0.1.0` at `69fe55c` (signing key unavailable).
- [ ] Verify final tag workflow artifacts and GitHub release checksums.
- [x] Do not create a Flathub PR; policy blocker documented.
- [x] Stop before Snapcraft authentication; exact human next step documented.
