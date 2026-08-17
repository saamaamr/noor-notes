# README Stable Snap Installation Design

## Goal

Update the Noor Notes README to reflect the live Snap Store channel map after version 0.1.1 revision 2 was promoted to `latest/stable`. Normal users should receive the stable installation path first, while testers can still opt into `latest/edge` deliberately.

## Scope

Change only the README text that describes Snap Store installation, channel selection, refreshing, and current Store publication status. Preserve the existing downloaded Snap, Flatpak, Ubuntu source, local build, verification, removal, feature, and architecture documentation.

## Content design

- Describe the Snap Store as the recommended stable packaged installation.
- Use `sudo snap install noor-notes` as the primary installation command.
- Retain `snap info noor-notes` and `snap list noor-notes` as verification commands.
- Use `sudo snap refresh noor-notes --stable` for explicit stable refreshes.
- Add a compact optional preview-channel subsection containing:
  - `sudo snap refresh noor-notes --edge` to opt into edge builds.
  - `sudo snap refresh noor-notes --stable` to return to stable.
- Explain briefly that edge builds may change more frequently and are intended for testing.
- Update the Store-status paragraph to state that version 0.1.1 revision 2 is published on both `latest/stable` and `latest/edge` for amd64.
- Keep the Snap Store listing link and removal command.

## Error prevention

- Do not claim that candidate or beta contains an independently published revision.
- Do not describe edge as the required installation channel.
- Do not alter local package filenames or their `--dangerous` installation instructions.
- Do not commit locally built `.snap` artifacts.

## Verification

- Query `snapcraft status noor-notes` and `snap info noor-notes` to confirm the live channel map.
- Confirm the stable install command appears as the primary Store command.
- Confirm edge is described only as an optional preview/testing path.
- Check Markdown fences and run `git diff --check`.
- Inspect the final diff for unrelated README changes.

