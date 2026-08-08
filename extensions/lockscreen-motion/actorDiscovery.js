function childrenOf(actor) {
    try {
        const children = actor?.get_children?.();
        return Array.isArray(children) ? children : [];
    } catch (_error) {
        return [];
    }
}

function styleClassesOf(actor) {
    try {
        return (actor?.get_style_class_name?.() ?? '')
            .split(/\s+/)
            .filter(Boolean);
    } catch (_error) {
        return [];
    }
}

export function findDescendantByStyleClass(root, className) {
    if (!root || typeof className !== 'string' || className.length === 0)
        return null;
    if (styleClassesOf(root).includes(className))
        return root;
    for (const child of childrenOf(root)) {
        const match = findDescendantByStyleClass(child, className);
        if (match)
            return match;
    }
    return null;
}

export function findClockActor(lockDialogGroup) {
    const time = findDescendantByStyleClass(lockDialogGroup, 'wack-time');
    if (!time)
        return null;

    let actor = time;
    while (actor) {
        let parent = null;
        try {
            parent = actor.get_parent?.() ?? null;
        } catch (_error) {
            return null;
        }
        if (parent === lockDialogGroup)
            return actor;
        actor = parent;
    }
    return null;
}

export function discoverActors({screenShield} = {}) {
    const dialog = screenShield?._dialog;
    const host = screenShield?._lockDialogGroup;
    if (!dialog || !host)
        return null;

    let backgrounds;
    try {
        backgrounds = [...(dialog._backgroundGroup ?? [])].filter(Boolean);
    } catch (_error) {
        return null;
    }
    const clock = findClockActor(host);
    if (backgrounds.length === 0 || !clock)
        return null;

    return {backgrounds, clock, host};
}
