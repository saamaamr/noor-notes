# Cohesive UI pass — 2026-09-06

Scope: approved Snow/Midnight design roadmap on the existing v1.1.3 source. No version bump, Store publication, database migration, new dependency, or change to document formats.

## Implementation map

| Area | Existing implementation reused | This pass |
|---|---|---|
| Shell/navigation | `ui/library_window.rs`, `adaptive_layout.rs`, `app_header.rs` | 1280 default width, three panes from 1200; existing ratio allocation and narrow navigation preserved |
| Shared styling | `appearance/palette.rs`, `resources/design-system.css` | Native color aliases, semantic checked switches, compact controls, document typography, readable cards |
| Integrated editor | `ui/note_preview.rs`, `editor_toolbar.rs`, `editor_menu_bar.rs` | Fit-content toolbar, mode-aware groups/separators and no empty menus; current Save As/Editor Mode/Format commands preserved; standalone-only Find cannot reappear on mode changes |
| Formatting | `ui/formatting_popover.rs`, `toolbar_primitives.rs` | Consistent sizing and compact scrollable popover; selection/command handlers unchanged |
| Settings | `account_settings.rs`, `writing_assistance_settings.rs`, `appearance_settings.rs` | Shared label/field/help primitive, non-stretched controls, wrapping status and action rows |
| Window lifetime | `managed_app.rs`, `window_lifecycle.rs` | Recreate removed main/settings windows; clone outside callbacks; respect close veto; shared main factory |
| Notes/storage | Existing `RichDocument`, `SqliteNoteRepository`, `AutosaveQueue` | No redesign of storage or user content |

Snow accent is slightly deeper than the original suggested blue so normal white button text meets 4.5:1. Success/warning text also meets 4.5:1. User-authored text colors, highlights, font sizes, source syntax and paper preferences are not overwritten by the UI redesign.

## Evidence

New/updated regression tests:

- `professional_controls`: real GTK toolbar sizing, formatting and emoji popover opening in both themes.
- `redesign_proof`: real MainWindow, integrated NotePreview, four modes and settings; synthetic temporary encrypted notes only.
- `window_lifecycle`: accepted/vetoed close, reentrant close, direct destroy, recreation; main action factory contract.
- `adaptive_layout`: updated breakpoint plus real shell sizing at desktop/narrow widths.
- `editor_menu_bar`: Code hides an empty Format menu; Rich Text restores it.
- `theme_contrast`: primary action, secondary/muted text, status text and native token coverage.
- Account/Writing settings tests: compact fields, accessible controls and action visibility.

To reproduce current visual proofs without capturing the desktop:

```bash
GDK_BACKEND=x11 GSK_RENDERER=cairo NOOR_REDESIGN_PROOF_DIR=/tmp/noor-redesign-proof \
  xvfb-run -a cargo test -p noor-notes --features development --test redesign_proof
```

The test saves 16 window/component renders: library, Rich Text, Markdown, Plain Text, Code, formatting, appearance and writing settings in Snow/Midnight. The editor-only window hosts the real integrated NotePreview, not the legacy NoteWindow. These are visual fixtures, not a claim of end-to-end cloud verification. Account proofs are generated separately by `NOOR_ACCOUNT_UI_PROOF_DIR` with `account_settings_ui`.

## Explicit limits

- Old 81-state PDF/JSON files were not present in the checked repository paths. Current source and test contracts were used; this is not a replacement 81-state coverage manifest.
- Real Google OAuth, two-device Supabase sync, external provider connectivity and live Snap runtime require their own acceptance checks.
- Scaled-font/compositor-specific visual coverage, exhaustive feature/theme screenshots and comparative startup/search benchmarks remain roadmap follow-ups. No claim of perfect or exhaustive verification.
- Installed Dev binary baseline: 13,666,296 bytes.

## Final verification and Dev handoff

- `cargo test --workspace`: 262 passing tests, zero failed/ignored; log `target/ui-polish-workspace-tests.log`.
- Development identity, real UI proof and toolbar tests: 8 passing checks; log `target/ui-polish-dev-tests.log`.
- `cargo fmt --all -- --check`, strict workspace/all-targets Clippy and `git diff --check`: pass.
- Release build with the existing public Supabase configuration and `development` feature: pass. No credentials/secrets added to source.
- Size gate: 13,677,528 bytes, below the repository's 15,000,000-byte limit; 11,232 bytes above the prior installed Dev build.
- Installed `/home/mamun/.local/bin/noor-notes-dev`, reporting `Noor Notes Dev 1.1.3`.
- Build/install SHA-256 match: `6054618c0295e6ac70c877f905ee1727c09a79ddc925852009b8eddeaa07d828`.
- Previous executable backup: `target/dev-before-redesign.fN6AxA/noor-notes-dev`.
- Fresh isolated X11/Cairo proofs: `target/ui-polish-proof/` (16 real window/component renders). No desktop capture or personal note content.
- No commit, push, version bump or Snap publication performed. No new Snap artifact was built, so the Snap runtime artifact contract is not claimed verified in this pass.

