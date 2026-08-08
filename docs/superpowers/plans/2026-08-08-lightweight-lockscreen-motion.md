# Lightweight Lock-Screen Motion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe, lightweight GNOME Shell companion extension that animates the existing Noor lock-screen wallpaper, clock, and ambient glow while respecting reduced motion and Power Saver.

**Architecture:** Keep Noor Notes window controls and the third-party WACK extension unchanged. Add a separate repository-owned `noor-lockscreen-motion@saamaamr.github.io` extension with a pure policy/discovery layer and a Shell-only motion session. The extension feature-detects WACK/GNOME actors, uses Clutter property transitions rather than video or JavaScript frame loops, and restores every touched actor on cleanup.

**Tech Stack:** GNOME Shell 50, GJS ES modules, Clutter, St, Gio/GSettings, Power Profiles D-Bus, shell scripts, GJS tests.

## Global Constraints

- Preserve `/home/mamun/Pictures/Noor/noor-premium-quote.png`, its Quran quotation, clock format, password field, day/night selection, and WACK unlock behavior.
- Do not patch `wack-lockscreen-clock@rinzler69-wastaken.github.com`.
- Do not add `unlock-dialog` privileges to `noor-notes-windowing@saamaamr.github.io`.
- Do not modify authentication, PAM, password input, session-lock decisions, keyboard input, or pointer input.
- Use no video, particle engine, per-frame JavaScript callback, repeating JavaScript timeout, network access, analytics, telemetry, remote asset, or new system dependency.
- Ambient motion must stop under Power Saver and all motion must stop when GNOME `enable-animations` is false.
- The extension must fail as a safe no-op when private GNOME/WACK actors are unavailable.
- No Snap metadata, build, upload, release, or Store action.

---

### Task 1: Motion policy and actor discovery contracts

**Files:**
- Create: `extensions/lockscreen-motion/policy.js`
- Create: `extensions/lockscreen-motion/actorDiscovery.js`
- Create: `extensions/lockscreen-motion/tests/test-policy.js`
- Create: `extensions/lockscreen-motion/tests/test-actor-discovery.js`

**Interfaces:**
- Produces: `motionPlan({lockActive, animationsEnabled, powerProfile, actorsReady}) -> {wallpaperFade, wallpaperScale, clockEntrance, ambientGlow}`.
- Produces: `findDescendantByStyleClass(root, className) -> actor | null`.
- Produces: `findClockActor(lockDialogGroup) -> actor | null`, locating the ancestor of the WACK `wack-time` descendant.
- Produces: `discoverActors({screenShield}) -> {backgrounds, clock, host} | null`.

- [ ] **Step 1: Write the failing policy test**

```js
import {motionPlan} from '../policy.js';

assertEqual(motionPlan({
    lockActive: true,
    animationsEnabled: true,
    powerProfile: 'balanced',
    actorsReady: true,
}), {
    wallpaperFade: true,
    wallpaperScale: true,
    clockEntrance: true,
    ambientGlow: true,
});

assertEqual(motionPlan({
    lockActive: true,
    animationsEnabled: true,
    powerProfile: 'power-saver',
    actorsReady: true,
}), {
    wallpaperFade: true,
    wallpaperScale: false,
    clockEntrance: false,
    ambientGlow: false,
});

assertEqual(motionPlan({
    lockActive: true,
    animationsEnabled: false,
    powerProfile: 'balanced',
    actorsReady: true,
}), {
    wallpaperFade: false,
    wallpaperScale: false,
    clockEntrance: false,
    ambientGlow: false,
});
```

- [ ] **Step 2: Run the test and verify RED**

Run: `gjs -m extensions/lockscreen-motion/tests/test-policy.js`

Expected: FAIL because `extensions/lockscreen-motion/policy.js` does not exist.

- [ ] **Step 3: Implement the minimal pure policy**

