# README Release Refresh Design

## Goal

Make the Noor Notes README the complete, accurate entry point for users and contributors after the v0.1.0 Linux package release.

## Scope

The README will describe the released application features, supported installation methods, artifact checksum verification, sandbox behavior, Always on Top limitations, encrypted synchronization, Xpad import, data recovery, development verification, licensing, and current Store publication status.

It will link directly to the v0.1.0 GitHub release and use commands that match the actual artifact names and repository scripts. It will not claim that Noor Notes is available in the Snap Store or Flathub.

## Structure

The document will lead with the release and primary features, followed by installation choices. Detailed operational information will follow in sections for sandbox/window behavior, synchronization and recovery, development, release automation, Store status, and license.

## Verification

README claims will be checked against the release assets, package manifests, workflow files, installer scripts, application metadata, and existing automated contract tests. Markdown links and commands will be inspected for consistency, and the repository test suite relevant to documentation and packaging will be run before committing.
