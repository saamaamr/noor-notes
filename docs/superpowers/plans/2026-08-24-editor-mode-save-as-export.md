# Integrated Editor Mode, Save As, and Responsive Editing Plan

**Goal:** Deliver working formatting entry points, real mode conversion, native five-format export, and ratio-based responsive editing without changing note persistence.

## Completed implementation sequence

- [x] Extend the safe filename/extension contract for DOCX, PDF, HTML, TXT, and Markdown.
- [x] Normalize the live note into one immutable export document.
- [x] Implement and structurally test TXT, Markdown, HTML, DOCX, and paginated Unicode PDF renderers.
- [x] Reproduce the real GTK formatting-popup dismissal and keep its content usable in a bounded scroller.
- [x] Replace integrated Edit/Insert menu groups with Save As and Editor Mode while retaining Format.
- [x] Route all four modes through the existing conversion confirmation, recovery, autosave, and persistence pipeline.
- [x] Show the active editor mode with visual and accessible selected state.
- [x] Implement one async Save As orchestrator with format filter, extension enforcement, worker-thread rendering, Gio replacement, owner-only permission, cancellation, busy guard, and error UI.
- [x] Wire all five exports in both integrated and standalone editor surfaces.
- [x] Replace fixed editor sizing during live use with 100%/92%/78% pane ratios and compact/narrow chrome states.
- [x] Add focused export, formatting, mode, and responsive GTK tests.
- [x] Run the full workspace suite under Xvfb, strict Clippy, formatting, and diff checks.
- [x] Build an optimized release and compare binary size against the previous build.

## Delivery checklist

- [x] Install/rebuild the Noor Notes Dev application from this verified worktree.
- [x] Smoke-check startup using the development identity.
- [ ] Stage only implementation, tests, Cargo metadata, and these design records.
- [ ] Merge to `main`, push `origin/main`, and verify local/remote commit equality.
