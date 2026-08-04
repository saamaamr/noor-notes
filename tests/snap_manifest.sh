#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
manifest="$repo_root/snap/snapcraft.yaml"

require() {
    needle=$1
    if ! grep -Fq -- "$needle" "$manifest"; then
        printf 'Missing required Snap manifest setting: %s\n' "$needle" >&2
        exit 1
    fi
}

if [ ! -f "$manifest" ]; then
    printf 'Missing Snap manifest: %s\n' "$manifest" >&2
    exit 1
fi

require 'base: core24'
require 'confinement: strict'
require 'platforms:'
require '  amd64:'
require 'extensions: [gnome]'
require '    - wayland'
require '    - x11'
require '    - network'
require '    - desktop'
require '    - password-manager-service'
require 'desktop: usr/share/applications/io.github.saamaamr.NoorNotes.desktop'
require 'target/release/noor-notes'
require 'data/io.github.saamaamr.NoorNotes.desktop'
require 'data/io.github.saamaamr.NoorNotes.metainfo.xml'
require 'data/io.github.saamaamr.NoorNotes.svg'

if grep -Fq -- 'extensions/gnome' "$manifest"; then
    printf 'The Snap manifest must not bundle the GNOME Shell extension.\n' >&2
    exit 1
fi
