# Noor Notes UI/UX + Snow/Midnight — Proposed Roadmap

**Status:** Approved; first cohesive implementation pass implemented, tested and installed as Dev. Detailed verification: `docs/ui-polish-verification.md`. Unchecked acceptance items are not claimed complete.
**Goal:** Compact, responsive “Calm Notes + Powerful Editor”; existing feature ar data preserve.
**Architecture:** Existing Rust/GTK4/libadwaita widgets, integrated NotePreview, command handlers ar semantic palette reuse. Storage, RichDocument ar AutosaveQueue redesign-er scope-er baire.
**Scope:** Ei conversation-er combined UI/UX direction ar duita theme palette. Detailed implementation tasks protita phase-er current code audit-er por finalize hobe.

## Rules

- No rebuild from scratch, new CSS framework, fake command ba unnecessary dependency.
- Existing dirty-worktree changes preserve; Account & Sync-er already installed compact UI-ke baseline dhora.
- Snapshot/PDF purono hote pare: current source/build verify kore feature coverage ledger banano.
- Screenshot shudhu Noor Notes window/isolated fixture; full desktop capture/control noy.
- User-authored font size, text color, highlight ar content silently change noy.
- Snow/Midnight UI choices; existing saved theme preference compatibility preserve.
- Phase acceptance chara stable release noy; new publication/version change separately scoped.

## Shared palette

| Semantic role | Snow | Midnight |
|---|---|---|
| App | #F6F7F9 | #0F1724 |
| Sidebar | #F4F6F8 | #111A2A |
| Notes list | #F8F9FB | #121C2D |
| Editor | #FFFFFF | #0F1724 |
| Card/menu/popover | #FFFFFF | #172235 |
| Primary text | #1F2937 | #F1F5F9 |
| Secondary text/icon | #475467 | #CBD5E1 |
| Metadata | #667085 | #94A3B8 |
| Border | #E4E7EC | #26364D |
| Hover | #F1F3F5 | #1D2A40 |
| Accent | #4B69DC | #6D8BFF |
| Accent hover | #425FCC | #819AFF |
| Selected/active | #EEF2FF | #1D2A4A |
| Text selection | #C7D2FE | #334A7A |
| Danger | #DC2626 | #F87171 |
| Success | #15803D | #4ADE80 |
| Warning | #B45309 | #FBBF24 |

Primary button text Snow-e white, Midnight-e deep-dark; actual contrast verify kore final. Disabled controls muted, enabled controls visibly readable. Focus accent-derived ring. Note colors small rails; semantic syntax categories-er separate Day/Night mapping. Mathematical color inversion noy.

## Phase 0 — Current-state audit + close/reopen reliability

Files: `apps/noor-notes/src/managed_app.rs`, `main_window.rs`, `ui/account_settings.rs`, `ui/appearance_settings.rs`, `ui/writing_assistance_settings.rs`, `sticky_note_window.rs`.

- [ ] Current integrated feature/action inventory; unsupported ar legacy-only actions alada mark.
- [ ] Destroyed window-ke cached reference diye present korar warning reproduce; main/settings/sticky lifetime alada trace.
- [ ] Close → reopen → close, menu Quit ar pending-save behavior-er failing regression test.
- [ ] Root-cause-specific fix, no force-kill workaround; existing session/save behavior preserve.

Acceptance: repeatedly close/reopen-e destroyed-window warning na; quit-er por process exit; saved test note reload-e unchanged.

## Phase 1 — Semantic design system

Files: `apps/noor-notes/src/appearance/{palette,manager,style_runtime,preferences}.rs`, `apps/noor-notes/resources/design-system.css`, `ui/{toolbar_primitives,popover_primitives,dialog_primitives}.rs`.

- [ ] Hard-coded UI colors map kore shared palette-e consolidate.
- [ ] Compact spacing scale 4/8/12/16/24/32; control target 32–36px, font scaling/accessibility onujayi grow korte parbe.
- [ ] Utility classes reuse; normal/hover/focus/checked/disabled/destructive state consistent.
- [ ] Live theme switch-e visible menu/settings/sticky-o update; widget tree rebuild noy.

Tests: `theme_stylesheet`, `theme_contrast`, `popover_theme_contract`, `source_palettes`, `appearance_preferences`.
Acceptance: normal text 4.5:1, large text/essential control graphics 3:1 target; keyboard focus clearly visible; no dark-on-dark popover.

