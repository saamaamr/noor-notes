# Noor Notes Multi-Palette Dark Mode Design

**Date:** 2026-08-06
**Status:** Approved design

## Goal

Replace the inconsistent prototype dark styling with a native, polished appearance system that gives equal priority to visual quality, readability, accessibility, and predictable interaction.

## Appearance Modes

Noor Notes exposes five appearance choices:

- **System** follows GNOME light/dark preference. When the system is dark, Noor Notes uses the saved preferred dark palette.
- **Light** forces the existing refined light palette.
- **Graphite** uses warm-neutral charcoal surfaces and restrained indigo accents. It is the default dark palette.
- **Midnight** uses deep navy-neutral surfaces and a calm blue accent.
- **OLED** uses near-black surfaces with carefully separated elevated layers and higher-contrast accents.

The selected appearance and preferred dark palette persist in the existing user configuration area. Note data and the encrypted database remain unchanged.

## Controls and Interaction

The same appearance state is exposed in three synchronized locations:

1. A compact header palette button cycles through Graphite, Midnight, and OLED while dark mode is active. Its tooltip and accessible label announce the active palette.
2. The main application menu provides direct selection of System, Light, Graphite, Midnight, and OLED.
3. A native Appearance settings window provides labeled preview rows for every mode and explains System behavior.

Changing a mode updates every open Noor Notes window immediately. All three controls reflect the same active state. Changes use a restrained native crossfade only when desktop animations are enabled.

## Semantic Palette Architecture

GTK CSS cannot rely on browser custom properties. The implementation therefore uses explicit root style classes and palette-scoped selectors:

- `.nn-theme-light`
- `.nn-theme-graphite`
- `.nn-theme-midnight`
- `.nn-theme-oled`

A focused appearance service owns persistence, resolves System into an effective theme, and applies exactly one effective class to every application window. Reusable CSS selectors cover background, surface, raised surface, border, primary and secondary text, accent, hover, selection, focus, disabled, success, warning, and error states.

No widget owns a hard-coded theme decision.

## Palette Intent

### Graphite

- Background: warm charcoal
- Surface: soft graphite
- Raised surface: visibly separated without heavy shadow
- Accent: restrained indigo-blue
- Mood: premium, calm, GNOME-native

### Midnight

- Background: deep blue-black
- Surface: desaturated navy
- Raised surface: slightly brighter navy
- Accent: clear sky blue
- Mood: focused, atmospheric, suitable for long writing sessions

### OLED

- Background: near black
- Surface: minimally lifted neutral black
- Raised surface: clear but restrained separation
- Accent: vivid accessible violet-blue
- Mood: high contrast and distraction-free without losing hierarchy

## Component Treatment

The palette applies consistently to:

- application and editor header bars
- sidebar and selected navigation state
- note list background, cards, metadata, and selection
- preview surface and document typography
- editor canvas, title, tags, cursor, and text selection
## Adaptive Iconography

All symbolic icons change automatically with the effective theme:

- neutral icons use the palette's primary or secondary foreground color
- active, selected, and toggled icons use the palette accent with a readable selected background
- disabled icons remain identifiable and meet the surrounding disabled-state contrast
- destructive icons remain neutral by default and use the error color only on destructive hover, focus, or confirmed destructive actions
- success and warning icons use their semantic palette colors and never communicate status without a text or accessible-label equivalent

Icons remain native symbolic assets; theme switching never substitutes emoji or unrelated colored artwork.

- formatting and appearance popovers
- find/replace panel
- status bars
- dialogs, menus, tooltips, and context menus
- destructive, warning, success, disabled, hover, pressed, and focused states

The editor canvas remains the visual focus. Dark surfaces avoid both flat gray monotony and large pure-black rectangles outside OLED mode.

## Note Paper Colors

Each optional paper color receives an explicit dark counterpart. Every paper palette defines readable foreground, secondary text, link, cursor, selection, and highlight colors. Color never becomes the sole status indicator.

## Accessibility

- Text and essential controls target WCAG AA contrast.
- Focus rings remain visible on every palette.
- Keyboard navigation and screen-reader labels cover all appearance controls.
- The header button has a descriptive tooltip rather than relying on its icon.
- High-contrast desktop settings remain authoritative.
- Reduced-motion settings suppress optional transitions.
- The design remains usable at 125%, 150%, and 200% scaling.

## Data and Failure Behavior

Appearance preferences are stored atomically outside the notes database. Invalid or unreadable preference files fail closed to System mode plus Graphite as the dark preference. A persistence failure keeps the in-memory selection active for the session and shows a non-destructive error message; it never affects notes.

## Testing and Visual Verification

Tests cover:

- preference defaults, round trips, and invalid values
- System resolution against light and dark desktop preferences
- immediate synchronization between all controls
- exactly one effective root class per window
- semantic selectors for all three dark palettes
- accessible names and tooltips
- note-paper foreground mappings
- persistence failure behavior

Fresh screenshots will cover Graphite, Midnight, OLED, the appearance menu, Appearance settings, the editor, and representative note paper colors. Light mode regression screenshots remain part of the check.

## Scope

This change is limited to appearance infrastructure, dark palettes, synchronized controls, settings UI, tests, documentation, installation, and screenshots. It does not alter note storage, application identity, sync behavior, or Snap Store revisions.

