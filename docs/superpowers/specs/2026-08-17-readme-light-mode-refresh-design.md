# README Light Mode Refresh Design

## Goal

Update the GitHub-facing Noor Notes README so its product overview accurately represents the completed Light Mode redesign. The update should be concise, professional, technically honest, and visually current without regenerating the unrelated full screenshot gallery.

## Scope

The README update will:

- replace the primary library screenshot with a fresh Light Mode capture from the current code;
- replace the primary dark-appearance screenshot with a fresh Graphite capture from the current code;
- add concise product-overview copy describing the clearer pane hierarchy, restrained note colours, compact controls, readable preview, and corrected responsive behavior;
- state that Rich Text keeps its existing 5-pixel top/bottom and 8-pixel left/right writing margins;
- update the screenshot gallery introduction to distinguish the two refreshed overview images from the broader gallery captured on 9 August 2026;
- preserve all existing installation, privacy, security, release, limitation, and contribution documentation.

The update will not regenerate the full 98-page gallery, rewrite unrelated README sections, change application behavior, modify package artifacts, or access personal notes.

## Screenshot Method

Use a temporary GTK review harness with synthetic notes and an isolated temporary appearance file. The harness must load the production stylesheet and real library components, capture Light and Graphite at the gallery's established 1248 × 702 canvas, and never open the normal Noor Notes database.

Write the refreshed images to the existing stable paths:

- `data/screenshots/noor-notes-library.png`
- `data/screenshots/noor-notes-dark.png`

The captures must demonstrate:

- subtle Light Mode pane separation;
- white or faintly tinted cards with retained colour rails;
- calm selected states;
- readable long content wrapping;
- clear Graphite text and control contrast.

The temporary harness, temporary appearance data, logs, and intermediate captures must be removed before committing.

## README Presentation

Keep the existing Product overview tables and image destinations so external links remain stable. Add a short paragraph immediately after the overview introduction explaining the refreshed Light Mode and adaptive library experience. Avoid marketing claims that cannot be verified by the repository.

Update the existing appearance feature bullet rather than adding a duplicate feature. The README should remain scannable and retain its privacy-first, offline-first positioning.

## Verification

Before pushing:

- inspect both new images at their original resolution;
- confirm each image is exactly 1248 × 702 and contains only synthetic data;
- run the repository screenshot-gallery contract;
- run Markdown/link-oriented repository checks that already exist;
- run `git diff --check` and inspect the final diff for unrelated changes;
- confirm only intended README, gallery-note, screenshot, design, and plan files are committed;
- leave the two pre-existing untracked Snap packages untouched;
- push `main` only after the final local commit and verify local `HEAD` equals `origin/main`.
