#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
metadata="$repo_root/data/io.github.saamaamr.NoorNotes.metainfo.xml"
desktop="$repo_root/data/io.github.saamaamr.NoorNotes.desktop"
editor_screenshot="$repo_root/data/screenshots/noor-notes-editor.png"
library_screenshot="$repo_root/data/screenshots/noor-notes-library.png"

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
require '<release version="0.1.0" date="2026-08-04" type="stable">' "$metadata"
require '<image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-editor.png</image>' "$metadata"
require '<image type="source" width="1248" height="702">https://raw.githubusercontent.com/saamaamr/noor-notes/main/data/screenshots/noor-notes-library.png</image>' "$metadata"
require '<caption>Browse notes in the library.</caption>' "$metadata"

require 'Exec=noor-notes' "$desktop"
require 'Icon=io.github.saamaamr.NoorNotes' "$desktop"
require 'Categories=Utility;Office;' "$desktop"

for screenshot in "$editor_screenshot" "$library_screenshot"; do
    if [ ! -s "$screenshot" ]; then
        printf 'Missing required screenshot: %s\n' "$screenshot" >&2
        exit 1
    fi
    if ! file "$screenshot" | grep -Fq 'PNG image data, 1248 x 702'; then
        printf 'Screenshot is not a 1248 x 702 PNG: %s\n' "$screenshot" >&2
        exit 1
    fi
done
