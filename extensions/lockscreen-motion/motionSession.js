import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import St from 'gi://St';

import {discoverActors} from './actorDiscovery.js';
import {motionPlan, SessionState} from './policy.js';

const WALLPAPER_DURATION_MS = 900;
const CLOCK_DELAY_MS = 120;
const CLOCK_DURATION_MS = 520;
const GLOW_HALF_CYCLE_MS = 4000;

function transition(actor, name, propertyName, from, to, duration) {
    actor.remove_transition(name);
    const animation = new Clutter.PropertyTransition({
        property_name: propertyName,
        duration,
        progress_mode: Clutter.AnimationMode.EASE_OUT_CUBIC,
    });
    animation.set_from(from);
    animation.set_to(to);
    actor.add_transition(name, animation);
}

function snapshotActor(actor) {
    const pivot = actor.get_pivot_point?.() ?? {x: 0, y: 0};
    return {
        actor,
        opacity: actor.opacity,
        scaleX: actor.scale_x,
        scaleY: actor.scale_y,
        translationY: actor.translation_y,
        pivotX: pivot.x,
        pivotY: pivot.y,
    };
}

function restoreSnapshot(snapshot) {
    const {actor} = snapshot;
    actor.remove_transition('noor-wallpaper-opacity');
    actor.remove_transition('noor-wallpaper-scale-x');
    actor.remove_transition('noor-wallpaper-scale-y');
    actor.remove_transition('noor-clock-opacity');
    actor.remove_transition('noor-clock-translation-y');
    actor.set_pivot_point?.(snapshot.pivotX, snapshot.pivotY);
    actor.set({
        opacity: snapshot.opacity,
        scale_x: snapshot.scaleX,
        scale_y: snapshot.scaleY,
        translation_y: snapshot.translationY,
    });
}

export class MotionSession {
    constructor() {
        this._state = new SessionState();
        this._activationKey = null;
        this._actors = null;
        this._backgroundSnapshots = [];
        this._clockSnapshot = null;
        this._clockDelayId = 0;
        this._glow = null;
        this._animationsEnabled = true;
        this._powerProfile = 'balanced';
    }

    start({screenShield, animationsEnabled, powerProfile, activationKey}) {
        if (activationKey === this._activationKey)
            return false;
        this.stop();

        const actors = discoverActors({screenShield});
        const plan = motionPlan({
            lockActive: true,
            animationsEnabled,
            powerProfile,
            actorsReady: actors !== null,
        });
        if (!actors)
            return false;

        this._activationKey = activationKey;
        this._state.begin(activationKey);
        this._actors = actors;
        this._animationsEnabled = animationsEnabled;
        this._powerProfile = powerProfile;
        this._backgroundSnapshots = actors.backgrounds.map(snapshotActor);
        this._clockSnapshot = snapshotActor(actors.clock);

        if (plan.wallpaperFade)
            this._animateWallpaper(plan.wallpaperScale);
        if (plan.clockEntrance)
            this._scheduleClockEntrance();
        if (plan.ambientGlow)
            this._createGlow();
        return true;
    }

    refreshPolicy({animationsEnabled, powerProfile}) {
        this._animationsEnabled = animationsEnabled;
        this._powerProfile = powerProfile;
        if (!this._actors)
            return;

        if (!animationsEnabled) {
            this._cancelClockEntrance();
            this._removeGlow();
            this._restoreAllSnapshots();
            return;
        }

        if (powerProfile === 'power-saver') {
            this._cancelClockEntrance();
            this._removeGlow();
            for (const snapshot of this._backgroundSnapshots) {
                const {actor} = snapshot;
                actor.remove_transition('noor-wallpaper-scale-x');
                actor.remove_transition('noor-wallpaper-scale-y');
                actor.set({scale_x: snapshot.scaleX, scale_y: snapshot.scaleY});
            }
            this._restoreClock();
        }
    }

