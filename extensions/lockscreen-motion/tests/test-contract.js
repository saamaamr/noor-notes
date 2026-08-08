import GLib from 'gi://GLib';

function assertTrue(value, message) {
    if (!value)
        throw new Error(message);
}

function readText(path) {
    const [ok, bytes] = GLib.file_get_contents(path);
    if (!ok)
        throw new Error(`unable to read ${path}`);
    return new TextDecoder().decode(bytes);
}

const metadata = JSON.parse(readText('extensions/lockscreen-motion/metadata.json'));
const extension = readText('extensions/lockscreen-motion/extension.js');
const motion = readText('extensions/lockscreen-motion/motionSession.js');

assertTrue(metadata.uuid === 'noor-lockscreen-motion@saamaamr.github.io',
    'metadata must use the dedicated companion UUID');
assertTrue(metadata['shell-version'].includes('50'),
    'metadata must support GNOME Shell 50');
assertTrue(metadata['session-modes'].includes('user') &&
    metadata['session-modes'].includes('unlock-dialog'),
    'extension must be available in user and unlock-dialog modes');

for (const required of [
    "connect('active-changed'",
    'org.gnome.desktop.interface',
    'changed::enable-animations',
    'net.hadess.PowerProfiles',
    'ActiveProfile',
    'this._motion.stop()',
]) {
    assertTrue(extension.includes(required), `extension lifecycle is missing ${required}`);
}

for (const forbidden of [
    'EventControllerKey',
    'EventControllerMotion',
    "connect('new-frame'",
    'Soup.Session',
    'fetch(',
    'timeout_add_seconds',
]) {
    assertTrue(!extension.includes(forbidden) && !motion.includes(forbidden),
        `lock-screen motion must not use ${forbidden}`);
}
