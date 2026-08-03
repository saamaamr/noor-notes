export const NOOR_APP_ID = 'io.github.saamaamr.NoorNotes';

export function authorizeWindow({appId, sender, owner, stale}) {
    return appId === NOOR_APP_ID && sender === owner && !stale;
}
