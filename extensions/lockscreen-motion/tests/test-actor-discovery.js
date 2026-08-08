import {
    discoverActors,
    findClockActor,
    findDescendantByStyleClass,
} from '../actorDiscovery.js';

function assertSame(actual, expected, message) {
    if (actual !== expected)
        throw new Error(message);
}

function assertEqual(actual, expected, message) {
    if (JSON.stringify(actual) !== JSON.stringify(expected))
        throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

function node(styleClass = '', children = []) {
    const actor = {
        _parent: null,
        get_children: () => children,
        get_parent: () => actor._parent,
        get_style_class_name: () => styleClass,
    };
    for (const child of children)
        child._parent = actor;
    return actor;
}

const time = node('unlock-dialog-clock-time wack-time');
const labels = node('', [time]);
const clock = node('', [labels]);
const unrelated = node('other-widget');
const host = node('', [unrelated, clock]);
const backgrounds = [node('background-one'), node('background-two')];
const screenShield = {
    _dialog: {_backgroundGroup: backgrounds},
    _lockDialogGroup: host,
};

assertSame(findDescendantByStyleClass(host, 'wack-time'), time,
    'recursive discovery should find the WACK time label');
assertSame(findClockActor(host), clock,
    'clock discovery should return the direct host child containing WACK time');
assertSame(findClockActor(node('', [])), null,
    'missing WACK time should safely return null');
assertSame(discoverActors({screenShield: null}), null,
    'missing screen shield should safely return null');
assertSame(discoverActors({screenShield: {_dialog: {}, _lockDialogGroup: host}}), null,
    'missing backgrounds should safely return null');

const discovered = discoverActors({screenShield});
assertEqual(discovered.backgrounds, backgrounds,
    'all background actors should be returned');
assertSame(discovered.clock, clock, 'the discovered clock should match');
assertSame(discovered.host, host, 'the lock-dialog group should host the glow');
