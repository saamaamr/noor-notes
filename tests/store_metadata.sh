#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
metadata="$repo_root/data/io.github.saamaamr.NoorNotes.metainfo.xml"
desktop="$repo_root/data/io.github.saamaamr.NoorNotes.desktop"
listing="$repo_root/data/store/LISTING.md"
banner="$repo_root/data/store/noor-notes-featured-banner.png"
screenshots='noor-notes-editor.png
noor-notes-library.png
noor-notes-writing-assistance.png
noor-notes-dark.png
noor-notes-formatting.png
noor-notes-find-replace.png
noor-notes-trash.png
noor-notes-responsive.png'

require() {
    needle=$1
    path=$2
    if ! grep -Fq -- "$needle" "$path"; then
        printf 'Missing required metadata: %s in %s\n' "$needle" "$path" >&2
        exit 1
    fi
}

require '<id>io.github.saamaamr.NoorNotes</id>' "$metadata"
require '<launchable type="desktop-id">io.github.saamaamr.NoorNotes.desktop</launchable>' "$metadata"
require '<developer id="io.github.saamaamr">' "$metadata"
require '<name>Abdullah Al Mamun</name>' "$metadata"
require '<url type="homepage">https://github.com/saamaamr/noor-notes</url>' "$metadata"
require '<url type="bugtracker">https://github.com/saamaamr/noor-notes/issues</url>' "$metadata"
require '<url type="vcs-browser">https://github.com/saamaamr/noor-notes</url>' "$metadata"
require '<summary>Private, encrypted notes with focused editing for Linux</summary>' "$metadata"
require '<release version="1.0.0" date="2026-08-19" type="stable">' "$metadata"
require '<release version="0.2.0" date="2026-08-17" type="stable">' "$metadata"
require '<release version="0.1.1" date="2026-08-05" type="stable">' "$metadata"
require '<caption>Browse and preview private notes.</caption>' "$metadata"
require '<caption>Control private local and optional online writing assistance.</caption>' "$metadata"

require 'Exec=noor-notes' "$desktop"
require 'Icon=io.github.saamaamr.NoorNotes' "$desktop"
require 'Categories=Utility;Office;' "$desktop"
require 'Private, encrypted notes with focused editing for Linux' "$listing"
require 'GPL-3.0-or-later' "$listing"
require 'data/io.github.saamaamr.NoorNotes.svg' "$listing"

if ! file "$banner" | grep -Fq 'PNG image data, 1920 x 640, 8-bit/color RGB'; then
    printf 'Featured banner is not a 1920 x 640 RGB PNG: %s\n' "$banner" >&2
    exit 1
fi
if [ "$(stat -c %s "$banner")" -ge 2097152 ]; then
    printf 'Featured banner must stay below 2 MiB: %s\n' "$banner" >&2
    exit 1
fi

for name in $screenshots; do
    require "https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/$name" "$metadata"
    screenshot="$repo_root/data/screenshots/$name"
    if [ ! -s "$screenshot" ]; then
        printf 'Missing required screenshot: %s\n' "$screenshot" >&2
        exit 1
    fi
    if ! file "$screenshot" | grep -Fq 'PNG image data, 1248 x 702'; then
        printf 'Screenshot is not a 1248 x 702 PNG: %s\n' "$screenshot" >&2
        exit 1
    fi
    if [ "$(stat -c %s "$screenshot")" -ge 2097152 ]; then
        printf 'Screenshot must stay below 2 MiB: %s\n' "$screenshot" >&2
        exit 1
    fi
done
