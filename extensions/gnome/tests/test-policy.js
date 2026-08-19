import {authorizeRequest, authorizeWindow} from '../policy.js';

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
const development = {
    ...valid,
    appId: 'io.github.saamaamr.NoorNotes.Devel',
    sender: ':1.84',
    owner: ':1.84',
};
assertTrue(authorizeWindow(development), 'Noor Notes Dev owner should be authorized');
assertFalse(
    authorizeWindow({...development, owner: valid.owner}),
    'Noor Notes Dev requests must not inherit the Store app owner');
assertTrue(authorizeRequest({...valid, method: 'SetAbove', windowId: 'Noor Note::018f2f91-8d87-7c4a-a9ee-9b90518f4123', enabled: true}), 'valid request should pass');
assertFalse(authorizeRequest({...valid, method: 'Delete', windowId: 'Noor Note::018f2f91-8d87-7c4a-a9ee-9b90518f4123', enabled: true}), 'unexpected methods must fail');
assertFalse(authorizeRequest({...valid, method: 'SetAbove', windowId: 'spoofed title', enabled: true}), 'spoofed titles must fail');
assertFalse(authorizeRequest({...valid, method: 'SetAbove', windowId: 'Noor Note::018f2f91-8d87-7c4a-a9ee-9b90518f4123', enabled: 'true'}), 'non-boolean values must fail');
