# Midnight Theme Consistency Design

## Problem

Noor Notes currently defines Snow values in the global `@nn_*` semantic aliases and repairs Midnight with component-specific `.nn-theme-midnight` selectors. Explicit Snow foregrounds on nested labels and buttons therefore override inherited Midnight foregrounds, while GTK popovers and dropdowns can render outside the window CSS subtree.

## Approved Architecture

Create one display-level theme stylesheet from an active semantic palette plus the shared Atomic GTK component stylesheet. Snow and Midnight define the same `@nn_*` names. Theme switching reloads the existing CSS provider and never rebuilds widget trees.

`AppearanceManager` owns the active style runtime, persists the selected mode, applies libadwaita light/dark preference, updates all registered window marker classes, reloads the semantic stylesheet once, and then notifies editor/source-palette listeners.

All popovers use a shared primitive and all GTK popover/dropdown/menu nodes receive semantic surface, foreground, border, hover, active, focus, and disabled styling. Root theme classes remain only as stable identity markers and for truly theme-specific data swatches such as rich-text color samples and sticky paper colors.

## Semantic Contract

Both palettes define app, sidebar, note-list, editor, surface, elevated surface, popover, modal, input, hover, active, selected, primary/secondary/muted/disabled/inverse text, border/subtle/strong border, accent/hover/soft/strong accent, focus, success, warning, danger, error, info, scrollbar, editor selection background, and editor selection foreground.

Snow uses the approved bright neutral palette. Midnight uses deep blue-black surfaces, light neutral text, and the approved blue accent. Component CSS may reference semantic names only; reusable component rules may not reference Snow- or Midnight-prefixed colors.

## Coverage

The change covers the integrated MainWindow/NotePreview path, editor toolbar and menus, formatting/font-size/emoji/more popovers, application/sort/note-action menus, Writing Assistance popovers, dialogs, settings, sticky windows, status bar, source editors, text selection, and all interaction states.

The existing Rich Text color/highlight choices and note identity rails remain data colors. Their Snow/Midnight display variants remain explicit because they represent user content rather than application chrome.

## Theme Contrast Test

Development builds expose a `Theme Contrast Test` application action. It cycles the real running application Snow → Midnight → Snow through `AppearanceManager`; production builds do not expose it. Automated tests validate semantic token contrast and that every application-owned popover uses the shared primitive. GTK integration tests remain separate from pure tests so a sandbox display failure is reported as an environment limitation rather than hidden by production changes.

## Constraints

- Preserve all existing features, note data, persistence, editor modes, shortcuts, sticky behavior, and source syntax highlighting.
- Add no dependency and no external CSS framework.
- Keep the existing curated Atomic GTK CSS approach.
- Do not perform database or filesystem work during style switching.
- Prevent preference/theme notification re-entry and redundant stylesheet reloads.