```js
const DISABLED = Object.freeze({
    wallpaperFade: false,
    wallpaperScale: false,
    clockEntrance: false,
    ambientGlow: false,
});

export function motionPlan({lockActive, animationsEnabled, powerProfile, actorsReady}) {
    if (!lockActive || !animationsEnabled || !actorsReady)
        return {...DISABLED};
    if (powerProfile === 'power-saver') {
        return {
            wallpaperFade: true,
            wallpaperScale: false,
            clockEntrance: false,
            ambientGlow: false,
        };
    }
    return {
        wallpaperFade: true,
        wallpaperScale: true,
        clockEntrance: true,
        ambientGlow: true,
    };
}
```

- [ ] **Step 4: Write actor-discovery tests using plain fake actors**

Test a WACK-like tree containing a `wack-time` label, a tree without that label, an absent dialog, an empty background group, and a valid screen-shield fixture. Assert that missing inputs return `null` and never throw.

```js
const clock = node('', [node('', [node('wack-time')])]);
const host = node('', [clock]);
assertSame(findClockActor(host), clock);
assertSame(findClockActor(node('', [])), null);
assertSame(discoverActors({screenShield: null}), null);
```

- [ ] **Step 5: Run actor-discovery tests and verify RED**

Run: `gjs -m extensions/lockscreen-motion/tests/test-actor-discovery.js`

Expected: FAIL because the discovery module is absent.

- [ ] **Step 6: Implement recursive feature-detected discovery**

Use only optional property/method checks. Treat `dialog._backgroundGroup` as an iterable, use `screenShield._lockDialogGroup` as the host, and find the clock wrapper by locating `wack-time` then walking to the direct child of the host. Never import GNOME Shell modules into this pure file.

- [ ] **Step 7: Run both tests and verify GREEN**

Run: `gjs -m extensions/lockscreen-motion/tests/test-policy.js && gjs -m extensions/lockscreen-motion/tests/test-actor-discovery.js`

Expected: both exit 0.

- [ ] **Step 8: Commit**

```bash
git add extensions/lockscreen-motion/policy.js extensions/lockscreen-motion/actorDiscovery.js extensions/lockscreen-motion/tests
git commit -m "test: define lockscreen motion policy"
```

---

### Task 2: Compositor-only motion session and safe cleanup

**Files:**
- Create: `extensions/lockscreen-motion/motionSession.js`
- Create: `extensions/lockscreen-motion/tests/test-session-state.js`
- Modify: `extensions/lockscreen-motion/policy.js`

**Interfaces:**
- Consumes: `motionPlan(...)` and `discoverActors(...)` from Task 1.
- Produces: `MotionSession` with `start({screenShield, animationsEnabled, powerProfile})`, `refreshPolicy(...)`, and `stop()`.
- Produces: pure `SessionState` with `begin(actorKey)`, `track(actorKey)`, and `clear()` for idempotence tests.

- [ ] **Step 1: Write failing lifecycle-state tests**

```js
const state = new SessionState();
assertTrue(state.begin('lock-1'));
assertFalse(state.begin('lock-1'));
state.track('background-a');
state.track('glow');
assertEqual(state.clear().sort(), ['background-a', 'glow']);
assertTrue(state.begin('lock-1'));
```

Also assert that `clear()` on a fresh state is safe and repeated `clear()` returns an empty list.

- [ ] **Step 2: Run and verify RED**

Run: `gjs -m extensions/lockscreen-motion/tests/test-session-state.js`

Expected: FAIL because `SessionState` is missing.

- [ ] **Step 3: Implement `SessionState` and verify GREEN**

Keep this class free of GI imports. It prevents duplicate activation and records every actor/effect that `MotionSession.stop()` must restore.

- [ ] **Step 4: Implement wallpaper entrance transitions**

For every discovered background actor:

```js
actor.remove_transition('noor-wallpaper-entry');
actor.set_pivot_point(0.5, 0.5);
actor.set({opacity: 0, scale_x: plan.wallpaperScale ? 1.015 : 1, scale_y: plan.wallpaperScale ? 1.015 : 1});
actor.ease({
    opacity: 255,
    scale_x: 1,
    scale_y: 1,
    duration: 900,
    mode: Clutter.AnimationMode.EASE_OUT_CUBIC,
});
```

