# Responsive Editor Toolbar and More Menu Design

## Problem

The editor toolbar is a single horizontal `gtk::Box`. Its fixed-size action groups require more width than a compact note window provides. GTK clips the trailing controls, making the `More note actions` button unreliable until the window is maximized. The More popover is also one tall vertical column, so it can exceed the usable height of a short window.

## Goals

- Keep every editor action reachable in compact and maximized windows.
- Preserve a compact single-row toolbar when space permits.
- Wrap toolbar actions into additional rows automatically when width is limited.
- Flow More-menu actions into additional columns when height is limited.
- Preserve existing action wiring, state, shortcuts, tooltips, and accessibility.
- Avoid database, storage, packaging, and Snap changes.

## Toolbar Layout

Replace the fixed horizontal toolbar container with a non-selectable `gtk::FlowBox`.

The actions remain ordered by editing priority:

1. Undo and Redo
2. Find
3. Bold, Italic, Bullets, and Formatting
4. Emoji
5. More note actions

The FlowBox uses compact row and column spacing. At normal widths all controls occupy one row. When the available width becomes smaller than their natural width, GTK moves trailing controls to a second or subsequent row instead of clipping them. The More button remains part of the flow and therefore remains allocated, visible, keyboard-focusable, and clickable.

Decorative separators are removed from the wrapping container because separators can become stranded at row boundaries. Visual grouping comes from spacing and button states instead.

## More Popover Layout

Replace the single tall action column with two structured areas:

- A vertically oriented, non-selectable `gtk::FlowBox` for note and view actions.
- A compact editor-mode footer beneath the action flow.

The action flow has a bounded number of rows per column. When all visible actions do not fit within that height, remaining actions continue in the next column. Archived and trashed states continue hiding irrelevant actions; the flow automatically closes those gaps.

The popover remains anchored to the More button and uses a constrained natural size so it fits short note windows. Nested Export and View controls retain their existing popovers. `View Only` remains a direct action in the main More popover.

## Responsive and Interaction Behavior

- Wide window: one toolbar row and the compact More popover.
- Narrow window: two or more toolbar rows with no clipped actions.
- Short window: More actions flow into multiple columns.
- Narrow and short window: both behaviors apply together.
- Opening or closing the popover does not change note content or editor state.
- Existing enabled, disabled, hidden, and toggled states remain authoritative.

## Accessibility

- Flow containers use `SelectionMode::None`; keyboard focus stays on actionable controls.
- Existing tooltips and accessible descriptions remain attached to their buttons.
- Tab order follows the visual action order.
- Wrapped controls retain the existing minimum hit target.
- Theme and high-DPI behavior continue to use native GTK sizing.

## Testing
