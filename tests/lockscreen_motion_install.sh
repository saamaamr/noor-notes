#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
test_root=$(mktemp -d)
trap 'rm -rf -- "$test_root"' EXIT

test_bin="$test_root/bin"
data_root="$test_root/data"
enable_log="$test_root/enabled-extensions"
fallback_log="$test_root/queued-extensions"
mkdir -p "$test_bin" "$data_root"

printf '%s\n' '#!/usr/bin/env bash' \
    'printf "%s\n" "$*" >> "${NOOR_ENABLE_LOG:?}"' \
    '[[ "$*" != *"noor-lockscreen-motion@saamaamr.github.io"* ]]' \
    >"$test_bin/gnome-extensions"
chmod +x "$test_bin/gnome-extensions"
printf '%s\n' '#!/usr/bin/env bash' \
    'printf "%s\n" "${NOOR_EXTENSION_UUID:?}" >> "${NOOR_FALLBACK_LOG:?}"' \
    >"$test_bin/gjs"
chmod +x "$test_bin/gjs"

installer_output=$(XDG_DATA_HOME="$data_root" \
    NOOR_ENABLE_LOG="$enable_log" \
    NOOR_FALLBACK_LOG="$fallback_log" \
    PATH="$test_bin:/usr/bin:/bin" \
    /bin/bash "$repo_root/scripts/install-gnome-extension.sh")

windowing_uuid='noor-notes-windowing@saamaamr.github.io'
motion_uuid='noor-lockscreen-motion@saamaamr.github.io'
extension_root="$data_root/gnome-shell/extensions"

for file in metadata.json extension.js policy.js dbus.xml stylesheet.css; do
    test -f "$extension_root/$windowing_uuid/$file"
done

for file in metadata.json extension.js policy.js actorDiscovery.js motionSession.js stylesheet.css; do
    test -f "$extension_root/$motion_uuid/$file"
done

grep -Fxq "enable $windowing_uuid" "$enable_log"
grep -Fxq "enable $motion_uuid" "$enable_log"
grep -Fxq "$motion_uuid" "$fallback_log"
grep -Fiq 'log out and back in once' <<<"$installer_output"
grep -Fq 'Super+L' <<<"$installer_output"

if find "$test_root" -iname '*wack*' -print -quit | grep -q .; then
    printf 'Installer must not create or modify a WACK extension path\n' >&2
    exit 1
fi

printf 'GNOME extension installer contract checks passed.\n'
