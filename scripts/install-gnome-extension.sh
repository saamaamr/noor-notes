#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
uuid="noor-notes-windowing@saamaamr.github.io"
data_root=${XDG_DATA_HOME:-"$HOME/.local/share"}
target="$data_root/gnome-shell/extensions/$uuid"

mkdir -p "$target"
for file in metadata.json extension.js policy.js dbus.xml stylesheet.css; do
    install -m 0644 "$repo_root/extensions/gnome/$file" "$target/$file"
done

if command -v gnome-extensions >/dev/null 2>&1; then
    gnome-extensions enable "$uuid" || true
fi

printf 'Installed GNOME extension at %s\n' "$target"
printf 'If the pin control stays disabled, log out and back in once.\n'
