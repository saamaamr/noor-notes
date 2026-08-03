#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
cd "$repo_root"

cargo test -p noor-sync --test offline --test conflicts --test remote_download
cargo test -p noor-crypto --test vectors

if rg -n 'offline edit|new from desktop|recoverable' supabase/migrations supabase/tests; then
    printf 'Known plaintext leaked into Supabase artifacts\n' >&2
    exit 1
fi

printf 'Two-device sync contracts and ciphertext-only checks passed.\n'
