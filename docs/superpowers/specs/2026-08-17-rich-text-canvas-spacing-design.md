# Rich Text Canvas Spacing Design

## Goal

Reduce the internal writing-area margins in Rich Text mode so content uses the note window more efficiently.

## Scope

The Rich Text editor canvas will use 5 pixels at the top and bottom and 8 pixels at the left and right. Markdown, Plain Text, and Code editor margins remain unchanged.

The change will preserve the existing reading-width clamp, CSS classes, accessibility label, formatting behavior, writing assistance, and note persistence.

## Implementation

Update the Rich Text branch of `configure_editor_canvas` to apply horizontal margins of 8 pixels and vertical margins of 5 pixels. Keep the source-mode branch unchanged.

## Verification

Update the focused editor-canvas regression test first so it asserts all four Rich Text margins and confirms source-mode margins remain unchanged. Run the test under a virtual display, then run formatting and relevant workspace checks.
