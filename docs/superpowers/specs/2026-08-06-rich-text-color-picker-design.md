# Rich Text Color Picker Design

Date: 2026-08-06
Status: Approved
Scope: Rich Text mode only

## Objective

Replace Noor Notes' incomplete four-color formatting controls with a professional preset palette and native custom color pickers for text color and highlight color. Colors must be visible, keyboard accessible, persistent after save and reopen, and compatible with existing rich notes.

Markdown, Plain Text, and Code remain source-editor modes and do not expose rich color formatting.

## Current problem and root cause

The current formatting popover creates four empty buttons for each color row: charcoal, blue, green, and red. The buttons receive CSS class names, but the design system defines no matching swatch colors, so they can appear blank. The signal handlers exist, but there is no native custom picker.

The rich document model already stores foreground and highlight values as optional strings. Named preset tags are registered when a buffer is prepared, but arbitrary custom values do not create tags during editing or reload. Therefore simply adding a picker would make custom colors temporary and would fail persistence.

The color and other rich-formatting controls are also not all disabled in source modes.

## Chosen interaction design

The compact formatting popover keeps separate **Text color** and **Highlight** sections.

Each section contains:

- a labeled row of professional preset swatches;
- a native GTK custom color button;
- a reset action;
- visible focus and selected states;
- accessible names and tooltips.

Text color presets:

- Auto
- Slate
- Blue
- Teal
- Green
- Amber
- Red
- Purple

Highlight presets:

- None
- Yellow
- Blue
- Mint
- Green
- Peach
- Pink
- Lavender

Auto removes the foreground mark and restores the editor's normal text color. None removes the highlight mark. These are actions with a reset icon rather than misleading transparent swatches.

Preset swatches are compact circles inside 32–36 px keyboard-focusable buttons. The custom picker sits at the end of each row and uses GTK's native `ColorDialogButton`. It does not allow alpha because text serialization stores opaque RGB colors.

The popover remains compact and uses wrapping swatch rows so every control stays reachable in narrow note windows.

## Professional adaptive palette

Preset identifiers are semantic and stable in storage. Their rendered colors adapt to the active Noor theme so presets remain readable on light and dark writing canvases.

Text presets use these mappings:

| ID | Light | Dark |
| --- | --- | --- |
| slate | `#334155` | `#E2E8F0` |
| blue | `#1D4ED8` | `#93C5FD` |
| teal | `#0F766E` | `#5EEAD4` |
| green | `#15803D` | `#86EFAC` |
| amber | `#A16207` | `#FCD34D` |
| red | `#B91C1C` | `#FCA5A5` |
| purple | `#7E22CE` | `#D8B4FE` |

Highlight presets use these mappings:

| ID | Light | Dark |
| --- | --- | --- |
| yellow | `#FEF3C7` | `#5F4B16` |
| blue | `#DBEAFE` | `#1E3A5F` |
| mint | `#CCFBF1` | `#134E4A` |
| green | `#DCFCE7` | `#14532D` |
| peach | `#FFEDD5` | `#7C2D12` |
| pink | `#FCE7F3` | `#6B214B` |
| lavender | `#EDE9FE` | `#4C3575` |

Graphite, Midnight, and OLED all use the dark preset mapping. Custom colors remain the exact user-selected RGB value and do not change with the application theme.

Preset swatch previews use the actual mapped color for the active theme. Selection is shown with both a check indicator and a border, never by color alone.

## Architecture

A focused rich-color module owns:

- preset IDs, labels, and light/dark RGB mappings;
- legacy alias compatibility;
- RGB normalization and validation;
- conversion between stored values and GTK tag names;
- foreground/highlight tag creation;
- applying the current theme to preset tags.

A reusable color-palette UI component owns:

- section label;
- reset control;
- preset swatches;
- `ColorDialogButton`;
- accessible labels, tooltips, and selected-state synchronization;
- an activation callback carrying either reset, preset ID, or normalized custom RGB.

`EditorToolbar` composes two instances of this component rather than constructing anonymous blank buttons.

`RichBuffer` remains responsible for applying marks to the selected range. It gains explicit operations to:

- apply a validated foreground value;
- apply a validated highlight value;
- clear foreground only;
- clear highlight only;
- ensure custom tags exist before applying or loading;
- update adaptive preset tag properties when the theme changes.

The note window subscribes rich buffers to appearance changes using weak references, matching the source-editor palette approach without retaining closed windows.

## Persistence and compatibility

No database migration is required. `TextMarks.foreground` and `TextMarks.highlight` already store optional strings.

Preset colors are stored by semantic ID, such as `blue` or `lavender`. Custom colors are normalized to uppercase `#RRGGBB`.

Dynamic GTK tag names encode custom colors without placing unvalidated input directly into a tag name. Snapshot decoding converts the tag back to the normalized stored value.

On reload, Noor Notes validates every stored color before creating a tag:

- known preset IDs are accepted;
- supported legacy IDs are mapped safely;
- valid `#RRGGBB` custom colors are accepted;
- invalid values are ignored without damaging note text.

Existing values `charcoal`, `blue`, `green`, and `red` remain readable. Foreground `charcoal` is treated as a legacy alias for `slate`. Legacy highlight `charcoal` retains a hidden compatibility mapping (`#D8C99B` light and `#5B5030` dark), while legacy highlight `red` maps to the new pink family. These compatibility entries remain loadable but are not shown as duplicate new presets.

Autosave continues to snapshot the rich document normally. Applying or clearing a color is one undoable user action and triggers the existing dirty/save-state path.

## Mode behavior

All text color, highlight color, font size, alignment, and clear-formatting controls are enabled only for Rich Text notes.

In Markdown, Plain Text, and Code:

- color controls are insensitive;
- their tooltips explain that rich colors require Rich Text;
- source syntax colors continue to come from GtkSourceView palettes;
- no rich tags are applied to source buffers.

## Error handling

The native picker is cancelled without changing the selection.

Invalid persisted custom colors fail closed: text remains readable with the default foreground or no highlight.

If no text is selected, choosing a color does not alter existing content or change the active preset indicator. The editor keeps focus. Applying colors to future typed text is outside this scope.

## Accessibility

Every preset exposes a descriptive label such as “Blue text” or “Yellow highlight.”

Controls support Tab navigation, Enter/Space activation, visible focus rings, and screen-reader names. Selected presets expose a checked state. Reset actions have explicit labels. Custom picker buttons report their current RGB value.

Swatches have a non-color selection indicator. Preset text mappings target readable contrast against the corresponding default writing canvas; custom color contrast remains user-controlled.

## Testing

Automated tests cover:

- all expected text and highlight presets;
- visible swatch CSS for light and dark themes;
- accessible labels and keyboard-focusable controls;
- native custom picker presence;
- rich-only sensitivity;
- preset apply, replace, and reset;
- custom RGB normalization and validation;
- custom foreground and highlight snapshot/load round trips;
- compatibility with legacy charcoal, blue, green, and red marks;
- invalid stored colors failing closed;
- theme switching updating preset tags but not custom RGB tags;
- undo/redo for color changes;
- autosave/database persistence after close and reopen;
- no changes to source-editor syntax palettes.

Verification runs formatting, strict Clippy, the workspace test suite, and the release build. Manual verification covers both pickers, presets, resets, save/reopen, undo/redo, all four themes, narrow windows, and keyboard-only operation.

## Out of scope

- Rich colors in Markdown, Plain Text, or Code
- Alpha/transparency in text or highlight colors
- Gradients
- Per-note saved custom palette history
- Eyedropper integration beyond what the native GTK dialog provides
- Changing the database schema
- Snap building, uploading, or releasing