Name or track transitions so `stop()` removes them and restores opacity 255, scale 1, and pivot 0.5/0.5. Under Power Saver, apply opacity only. Under reduced motion, set the final state immediately and do not call `ease()`.

- [ ] **Step 5: Implement the clock entrance**

After one `GLib.timeout_add(GLib.PRIORITY_DEFAULT, 120, ...)` used only as a one-shot stagger, fade the discovered clock wrapper from opacity 0 and translation_y 14 to opacity 255 and translation_y 0 over 520 ms with `EASE_OUT_CUBIC`. Do not animate the authentication prompt; WACK owns it.

Track and cancel the one-shot source on `stop()`. Never create a repeating timeout.

- [ ] **Step 6: Implement the ambient glow with a Clutter property transition**

Create one `St.Widget` with classes `noor-lockscreen-glow` and `noor-lockscreen-glow-active`, `reactive: false`, `can_focus: false`, and `track_hover: false`. Insert it above the background but below foreground controls. Add one `Clutter.PropertyTransition` for `opacity`:

```js
const transition = new Clutter.PropertyTransition({
    property_name: 'opacity',
    duration: 4000,
    repeat_count: -1,
    auto_reverse: true,
    progress_mode: Clutter.AnimationMode.EASE_IN_OUT_SINE,
});
transition.set_from(20);
transition.set_to(42);
glow.add_transition('noor-ambient-breathe', transition);
```

Size the glow from the current monitor geometry and center it. `stop()` must remove the transition and destroy the actor. Do not connect a `new-frame` signal.

- [ ] **Step 7: Implement policy refresh**

When Power Saver begins, remove scale/clock/glow transitions, destroy the glow, restore final actor transforms, and retain only any already-running entrance fade. When animations are disabled, call `stopMotion()` but keep the session observers alive so re-enabling animations can apply policy only on the next lock activation, not unexpectedly mid-session.

- [ ] **Step 8: Add CSS for the noninteractive glow**

Use a restrained antique-gold translucent background and a broad soft box shadow. Do not style the password field, clock, quote, panel, or notifications.

- [ ] **Step 9: Run the pure tests and syntax check**

Run:

```bash
gjs -m extensions/lockscreen-motion/tests/test-policy.js
gjs -m extensions/lockscreen-motion/tests/test-actor-discovery.js
gjs -m extensions/lockscreen-motion/tests/test-session-state.js
gjs -m -c "import('./extensions/lockscreen-motion/motionSession.js')"
```

Expected: all pure tests pass; module import succeeds in a GJS environment with GNOME Shell imports available. If the standalone import cannot resolve Shell resources outside GNOME Shell, use `gjs -m extensions/lockscreen-motion/tests/test-session-state.js` plus `gjs -c` syntax parsing and record that limitation.

- [ ] **Step 10: Commit**

```bash
git add extensions/lockscreen-motion/motionSession.js extensions/lockscreen-motion/stylesheet.css extensions/lockscreen-motion/tests/test-session-state.js extensions/lockscreen-motion/policy.js
git commit -m "feat: add lightweight lockscreen motion session"
```

---

### Task 3: GNOME extension lifecycle, accessibility, and Power Profiles

**Files:**
- Create: `extensions/lockscreen-motion/metadata.json`
- Create: `extensions/lockscreen-motion/extension.js`
- Create: `extensions/lockscreen-motion/tests/test-contract.js`

**Interfaces:**
- Consumes: `MotionSession` from Task 2.
- Produces: extension UUID `noor-lockscreen-motion@saamaamr.github.io` supporting GNOME Shell `50` and session modes `user`, `unlock-dialog`.

- [ ] **Step 1: Write the failing metadata/lifecycle contract test**

Read `metadata.json` and `extension.js` as text. Assert:

