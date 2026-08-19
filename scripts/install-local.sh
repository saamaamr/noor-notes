#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
bin_root=${XDG_BIN_HOME:-"$HOME/.local/bin"}
data_root=${XDG_DATA_HOME:-"$HOME/.local/share"}
target_root=${CARGO_TARGET_DIR:-"$repo_root/target"}

cd "$repo_root"
cargo build --release -p noor-notes --features development
install -Dm755 "$target_root/release/noor-notes" "$bin_root/noor-notes-dev"
install -Dm644 data/io.github.saamaamr.NoorNotes.Devel.desktop \
    "$data_root/applications/io.github.saamaamr.NoorNotes.Devel.desktop"
install -Dm644 data/io.github.saamaamr.NoorNotes.svg \
    "$data_root/icons/hicolor/scalable/apps/io.github.saamaamr.NoorNotes.svg"
"$repo_root/scripts/install-gnome-extension.sh"

rm -f "$bin_root/noor-notes"
rm -f "$data_root/applications/io.github.saamaamr.NoorNotes.desktop"
rm -f "$data_root/metainfo/io.github.saamaamr.NoorNotes.metainfo.xml"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$data_root/applications" || true
fi

printf 'Noor Notes Dev installed. Launch it from Applications or run %s/noor-notes-dev\n' "$bin_root"
