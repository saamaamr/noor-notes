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
