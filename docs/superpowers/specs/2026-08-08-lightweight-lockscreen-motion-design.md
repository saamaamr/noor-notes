# Lightweight Lock-Screen Motion Design

Date: 2026-08-08
Status: Approved concept; implementation pending

## Goal

Add polished but lightweight motion to Mamun's existing GNOME lock screen without changing the Quran quotation, wallpaper artwork, clock format, password field, day/night selection, or unlock behavior.

The requested combined experience includes:

- a short wallpaper entrance animation;
- a short clock and authentication-prompt entrance animation;
- a very subtle continuous ambient golden glow;
- automatic suppression of nonessential motion when GNOME animations are disabled or Power Saver is active.

## Current Environment

- GNOME Shell runs the enabled `wack-lockscreen-clock@rinzler69-wastaken.github.com` extension in Cupertino mode.
- Its existing slow unlock crossfade remains enabled and must not be replaced.
- The current dark wallpaper is `/home/mamun/Pictures/Noor/noor-premium-quote.png` at 3072x1728.
- The Quran quotation is part of the current wallpaper composition; it is not to be regenerated or rewritten.
- Noor Notes' existing `noor-notes-windowing@saamaamr.github.io` extension is limited to window controls and must retain that narrow security boundary.
- No automatic day/night timer belongs to this feature.

## Architecture

Create a separate companion GNOME Shell extension under `extensions/lockscreen-motion/`. Do not patch the installed WACK extension and do not add lock-screen privileges to the Noor Notes window-control extension.

The companion extension will:

1. declare `user` and `unlock-dialog` session modes;
2. observe lock-screen activation and actor availability;
3. feature-detect the relevant GNOME/WACK actors rather than assuming they always exist;
4. attach animations only while the lock screen is visible;
5. remove every signal, transition, effect, and temporary actor during unlock or extension disable;
6. degrade to a safe no-op if GNOME Shell internals or the WACK layout are unavailable.

Pure policy helpers will be separated from Shell actor manipulation so power, accessibility, timing, and capability behavior can be tested with GJS without starting GNOME Shell.

## Motion Design

### Wallpaper entrance

- Duration: 900 ms.
- Start state: opacity 0 and scale 1.015.
- End state: opacity 255 and scale 1.0.
- Easing: ease-out cubic.
- Run once each time the lock screen becomes active.
- Animate the complete existing wallpaper composition, including its embedded quotation, so spelling and typography remain unchanged.

### Clock and prompt entrance

- Clock/date: 520 ms rise of 14 px with a fade from 0 to full opacity.
- Authentication prompt: retain WACK's existing prompt transition; the companion must not apply a second transform when that prompt is already animating.
- Stagger: clock begins 120 ms after the wallpaper entrance starts.
- Unlock crossfade: keep WACK's configured slow crossfade unchanged.

### Ambient glow

- Add one noninteractive, nonfocusable glow actor behind foreground controls and above the wallpaper.
- Use a warm antique-gold tint consistent with the existing Islamic artwork.
- Maximum opacity must remain low enough that text contrast is not reduced.
- Animate only opacity between two restrained values over an 8-second cycle using compositor transitions; do not use a video, particle system, frame callback, or repeating JavaScript timeout.
- Stop and remove the glow as soon as the lock screen is dismissed.

## Accessibility and Power Policy

Entrance and ambient motion are allowed only when GNOME's `enable-animations` preference is true.

When Power Saver is active:

- keep the one-time entrance fade only;
- skip scaling, translation, and continuous ambient glow;
- never prevent the display from sleeping.

When GNOME animations are disabled:

- show all actors immediately in their final state;
- do not create the ambient glow;
- do not schedule animation work.

The extension must respond to preference and power-profile changes while the lock screen is open.

## Safety and Failure Handling

- Never modify authentication, password input, PAM, session locking, or unlock decisions.
- Never capture keyboard or pointer input.
- The glow actor must be nonreactive and excluded from focus navigation.
- Actor discovery must use optional checks; missing actors produce a logged diagnostic and a no-op, not a Shell exception.
- Repeated lock/unlock cycles must not accumulate actors, effects, transitions, or signal handlers.
- Disable must restore opacity, scale, translation, and effects on any actor touched by the extension.
- No network access, telemetry, analytics, remote assets, or new system dependency.
- No Snap metadata or Snap Store action.

## Installation

Extend `scripts/install-gnome-extension.sh` to install both repository-owned extensions into the user's local GNOME extension directory. Installation remains user-local and does not require root.

Because GNOME Shell cannot be safely restarted in a Wayland session, installation will report that one logout/login is required before first use or after extension-code changes.

## Testing

Automated tests will cover:

- full motion under normal power with animations enabled;
- entrance-only behavior under Power Saver;
- no motion when GNOME animations are disabled;
- safe no-op when required actors are absent;
- idempotent activation and cleanup across repeated cycles;
- metadata includes the unlock-dialog session mode;
- installer copies and enables the companion extension without touching unrelated extensions.

Manual verification will cover:

- lock with Super+L;
- wallpaper entrance;
- clock entrance;
- ambient glow subtlety and text contrast;
- prompt and unlock transitions;
- Power Saver behavior;
- GNOME animations-disabled behavior;
- repeated lock/unlock cycles;
- screen blanking and unlock reliability.

## Acceptance Criteria

- The lock screen visibly gains all three approved motion layers.
- The animation remains smooth and restrained, with no video playback or per-frame JavaScript work.
- The Quran quotation and current visual composition remain unchanged.
- Unlocking, password entry, screen blanking, and WACK's existing crossfade continue working.
- Reduced-motion and Power Saver policies behave exactly as specified.
- The extension cleans up completely and fails safely.
