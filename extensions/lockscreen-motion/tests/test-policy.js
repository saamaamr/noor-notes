import {motionPlan} from '../policy.js';

function assertEqual(actual, expected, message) {
    if (JSON.stringify(actual) !== JSON.stringify(expected))
        throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

const full = {
    wallpaperFade: true,
    wallpaperScale: true,
    clockEntrance: true,
    ambientGlow: true,
};
const fadeOnly = {
    wallpaperFade: true,
    wallpaperScale: false,
    clockEntrance: false,
    ambientGlow: false,
};
const disabled = {
    wallpaperFade: false,
    wallpaperScale: false,
    clockEntrance: false,
    ambientGlow: false,
};

assertEqual(motionPlan({
    lockActive: true,
    animationsEnabled: true,
    powerProfile: 'balanced',
    actorsReady: true,
}), full, 'balanced mode should enable restrained full motion');

assertEqual(motionPlan({
    lockActive: true,
    animationsEnabled: true,
    powerProfile: 'power-saver',
    actorsReady: true,
}), fadeOnly, 'Power Saver should keep only the one-time fade');

for (const state of [
    {lockActive: false, animationsEnabled: true, powerProfile: 'balanced', actorsReady: true},
    {lockActive: true, animationsEnabled: false, powerProfile: 'balanced', actorsReady: true},
    {lockActive: true, animationsEnabled: true, powerProfile: 'balanced', actorsReady: false},
]) {
    assertEqual(motionPlan(state), disabled, 'inactive, reduced-motion, or missing actors should disable motion');
}
