#!/bin/sh
set -eu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
if rg -n --hidden --glob '!target/**' --glob '!.git/**' -- '-----BEGIN (RSA|OPENSSH|EC) PRIVATE KEY-----|sbp_[A-Za-z0-9]{20,}' .; then
    echo 'Potential committed secret detected' >&2
    exit 1
fi
bash tests/snap_manifest.sh
bash tests/release_metadata.sh
bash tests/snap_cadence_workflow.sh
bash tests/flatpak_manifest.sh
gjs -m extensions/gnome/tests/test-policy.js
cargo cyclonedx --format json --all
