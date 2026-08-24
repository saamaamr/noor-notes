#!/bin/sh

set -eu

if [ "$#" -ne 1 ]; then
    printf 'Usage: %s <snap-file-or-mounted-root>\n' "$0" >&2
    exit 2
fi

artifact=$1
cleanup=
if [ -d "$artifact" ]; then
    root=$artifact
else
    if ! command -v unsquashfs >/dev/null 2>&1; then
        printf 'unsquashfs is required to inspect %s\n' "$artifact" >&2
        exit 2
    fi
    root=$(mktemp -d)
    cleanup=$root
    trap 'rm -rf "$cleanup"' EXIT HUP INT TERM
    unsquashfs -quiet -no-progress -d "$root" "$artifact"
fi

find_bundled() {
    name=$1
    find "$root/usr/lib" -type f -o -type l 2>/dev/null \
        | grep -E "/${name}(\.|$)" \
        | head -n 1
}

for platform_library in libgtk-4.so libadwaita-1.so libgtksourceview-5.so; do
    bundled=$(find_bundled "$platform_library" || true)
    if [ -n "$bundled" ]; then
        printf 'Snap must use the GNOME content runtime for %s; bundled copy found: %s\n' \
            "$platform_library" "$bundled" >&2
        exit 1
    fi
done

if find "$root/usr/lib" -type d -path '*/gtk-4.0' -print -quit 2>/dev/null | grep -q .; then
    printf 'Snap must not bundle GTK 4 modules from the core24 archive\n' >&2
    exit 1
fi

if ! find_bundled libspelling-1.so >/dev/null; then
    printf 'Snap must bundle libspelling because the GNOME 46 content runtime does not provide it\n' >&2
    exit 1
fi

printf 'Snap runtime contract passed: GNOME owns GTK/libadwaita/GtkSourceView; app owns libspelling\n'