    stop() {
        this._cancelClockEntrance();
        this._removeGlow();
        this._restoreAllSnapshots();
        this._backgroundSnapshots = [];
        this._clockSnapshot = null;
        this._actors = null;
        this._activationKey = null;
        this._state.clear();
    }

    _animateWallpaper(withScale) {
        for (const snapshot of this._backgroundSnapshots) {
            const {actor} = snapshot;
            actor.set_pivot_point?.(0.5, 0.5);
            actor.opacity = 0;
            transition(actor, 'noor-wallpaper-opacity', 'opacity',
                0, snapshot.opacity, WALLPAPER_DURATION_MS);
            if (withScale) {
                const startX = snapshot.scaleX * 1.015;
                const startY = snapshot.scaleY * 1.015;
                actor.set({scale_x: startX, scale_y: startY});
                transition(actor, 'noor-wallpaper-scale-x', 'scale-x',
                    startX, snapshot.scaleX, WALLPAPER_DURATION_MS);
                transition(actor, 'noor-wallpaper-scale-y', 'scale-y',
                    startY, snapshot.scaleY, WALLPAPER_DURATION_MS);
            }
            this._state.track(actor);
        }
    }

    _scheduleClockEntrance() {
        this._clockDelayId = GLib.timeout_add(
            GLib.PRIORITY_DEFAULT,
            CLOCK_DELAY_MS,
            () => {
                this._clockDelayId = 0;
                if (!this._clockSnapshot || !this._animationsEnabled || this._powerProfile === 'power-saver')
                    return GLib.SOURCE_REMOVE;
                const {actor, opacity, translationY} = this._clockSnapshot;
                actor.set({opacity: 0, translation_y: translationY + 14});
                transition(actor, 'noor-clock-opacity', 'opacity',
                    0, opacity, CLOCK_DURATION_MS);
                transition(actor, 'noor-clock-translation-y', 'translation-y',
                    translationY + 14, translationY, CLOCK_DURATION_MS);
                this._state.track(actor);
                return GLib.SOURCE_REMOVE;
            });
    }

    _createGlow() {
        const {host, clock} = this._actors;
        const glow = new St.Widget({
            style_class: 'noor-lockscreen-glow',
            reactive: false,
            can_focus: false,
            track_hover: false,
            opacity: 20,
        });
        const stageWidth = global.stage?.width ?? 1920;
        const stageHeight = global.stage?.height ?? 1080;
        const width = Math.min(Math.round(stageWidth * 0.58), 1120);
        const height = Math.min(Math.round(stageHeight * 0.5), 620);
        glow.set_size(width, height);
        glow.set_position(
            Math.round((stageWidth - width) / 2),
            Math.round((stageHeight - height) * 0.56));
        host.insert_child_below(glow, clock);

        const breathe = new Clutter.PropertyTransition({
            property_name: 'opacity',
            duration: GLOW_HALF_CYCLE_MS,
            repeat_count: -1,
            auto_reverse: true,
            progress_mode: Clutter.AnimationMode.EASE_IN_OUT_SINE,
        });
        breathe.set_from(20);
        breathe.set_to(42);
        glow.add_transition('noor-ambient-breathe', breathe);
        this._glow = glow;
        this._state.track(glow);
    }

    _cancelClockEntrance() {
        if (this._clockDelayId) {
            GLib.Source.remove(this._clockDelayId);
            this._clockDelayId = 0;
        }
    }

    _removeGlow() {
        if (!this._glow)
            return;
        this._glow.remove_transition('noor-ambient-breathe');
        this._glow.destroy();
        this._glow = null;
    }

    _restoreClock() {
        if (this._clockSnapshot)
            restoreSnapshot(this._clockSnapshot);
    }

    _restoreAllSnapshots() {
        for (const snapshot of this._backgroundSnapshots)
            restoreSnapshot(snapshot);
        this._restoreClock();
    }
}
