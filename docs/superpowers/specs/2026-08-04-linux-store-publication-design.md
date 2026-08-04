# Noor Notes Linux Store Publication Design

## Goal

Prepare, validate, and submit Noor Notes to both Ubuntu App Center through the Snap Store and Linux software centers through Flathub.

## Release Identity

- Application name: Noor Notes
- Application ID: `io.github.saamaamr.NoorNotes`
- Snap name target: `noor-notes`
- Source repository: `https://github.com/saamaamr/noor-notes`
- License: `GPL-3.0-or-later`
- Initial store release: `0.1.0`

The release uses one tagged source revision and builds both packages from that immutable revision.

## Shared Store Metadata

Complete the AppStream record with developer identity, homepage, source, issue tracker, release notes, launchable desktop ID, content rating, and screenshot references. Validate the desktop file, icon, and AppStream XML. Add polished screenshots that accurately show the note editor, library, rich formatting, and Trash recovery controls.

## Snap Store Package

Add `snap/snapcraft.yaml` using strict confinement and the GNOME extension appropriate for the current GTK4/Libadwaita stack. Declare only required interfaces for Wayland/X11 display, desktop integration, network sync, and Secret Service access. The Snap must store mutable data under supported user data directories and must not attempt to install files outside confinement.

Build and smoke-test the Snap locally. Add a GitHub Actions workflow that builds the Snap on tagged releases and preserves it as an artifact. Publishing requires the user to create a free Ubuntu One account, accept Snap Store terms, authenticate Snapcraft, and authorize registration of the `noor-notes` name. Store review remains controlled by Canonical.

## Flathub Package

Replace the development-only local-directory source with an immutable Git tag/archive and vendored Cargo dependency sources suitable for an offline Flatpak build. Use the current supported GNOME runtime, the minimum required sandbox permissions, and validated AppStream metadata.

Build and smoke-test with `flatpak-builder`. Add a manifest validation workflow. Submit the final manifest to Flathub through its GitHub submission process using the already authenticated GitHub account. Flathub review and acceptance remain controlled by Flathub maintainers.

## GNOME Extension Limitation

Store-sandboxed packages do not install the bundled GNOME Shell extension automatically. Editing, local storage, import, rich text, Trash actions, and supported window behavior remain available. On GNOME Wayland, Always on Top may remain disabled unless the extension is installed separately outside the store package. Store descriptions must state this accurately.

## Security and Privacy

- Keep strict Snap confinement and Flatpak sandboxing.
- Request no filesystem-wide access.
- Request network access only for optional synchronization.
- Use the desktop Secret Service/keyring interface for credentials.
- Do not include account credentials, signing secrets, or store tokens in Git.

## Validation and Delivery

- Run Rust formatting, strict Clippy, complete tests, desktop-file validation, AppStream validation, Snap lint/build checks, and Flatpak build checks.
- Install and smoke-test locally built packages when the host tooling permits.
- Create and push the `v0.1.0` Git tag only after both manifests reference the final tested commit.
- Submit Flathub through a GitHub pull request.
- Stop at the Snapcraft authentication boundary so the user can create and authorize their Ubuntu One account, then resume registration and upload.

## External Boundaries

Store accounts, legal acceptance, name approval, automated review, manual review, and final listing publication cannot be bypassed or guaranteed. The project can be made submission-ready and submitted, but each store decides acceptance and publication timing.