- exact UUID and Shell version;
- `session-modes` contains `user` and `unlock-dialog`;
- extension connects to `Main.screenShield` `active-changed`;
- extension reads `org.gnome.desktop.interface` `enable-animations`;
- extension uses `net.hadess.PowerProfiles` `ActiveProfile`;
- `disable()` disconnects all signals and stops the motion session;
- no key/pointer event controller, `new-frame`, network API, or repeating GLib timeout appears.

- [ ] **Step 2: Run and verify RED**

Run: `gjs -m extensions/lockscreen-motion/tests/test-contract.js`

Expected: FAIL because metadata and extension lifecycle do not exist.

- [ ] **Step 3: Create metadata**

```json
{
  "uuid": "noor-lockscreen-motion@saamaamr.github.io",
  "name": "Noor Lockscreen Motion",
  "description": "Adds restrained, power-aware motion to the Noor GNOME lock screen.",
  "shell-version": ["50"],
  "session-modes": ["user", "unlock-dialog"],
  "url": "https://github.com/saamaamr/noor-notes",
  "version": 1
}
```

- [ ] **Step 4: Implement extension enable/disable**

On enable:

- create `Gio.Settings({schema_id: 'org.gnome.desktop.interface'})`;
- create the Power Profiles proxy with `Gio.DBusProxy.makeProxyWrapper`;
- connect `Main.screenShield` `active-changed`, interface `changed::enable-animations`, and proxy `g-properties-changed` with owned IDs or `connectObject`;
- on active lock, schedule one idle callback so WACK can finish its actor layout, then call `MotionSession.start(...)`;
- immediately synchronize once after connecting signals so enabling during an already-active unlock-dialog is handled;
- on unlock, call `MotionSession.stop()`.

On disable:

- remove the pending idle source;
- disconnect all settings, D-Bus, and screen-shield signals;
- call `MotionSession.stop()`;
- clear references.

Wrap actor setup in `try/catch`; log one concise diagnostic and leave the lock screen untouched on failure.

- [ ] **Step 5: Run lifecycle contract and all extension tests**

Run:

```bash
gjs -m extensions/lockscreen-motion/tests/test-contract.js
gjs -m extensions/lockscreen-motion/tests/test-policy.js
gjs -m extensions/lockscreen-motion/tests/test-actor-discovery.js
gjs -m extensions/lockscreen-motion/tests/test-session-state.js
gjs -m extensions/gnome/tests/test-policy.js
```

Expected: all exit 0; the existing window-control policy remains green.

- [ ] **Step 6: Commit**

```bash
git add extensions/lockscreen-motion
git commit -m "feat: add Noor lockscreen motion extension"
```

---

### Task 4: User-local installation and regression checks

**Files:**
- Modify: `scripts/install-gnome-extension.sh`
- Modify: `tests/install_ubuntu.sh`
- Create: `tests/lockscreen_motion_install.sh`

**Interfaces:**
- Consumes: both repository extension directories.
- Produces: user-local installs at `$XDG_DATA_HOME/gnome-shell/extensions/noor-notes-windowing@saamaamr.github.io` and `$XDG_DATA_HOME/gnome-shell/extensions/noor-lockscreen-motion@saamaamr.github.io`.

- [ ] **Step 1: Write a failing isolated installer test**

Use `mktemp -d`, set `XDG_DATA_HOME` to that directory, and shadow `gnome-extensions` with a test stub that records enable requests. Run the installer and assert:

- both UUID directories exist;
- exactly the declared files are copied;
- both UUIDs are requested for enable;
- no installed WACK or unrelated extension path is modified;
- output mentions logout/login rather than attempting to restart GNOME Shell.

- [ ] **Step 2: Run and verify RED**

Run: `bash tests/lockscreen_motion_install.sh`

Expected: FAIL because the installer only copies the window-control extension.

- [ ] **Step 3: Refactor the installer around `install_extension(source, uuid)`**

Install `metadata.json`, `extension.js`, `stylesheet.css`, `policy.js`, `actorDiscovery.js`, and `motionSession.js` only when present in that source directory. Keep the current window-control file list intact. Enable both UUIDs best-effort without disabling anything else.

