#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
installer="$repo_root/scripts/install-ubuntu.sh"

help_output=$(/bin/bash "$installer" --help 2>&1)
grep -Fq 'Install Noor Notes on Ubuntu' <<<"$help_output"
grep -Fq 'system dependencies' <<<"$help_output"
grep -Fq 'scripts/install-local.sh' <<<"$help_output"

if unsupported_output=$(PATH=/nonexistent /bin/bash "$installer" 2>&1); then
    printf 'Expected installer to reject a host without apt-get\n' >&2
    exit 1
fi
grep -Fq 'This installer supports Ubuntu and other APT-based systems only.' \
    <<<"$unsupported_output"

/bin/bash "$repo_root/tests/lockscreen_motion_install.sh"

for package in libspelling-1-dev enchant-2 hunspell-en-us; do
    grep -Fq "$package" "$installer" || {
        printf 'Ubuntu installer must install %s\n' "$package" >&2
        exit 1
    }
done

printf 'Ubuntu installer contract checks passed.\n'