## Phase 2 — Shell + library

Files: `ui/{adaptive_layout,app_header,library_sidebar,note_collection,note_card,empty_state}.rs`, `main_window.rs`.

- [ ] Desktop starting ratio 10% sidebar / 18–20% list / remaining editor; content minimum ar user resize preferences respect.
- [ ] Medium/narrow window-e collapse/navigation transition; percentages blindly enforce noy.
- [ ] Compact header, soft sidebar state, aligned counts, readable note cards ar color rails.
- [ ] All Notes/Pinned/Favorites/Tags/Archived/Trash/Recent/search empty states.
- [ ] Search, sorting, note selection ar overflow actions keyboard-friendly.

Tests: `adaptive_layout`, `library_view`, `library_state`, `search`, `note_card_archive`, `library_performance`.
Acceptance: narrow window-e clipped action noy; selection/pin/favorite state both themes-e visible.

## Phase 3 — Integrated editor + formatting

Files: `ui/{note_preview,editor_header,editor_toolbar,editor_menu_bar,formatting_popover,rich_color_palette,editor_canvas,editor_status_bar}.rs`.

- [ ] Title editable/discoverable; title, metadata, toolbar ar body shared alignment.
- [ ] Default UI title 20–24px, body 14–16px starting target; user document formatting unchanged. Reading width roughly 65–80 characters, responsive available space.
- [ ] Compact grouped toolbar; current supported menu arrangement preserve, no automatic legacy menu restoration.
- [ ] Rich Text/Markdown/Plain Text/Code capability-aware controls; common commands share existing handlers.
- [ ] Selection, active formatting, undo/redo, emoji, list, colors, font size ar autosave regression checks.
- [ ] Edit/Done ar read-only sticky action clearly distinguishable.

Tests: `note_preview_edit`, `note_preview_responsive`, `toolbar_actions`, `editor_command_capabilities`, `formatting_activation`, `editor_history`, `rich_formatting_persistence`, `list_editing`, `editor_conversion`, `source_editor`.
Acceptance: visible action works; typing/formatting → save → reopen preserves content; unsupported action absent/disabled honestly.

## Phase 4 — Lifecycle, export, sticky, settings

Files: existing note lifecycle handlers, `sticky_note_window.rs`, `ui/{account_settings,appearance_settings,writing_assistance_settings,dialog_primitives}.rs`; locate current export dialog before editing.

- [ ] Archive/Trash/Restore/Permanent delete hierarchy; destructive action separated and confirmed where appropriate.
- [ ] Save As supported formats with truthful fidelity description: TXT plain, Markdown limited; HTML/DOCX/PDF actual output verify.
- [ ] Sticky top bar title + body only, compact pin, visible always-on-top state, close-state synchronization.
- [ ] Current compact Account & Sync refine across signed-out/locked/recovery/syncing/offline/error/backup states; auth behavior unchanged.
- [ ] Appearance ar Writing Assistance shared compact controls; recovery/status text never truncated into unusability.

Tests: `sticky_lifecycle`, `account_settings_ui`, `cloud_backup_ui`, `vault_onboarding`, `safe_export`, `export_docx`, `accessibility`, `shortcuts`.
Acceptance: each lifecycle reversible where supported; errors actionable; no invented backup/export capability.

## Phase 5 — Coverage, performance, handoff

- [ ] Real UI proof matrix: every current feature × applicable mode/theme/state; enabled and disabled controls; narrow/default/wide windows; scaled fonts.
- [ ] Current feature reference manifests reconcile; every entry verified / unsupported / blocked explicitly labelled.
- [ ] Compare startup, binary size, large-note switching/search against recorded baseline; no unsupported performance claim.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `xvfb-run -a cargo test --workspace` using approved host virtual display when sandbox GTK/loopback is unavailable.
- [ ] `cargo build --release -p noor-notes --features development` with existing public cloud build configuration preserved.
- [ ] `git diff --check`; read-only review; scoped Dev install with previous binary backup and checksum verification.
- [ ] README, current screenshots ar feature coverage update; no secrets or user note contents in proofs.

Delivery order: reliability → design tokens → shell/library → editor → remaining screens → full verification. Protita phase independently reviewable. Ei roadmap approval code commit/push ba Snap publication-er automatic instruction noy.
