#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
data_root=${XDG_DATA_HOME:-"$HOME/.local/share"}
extension_root="$data_root/gnome-shell/extensions"
windowing_uuid="noor-notes-windowing@saamaamr.github.io"
motion_uuid="noor-lockscreen-motion@saamaamr.github.io"

queue_extension_for_next_login() {
    local uuid=$1
    command -v gjs >/dev/null 2>&1 || return 1

    NOOR_EXTENSION_UUID="$uuid" gjs -c '
        const {Gio, GLib} = imports.gi;
        const uuid = GLib.getenv("NOOR_EXTENSION_UUID");
        const settings = new Gio.Settings({schema_id: "org.gnome.shell"});
        const enabled = settings.get_strv("enabled-extensions");
        if (!enabled.includes(uuid)) {
            enabled.push(uuid);
            if (!settings.set_strv("enabled-extensions", enabled))
                imports.system.exit(1);
            Gio.Settings.sync();
        }
    '
}

install_extension() {
    local source_dir=$1
    local uuid=$2
    shift 2
    local target="$extension_root/$uuid"

    mkdir -p "$target"
    for file in "$@"; do
        install -m 0644 "$source_dir/$file" "$target/$file"
    done

    if command -v gnome-extensions >/dev/null 2>&1; then
        if ! gnome-extensions enable "$uuid"; then
            queue_extension_for_next_login "$uuid" || \
                printf 'Enable %s after logging in again.\n' "$uuid" >&2
        fi
    fi

    printf 'Installed GNOME extension at %s\n' "$target"
}

install_extension "$repo_root/extensions/gnome" "$windowing_uuid" \
    metadata.json extension.js policy.js dbus.xml stylesheet.css

install_extension "$repo_root/extensions/lockscreen-motion" "$motion_uuid" \
    metadata.json extension.js policy.js actorDiscovery.js motionSession.js stylesheet.css

printf 'Log out and back in once so GNOME Shell loads newly installed extensions.\n'
printf 'Then press Super+L to verify the lightweight lock-screen motion.\n'
