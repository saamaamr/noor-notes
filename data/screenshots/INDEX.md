# Noor Notes screenshot gallery

Most of this comprehensive visual reference was captured from the real GTK4/libadwaita interface on 9 August 2026. The five-image Store set was finalized for v1.0.0 on 19 August 2026: Library, Editor, Graphite, and Responsive were recaptured from the current interface, while the unchanged Writing Assistance view was retained from 17 August. Every Store capture used an isolated temporary SQLCipher database with synthetic notes; no personal note database was opened.

All individual screenshots and gallery overviews use a stable 1248 × 702 RGB canvas. “Compact” documents the smallest visible non-maximized window; a truly minimized window cannot be screenshotted because it is not rendered.

Download or print the complete 98-page collection as [noor-notes-complete-gallery.pdf](noor-notes-complete-gallery.pdf).

## Stable product overview

- [noor notes library](noor-notes-library.png)
- [noor notes editor](noor-notes-editor.png)
- [noor notes writing assistance](noor-notes-writing-assistance.png)
- [noor notes dark](noor-notes-dark.png)
- [noor notes formatting](noor-notes-formatting.png)
- [noor notes find replace](noor-notes-find-replace.png)
- [noor notes trash](noor-notes-trash.png)
- [noor notes responsive](noor-notes-responsive.png)

## Library

- [active card archive action](library/active-card-archive-action.png)
- [active card menu](library/active-card-menu.png)
- [archived card menu](library/archived-card-menu.png)
- [archived](library/archived.png)
- [compact all notes](library/compact-all-notes.png)
- [empty archive](library/empty-archive.png)
- [empty pinned](library/empty-pinned.png)
- [empty trash](library/empty-trash.png)
- [favorites](library/favorites.png)
- [maximized all notes](library/maximized-all-notes.png)
- [permanent delete confirmation](library/permanent-delete-confirmation.png)
- [pinned](library/pinned.png)
- [recent](library/recent.png)
- [restored all notes](library/restored-all-notes.png)
- [search no results](library/search-no-results.png)
- [search results](library/search-results.png)
- [selected card preview](library/selected-card-preview.png)
- [sort menu](library/sort-menu.png)
- [sort updated](library/sort-updated.png)
- [tags](library/tags.png)
- [trash card menu](library/trash-card-menu.png)
- [trash restore selected](library/trash-restore-selected.png)
- [trash](library/trash.png)

## Editor

- [compact rich editor](editor/compact-rich-editor.png)
- [export menu](editor/export-menu.png)
- [go to line dialog](editor/go-to-line-dialog.png)
- [header archive delete](editor/header-archive-delete.png)
- [maximized rich editor](editor/maximized-rich-editor.png)
- [mode menu](editor/mode-menu.png)
- [narrow toolbar wrap](editor/narrow-toolbar-wrap.png)
- [note colour menu](editor/note-colour-menu.png)
- [restored rich editor](editor/restored-rich-editor.png)
- [saved status](editor/saved-status.png)
- [short multicolumn more](editor/short-multicolumn-more.png)
- [undo redo enabled](editor/undo-redo-enabled.png)
- [view options menu](editor/view-options-menu.png)
- [zoom 110 percent](editor/zoom-110-percent.png)

## Formatting

- [alignment controls](formatting/alignment-controls.png)
- [bold italic selected](formatting/bold-italic-selected.png)
- [bullet list](formatting/bullet-list.png)
- [clear formatting](formatting/clear-formatting.png)
- [custom font size](formatting/custom-font-size.png)
- [font size presets](formatting/font-size-presets.png)
- [highlight presets](formatting/highlight-presets.png)
- [numbered list](formatting/numbered-list.png)
- [popover overview](formatting/popover-overview.png)
- [text colour presets](formatting/text-colour-presets.png)
- [underline strikethrough](formatting/underline-strikethrough.png)

## Editor modes

- [code rust](modes/code-rust.png)
- [long source](modes/long-source.png)
- [markdown](modes/markdown.png)
- [plain text](modes/plain-text.png)
- [rich to markdown confirmation](modes/rich-to-markdown-confirmation.png)

## View-Only

- [compact](view-only/compact.png)
- [maximized](view-only/maximized.png)

## Themes

- [graphite editor](themes/graphite-editor.png)
- [graphite library](themes/graphite-library.png)
- [light editor](themes/light-editor.png)
- [light library](themes/light-library.png)
- [midnight editor](themes/midnight-editor.png)
- [midnight library](themes/midnight-library.png)
- [oled editor](themes/oled-editor.png)
- [oled library](themes/oled-library.png)

## Responsive layouts

- [editor compact](responsive/editor-compact.png)
- [editor maximized](responsive/editor-maximized.png)
- [editor narrow](responsive/editor-narrow.png)
- [editor restored](responsive/editor-restored.png)
- [editor short menu columns](responsive/editor-short-menu-columns.png)
- [library compact](responsive/library-compact.png)
- [library maximized](responsive/library-maximized.png)
- [library restored](responsive/library-restored.png)

## Menus and dialogs

- [active card context](menus/active-card-context.png)
- [application menu](menus/application-menu.png)
- [archived card context](menus/archived-card-context.png)
- [editor more actions](menus/editor-more-actions.png)
- [permanent delete confirmation](menus/permanent-delete-confirmation.png)
- [sort menu](menus/sort-menu.png)
- [trash card context](menus/trash-card-context.png)

## Settings

- [appearance dark](settings/appearance-dark.png)
- [appearance light](settings/appearance-light.png)
- [keyboard shortcuts](settings/keyboard-shortcuts.png)

## Contact sheets

- [complete gallery](contact-sheets/complete-gallery.png)
- [editor](contact-sheets/editor.png)
- [formatting](contact-sheets/formatting.png)
- [library](contact-sheets/library.png)
- [menus](contact-sheets/menus.png)
- [modes](contact-sheets/modes.png)
- [responsive](contact-sheets/responsive.png)
- [settings](contact-sheets/settings.png)
- [themes](contact-sheets/themes.png)
- [view only](contact-sheets/view-only.png)

## Honest capture boundaries

- Rich Text exposes bullet and numbered lists; the current product has no checklist control, so no checklist image is fabricated.
- The custom text/highlight color buttons are visible in the formatting views, but the portal-backed color chooser could not be reliably opened by the isolated accessibility harness; no misleading chooser image is included.
- The stable find/replace overview is retained from the existing repository gallery. The isolated in-process renderer could not capture the editor after closing its underlying library window, so three misleading new search images were removed.
- Wayland reports Always on Top as unavailable in the captured environment; the menu documents that real disabled state.
- Screenshots show UI states and controls. Automated tests remain the source of truth for behavior that cannot be proven by a still image.
