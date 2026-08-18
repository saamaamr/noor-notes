#!/bin/sh

set -eu

binary=${1:-target/release/noor-notes}
maximum_bytes=15000000

if [ ! -x "$binary" ]; then
    printf 'Missing executable release binary: %s\n' "$binary" >&2
    exit 1
fi

actual_bytes=$(wc -c < "$binary")
if [ "$actual_bytes" -gt "$maximum_bytes" ]; then
    printf 'Release binary is %s bytes; expected no more than %s bytes\n' \
        "$actual_bytes" "$maximum_bytes" >&2
    exit 1
fi

if file "$binary" | grep -q 'not stripped'; then
    printf 'Release binary still contains removable symbol tables\n' >&2
    exit 1
fi

printf 'Release binary size: %s bytes (limit: %s)\n' "$actual_bytes" "$maximum_bytes"
