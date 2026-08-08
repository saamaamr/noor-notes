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

export class SessionState {
    constructor() {
        this._activationKey = null;
        this._resources = new Set();
    }

    begin(activationKey) {
        if (activationKey === this._activationKey)
            return false;
        this._activationKey = activationKey;
        this._resources.clear();
        return true;
    }

    track(resourceKey) {
        this._resources.add(resourceKey);
    }

    clear() {
        const resources = [...this._resources];
        this._resources.clear();
        this._activationKey = null;
        return resources;
    }
}
