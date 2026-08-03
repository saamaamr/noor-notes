import {authorizeWindow} from '../policy.js';

function assertFalse(value, message) {
    if (value)
        throw new Error(message);
}

function assertTrue(value, message) {
    if (!value)
        throw new Error(message);
}

const valid = {
    appId: 'io.github.saamaamr.NoorNotes',
    sender: ':1.42',
    owner: ':1.42',
    stale: false,
};

assertTrue(authorizeWindow(valid), 'Noor Notes owner should be authorized');
assertFalse(authorizeWindow({...valid, appId: 'org.example.Other'}), 'other apps must fail');
assertFalse(authorizeWindow({...valid, sender: ':1.99'}), 'other bus names must fail');
assertFalse(authorizeWindow({...valid, stale: true}), 'stale windows must fail');