- [ ] **Step 4: Extend Ubuntu installation assertions**

Update repository installation tests to expect the new companion extension files while keeping application, desktop, icon, and window-control assertions unchanged.

- [ ] **Step 5: Run installer tests and extension tests**

Run:

```bash
bash tests/lockscreen_motion_install.sh
bash tests/install_ubuntu.sh
gjs -m extensions/lockscreen-motion/tests/test-contract.js
gjs -m extensions/gnome/tests/test-policy.js
```

Expected: all exit 0.

- [ ] **Step 6: Commit**

```bash
git add scripts/install-gnome-extension.sh tests/install_ubuntu.sh tests/lockscreen_motion_install.sh
git commit -m "build: install lockscreen motion companion"
```

---

### Task 5: Documentation, full verification, and local activation

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-08-08-lightweight-lockscreen-motion-design.md`

**Interfaces:**
- Consumes: verified extension and installer from Tasks 1-4.
- Produces: accurate user instructions and an installed local extension; no remote publication.

- [ ] **Step 1: Update README**

Document the three motion layers, reduced-motion and Power Saver behavior, GNOME Shell 50 requirement, user-local installation, one logout/login requirement, and how to disable only `noor-lockscreen-motion@saamaamr.github.io` if troubleshooting. Explicitly state that the feature does not control automatic day/night switching.

- [ ] **Step 2: Update spec status without overstating visual verification**

After automated checks and local installation succeed, change the status line from `Approved concept; implementation pending` to `Implemented; real GNOME lock-screen verification pending`. Change it to `Implemented and verified` only after the real Super+L checklist has been observed after logout/login.

- [ ] **Step 3: Run the full automated verification gate**

```bash
gjs -m extensions/lockscreen-motion/tests/test-policy.js
gjs -m extensions/lockscreen-motion/tests/test-actor-discovery.js
gjs -m extensions/lockscreen-motion/tests/test-session-state.js
gjs -m extensions/lockscreen-motion/tests/test-contract.js
gjs -m extensions/gnome/tests/test-policy.js
bash tests/lockscreen_motion_install.sh
bash tests/install_ubuntu.sh
PATH=/home/mamun/.cargo/bin:$PATH cargo fmt --all -- --check
PATH=/home/mamun/.cargo/bin:$PATH cargo clippy --workspace --all-targets --all-features -- -D warnings
PATH=/home/mamun/.cargo/bin:$PATH cargo test --workspace
PATH=/home/mamun/.cargo/bin:$PATH cargo build --workspace --release
git diff --check
```

Expected: every command exits 0 with no Rust warning or test failure.

- [ ] **Step 4: Install to the user account**

Run: `bash scripts/install-gnome-extension.sh`

Expected: both repository-owned UUIDs are installed and enable requests succeed. Do not modify or reinstall WACK.

- [ ] **Step 5: Verify installed files and enable state**

```bash
gnome-extensions info noor-lockscreen-motion@saamaamr.github.io
gnome-extensions list --enabled | rg '^noor-lockscreen-motion@saamaamr.github.io$'
```

Expected: extension is installed and enabled. If GNOME reports that a logout/login is required before loading the new UUID, record that honestly instead of claiming live activation.

- [ ] **Step 6: Manual lock-screen verification after logout/login**

Verify Super+L entrance, wallpaper fade/scale, clock rise/fade, ambient breathing glow, password prompt, unlock crossfade, Power Saver, reduced motion, display blanking, and three repeated lock/unlock cycles. Do not claim these visual checks passed unless they were observed in the real GNOME session.

- [ ] **Step 7: Commit documentation and final verification state**

```bash
git add README.md docs/superpowers/specs/2026-08-08-lightweight-lockscreen-motion-design.md
git commit -m "docs: document lightweight lockscreen motion"
```

- [ ] **Step 8: Report without publishing**

Report commits, exact modified files, automated command results, install state, whether logout/login and real lock-screen verification remain, and any GNOME private-API limitation. Do not push to GitHub or perform any Snap action without a new explicit request.
