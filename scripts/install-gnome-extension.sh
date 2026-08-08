#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
data_root=${XDG_DATA_HOME:-"$HOME/.local/share"}
extension_root="$data_root/gnome-shell/extensions"
windowing_uuid="noor-notes-windowing@saamaamr.github.io"
motion_uuid="noor-lockscreen-motion@saamaamr.github.io"

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
        gnome-extensions enable "$uuid" || true
    fi

    printf 'Installed GNOME extension at %s\n' "$target"
}

install_extension "$repo_root/extensions/gnome" "$windowing_uuid" \
    metadata.json extension.js policy.js dbus.xml stylesheet.css

install_extension "$repo_root/extensions/lockscreen-motion" "$motion_uuid" \
    metadata.json extension.js policy.js actorDiscovery.js motionSession.js stylesheet.css

printf 'Log out and back in once so GNOME Shell loads newly installed extensions.\n'
printf 'Then press Super+L to verify the lightweight lock-screen motion.\n'
