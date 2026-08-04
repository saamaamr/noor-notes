# Task 4 report — Release and submissions

Status: FIX_ROUND_1_PENDING_HOSTED_RC_VALIDATION

## Release candidate

- Initial candidate `69fe55c741c99966ef24d722ea5c63e0767afa96` passed its
  consolidated hosted validation in
  [run 30907910334](https://github.com/saamaamr/noor-notes/actions/runs/30907910334).
- Its independent review identified mutable action/container references, so it
  is not eligible for final tagging.
- Fix-round candidate: `32062e467e9999e4702b3ebfeb07145c75ac0da7`
- Annotated fix-round tag: `v0.1.0-rc.6`
- Consolidated hosted validation: [run 30909555115](https://github.com/saamaamr/noor-notes/actions/runs/30909555115)

The candidate tag starts only `.github/workflows/release.yml`. Its Snap and
Flatpak jobs build, lint/install/smoke-test, and retain their artifacts. The
GitHub-release job is deliberately limited to the exact final `v0.1.0` tag, so
the candidate cannot publish a release.

At this report's checkpoint the fix-round RC run is still in progress. It is
not final-release evidence. Do not create or push `v0.1.0` until both jobs
complete successfully and the candidate is independently re-reviewed.

Fix round 1 pins every GitHub Action to a reviewed full commit SHA with a
version comment. It also pins the privileged GNOME 50 Flatpak builder container
to `sha256:ab91c589e30298efc3bca549141aa1672a250fefa57d50e11300276f2dfc558f`.
The release-workflow contracts reject a mutable action reference, an
unreviewed SHA, a missing version comment, or any other Flatpak container
image. `softprops/action-gh-release` is pinned to
`3bb12739c298aeb8a4eeaf626c5b8d85266b0e65` (`v2.6.2`), and remains isolated in
the only job granted `contents: write`.

The earlier `v0.1.0-rc.4` validation is superseded because it preceded the
permission and workflow-consolidation fix. Its standalone Flatpak validation
run [30907660196](https://github.com/saamaamr/noor-notes/actions/runs/30907660196)
completed successfully; the redundant in-progress RC.4 jobs were cancelled.

## Safe source strategy

The Flatpak manifest pins immutable payload commit
`37c1ddd55867e8c520b0596b9c9c09fb7250b12b`. That commit is an ancestor of the
orchestration candidate and its application/package inputs are unchanged
through the release-orchestration commits. The following orchestration commit
can therefore pin the already-existing payload without a self-reference;
`v0.1.0-rc.6` tests the fix-round orchestration commit and its pinned payload.

The remote default branch is merge commit
`6941e56c1923e89c558680c852bba8df47160ab5`, not an ancestor of the candidate,
so it cannot be fast-forwarded. Its second parent is the common base
`e9c8402384a2f9512a6a2809bfde3e7b41691535` and its tree is byte-for-byte equal
to that base. After the fix-round RC is green and approved, but before final
tagging, use this exact non-destructive procedure to merge the remote-default
history while proving the source tree is unchanged:

```sh
git fetch --prune origin
candidate=32062e467e9999e4702b3ebfeb07145c75ac0da7
base=$(git merge-base origin/main "$candidate")
test "$base" = e9c8402384a2f9512a6a2809bfde3e7b41691535
git diff --quiet "$base" origin/main
git switch --detach "$candidate"
git merge --no-ff --no-edit origin/main
merged=$(git rev-parse HEAD)
test "$(git rev-parse "$merged^{tree}")" = "$(git rev-parse "$candidate^{tree}")"
git push origin "$merged":refs/heads/main
git fetch origin main
test "$(git rev-parse origin/main)" = "$merged"
git ls-tree -r --name-only origin/main -- .github/workflows/release.yml data/screenshots/noor-notes-editor.png data/screenshots/noor-notes-library.png
```

Only after this evidence, the final-tag approval, and any required replacement
RC for the merge commit may an **annotated** `v0.1.0` tag be created. A signed
tag cannot be created from the current environment: no GPG signing key is
configured or available. Do not claim a signed tag or fabricate one.

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
- [x] Verify run `30907910334` has successful Snap and Flatpak jobs.
- [ ] Verify fix-round run `30909555115` has successful Snap and Flatpak jobs.
- [ ] Obtain an independent re-review of the immutable dependency pins.
- [ ] Merge the reviewed candidate with `origin/main` using the exact
      tree-equality procedure above; do not fast-forward-push it.
- [ ] Create/push annotated `v0.1.0` at the separately approved final merge
      commit (signing key unavailable).
- [ ] Verify final tag workflow artifacts and GitHub release checksums.
- [x] Do not create a Flathub PR; policy blocker documented.
- [x] Stop before Snapcraft authentication; exact human next step documented.
