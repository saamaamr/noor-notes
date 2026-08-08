import {SessionState} from '../policy.js';

function assertTrue(value, message) {
    if (!value)
        throw new Error(message);
}

function assertFalse(value, message) {
    if (value)
        throw new Error(message);
}

function assertEqual(actual, expected, message) {
    if (JSON.stringify(actual) !== JSON.stringify(expected))
        throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

const state = new SessionState();
assertTrue(state.begin('lock-1'), 'first activation should begin');
assertFalse(state.begin('lock-1'), 'duplicate activation should be ignored');
state.track('background-a');
state.track('glow');
state.track('glow');
assertEqual(state.clear().sort(), ['background-a', 'glow'],
    'cleanup should return each tracked resource once');
assertEqual(state.clear(), [], 'repeated cleanup should be harmless');
assertTrue(state.begin('lock-1'), 'cleanup should allow a later lock cycle');
assertTrue(state.begin('lock-2'), 'a new activation key should replace the old cycle');
