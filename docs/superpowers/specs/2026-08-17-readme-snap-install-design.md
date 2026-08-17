# README Snap Store Installation Design

**Date:** 2026-08-17

## Goal

Make the published Snap Store package the clearest and fastest installation path in the Noor Notes README while retaining the existing downloadable Snap, Flatpak, and source-install alternatives.

## Audience

Linux users who want to install, launch, update, inspect, or remove Noor Notes without building the application from source.

## README structure

Add an **Install from Snap Store** subsection at the beginning of the existing **Installation** section. Present `latest/edge` as the recommended current package source and clearly identify it as a preview channel rather than a stable release.

The subsection will provide these workflows:

1. Install and launch Noor Notes:

   ```bash
   sudo snap install noor-notes --edge
   noor-notes
   ```

2. Inspect the published package and installed revision:

   ```bash
   snap info noor-notes
   snap list noor-notes
   ```

3. Refresh an existing installation to the current edge revision:

   ```bash
   sudo snap refresh noor-notes --edge
   ```

4. Remove Noor Notes when requested:

   ```bash
   sudo snap remove noor-notes
   ```

Retain the existing **Release packages**, Flatpak, and source-install instructions as alternatives beneath the Store method.

## Accuracy updates

- Change the installation-method summary so it identifies the Snap Store package as the recommended packaged installation.
- Correct the Engineering highlights statement that currently avoids claiming Store availability.
- Replace the stale release-automation statement that says Snap Store publication is unavailable. State that version tags build release artifacts, Store publication is currently a manual owner action, and the latest published Snap is available from `latest/edge`.
- Link to `https://snapcraft.io/noor-notes` near the Store installation instructions.
- Do not claim a `stable`, `candidate`, or `beta` channel release.

## Scope boundaries

- Do not change application code, packaging configuration, release automation, or stored data behavior.
- Do not remove the downloadable Snap, Flatpak, source installation, checksum verification, Xpad sandbox limitation, or troubleshooting guidance.
- Do not duplicate a second installation section near the top of the README.

## Verification

- Confirm every Snap command is syntactically valid.
- Confirm the Store reports Noor Notes `0.1.1`, revision `2`, on `latest/edge`.
- Search the final README for stale claims that the Snap Store package is unavailable.
- Run Markdown-focused structural checks available in the repository and `git diff --check`.
- Inspect the final diff to ensure only the approved README documentation and its design/plan artifacts changed.
