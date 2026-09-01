#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
gallery="$repo_root/docs/images/1.1.3"
screenshots='noor-notes-editor.png
noor-notes-library.png
noor-notes-midnight.png
noor-notes-sticky-read-only.png'

for name in $screenshots; do
    image="$gallery/$name"
    test -s "$image" || {
        printf 'Missing release screenshot: %s\n' "$image" >&2
        exit 1
    }
    file "$image" | grep -Fq 'PNG image data, 1248 x 702' || {
        printf 'Release screenshot is not 1248 x 702: %s\n' "$image" >&2
        exit 1
    }
    test "$(stat -c %s "$image")" -lt 2097152 || {
        printf 'Release screenshot must stay below 2 MiB: %s\n' "$image" >&2
        exit 1
    }
done

actual=$(find "$gallery" -maxdepth 1 -type f -name '*.png' -printf '%f\n' | sort)
expected=$(printf '%s\n' "$screenshots" | sort)
if test "$actual" != "$expected"; then
    printf 'Release screenshot inventory differs from the documented gallery\n' >&2
    printf 'Expected:\n%s\nActual:\n%s\n' "$expected" "$actual" >&2
    exit 1
fi
