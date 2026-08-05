export const NOOR_APP_ID = 'io.github.saamaamr.NoorNotes';

export function authorizeWindow({appId, sender, owner, stale}) {
    return appId === NOOR_APP_ID && sender === owner && !stale;
}

const METHODS = new Set(['SetAbove', 'SetAllWorkspaces']);
const WINDOW_ID = /^Noor Note::[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function authorizeRequest(request) {
    return authorizeWindow(request)
        && METHODS.has(request.method)
        && typeof request.enabled === 'boolean'
        && typeof request.windowId === 'string'
        && WINDOW_ID.test(request.windowId);
}
