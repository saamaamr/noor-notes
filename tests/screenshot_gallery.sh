#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
gallery="$repo_root/data/screenshots"
index="$gallery/INDEX.md"

test -s "$index" || {
    printf 'Missing screenshot index: %s\n' "$index" >&2
    exit 1
}

indexed=$(sed -n 's/.*](\([^)]*\.png\)).*/\1/p' "$index" | sort -u)
test -n "$indexed" || {
    printf 'Screenshot index contains no PNG links\n' >&2
    exit 1
}

printf '%s\n' "$indexed" | while IFS= read -r relative; do
    image="$gallery/$relative"
    test -s "$image" || {
        printf 'Missing indexed screenshot: %s\n' "$relative" >&2
        exit 1
    }
    case "$relative" in
        contact-sheets/*) ;;
        *)
            file "$image" | grep -Fq 'PNG image data, 1248 x 702' || {
                printf 'Individual screenshot is not 1248 x 702: %s\n' "$relative" >&2
                exit 1
            }
            ;;
    esac
done

actual=$(find "$gallery" -type f -name '*.png' -printf '%P\n' | sort)
if test "$indexed" != "$actual"; then
    expected_file=$(mktemp)
    actual_file=$(mktemp)
    trap 'rm -f "$expected_file" "$actual_file"' EXIT HUP INT TERM
    printf '%s\n' "$indexed" > "$expected_file"
    printf '%s\n' "$actual" > "$actual_file"
    printf 'Screenshot index and PNG inventory differ\n' >&2
    diff -u "$expected_file" "$actual_file" >&2 || true
    exit 1
fi