The GTK fixture waits for real frame-clock rendering; draining pending events alone was insufficient under build load. Production code was not changed to work around screenshot/display timing. The complete 81-state and external-service acceptance work above remains explicitly separate.

## Follow-up — editor margins and separate Read-only layout

- Editor margins now open in a bottom-positioned toolbar popover; Left, Right and Reset retain the existing session-only behavior and default editor gutters. The permanent slider row no longer consumes writing space.
- Separate Read-only windows hide the complete document-chrome container, eliminating the invisible heading/toolbar gap. The note title remains in the native header; body content stays top-aligned.
- Sticky insets scale with window width (3.3%, bounded to 12–24 logical pixels). These do not change the main editor margins or saved note preferences.
- Native surface layout events replace the ineffective widget `notify::width` observer, so resizing updates sticky spacing and reading width without polling or rebuilding widgets.
- Regression first reproduced the 56-pixel hidden-chrome top gap, then caught stale reading width on resize. Both pass after the targeted fixes.
- Fresh scoped verification: 12 passing checks across `sticky_note_window`, `sticky_lifecycle`, `preview_editor_surface`, `note_preview_responsive`, `note_preview_edit`, `professional_controls`, and `redesign_proof`. Log: `target/sticky-layout-tests.log`. Strict workspace/all-target Clippy, formatting and diff checks pass.
- Six current real sticky window renders: `target/sticky-layout-proof/`, Snow/Midnight at requested widths 320, 560 and 960; the test also resizes back to compact. Isolated X11/Cairo, synthetic text only. Xvfb reports missing DRI3 acceleration, not a test failure. No personal desktop capture.
- This follow-up does not claim a fresh full-workspace or Live Snap run; the earlier complete-workspace result above belongs to the preceding UI pass.
- Updated Dev release built and installed: `Noor Notes Dev 1.1.3`, 13,678,488 bytes (960 bytes above the preceding build; size gate passes). Build/install SHA-256: `cd289e008bfed413dea9f9f7a6817a0c001ae83953b9faf27a30a126a3414aae`. Backup: `target/dev-before-sticky-layout.idHcX7/noor-notes-dev`. No running personal window was closed; reopen Dev to load this executable. No commit, push or Store release.

## 1.1.4 release preparation — 2026-09-06

- Live Store baseline: stable 1.1.3/revision 19 and edge 1.1.3/revision 22. Requested patch increment is 1.1.4; not the monthly minor increment.
- Independent pre-release review found and corrected a Rust let-chain incompatible with the Snap compiler and loss of surviving sticky ownership on main-window recreation. Shared application-lifetime sticky state preserves the existing open-window behavior; callbacks follow the recreated library. The real GTK regression reproduced the old failure and passes for both Exit read-only and direct sticky close after reopening.
- CI's previous failure was a 1280-pixel Xvfb screen versus a requested 1480-pixel test window. CI now explicitly uses isolated X11/Cairo at 1920×1080; allocation assertions use actual content width rather than including CSD shadows.
- Final source-version workspace run: `target/release-1.1.4-workspace.log`; Dev CLI/identity/sticky run: `target/release-1.1.4-dev-tests.log`; both completed successfully. Strict all-target/all-feature Clippy, formatting, diff, Snap manifest, release metadata, cadence, automatic-versioning, security-workflow and Flatpak-manifest checks passed.
- Current dependency audit and deny gates passed with allowed warnings (including unmaintained paste, the lru panic-safety advisory, a yanked chacha20 version and duplicate dependency versions). These were not silently suppressed or represented as a warning-free audit. Logs: `target/release-audit.log`, `target/release-deny.log`.
- No staging public cloud configuration is being added to the Store workflow. Live OAuth/multi-device acceptance remains pending and is not claimed as part of this local-first UI release.
- Store publication completed successfully: [release run 34038250987](https://github.com/saamaamr/noor-notes/actions/runs/34038250987) passed build, runtime contract, Snapcraft lint, local installation smoke and Store-installed edge smoke, then promoted the same revision without rebuilding. Live `snap info noor-notes` confirmed **1.1.4/revision 23** on both stable and edge, **5.19 MB** compressed, on 2026-09-06.
- Released source: `5b5afc783bf9c0bae2d93beab74621da7ced6589`. [Security run 34038250593](https://github.com/saamaamr/noor-notes/actions/runs/34038250593) also passed. Final local coverage: **263 workspace tests and 5 Dev tests**, zero failures.
- Installed Dev executable reports `Noor Notes Dev 1.1.4`: **13,679,064 bytes**, below the 15,000,000-byte gate. Build/install SHA-256: `e424cf0235d402a1de716109a0a6fd006220a07d7bc4dacbf190ee503ac40969`. Previous Dev backup: `target/dev-before-1.1.4.QCtPE7/noor-notes-dev`. No running personal window was closed or personal Snap automatically refreshed.
