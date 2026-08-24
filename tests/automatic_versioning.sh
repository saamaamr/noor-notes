#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
planner="$repo_root/scripts/next-snap-version.py"
applier="$repo_root/scripts/set-build-version.py"

expect_version() {
    kind=$1
    current=$2
    expected=$3
    actual=$(python3 "$planner" "$kind" "$current")
    if [ "$actual" != "$expected" ]; then
        printf '%s from %s: expected %s, got %s\n' "$kind" "$current" "$expected" "$actual" >&2
        exit 1
    fi
}

expect_version edge 1.0.0 1.0.1
expect_version edge 1.1.9 1.1.10
expect_version hotfix 1.1.0 1.1.1
expect_version hotfix 1.1.9 1.1.10
expect_version stable 1.0.7 1.1.0
expect_version stable 1.9.4 1.10.0

if python3 "$planner" edge 1.0 >/dev/null 2>&1; then
    printf 'Invalid Store version must be rejected\n' >&2
    exit 1
fi

if python3 "$planner" beta 1.0.0 >/dev/null 2>&1; then
    printf 'Unknown release kind must be rejected\n' >&2
    exit 1
fi

fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT HUP INT TERM
mkdir -p "$fixture/apps/noor-notes" "$fixture/crates" "$fixture/data"
cp "$repo_root/snapcraft.yaml" "$repo_root/Cargo.lock" "$fixture/"
cp "$repo_root/apps/noor-notes/Cargo.toml" "$fixture/apps/noor-notes/"
for crate in crypto domain storage sync windowing xpad-import; do
    mkdir -p "$fixture/crates/$crate"
    cp "$repo_root/crates/$crate/Cargo.toml" "$fixture/crates/$crate/"
done
cp "$repo_root/data/io.github.saamaamr.NoorNotes.metainfo.xml" "$fixture/data/"

python3 "$applier" 1.4.5 --root "$fixture" --date 2026-08-19
python3 - "$fixture" <<'PY'
import sys
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path


root = Path(sys.argv[1])
expected = "1.4.5"
manifests = sorted(root.glob("crates/*/Cargo.toml")) + [root / "apps/noor-notes/Cargo.toml"]
for path in manifests:
    manifest = tomllib.loads(path.read_text(encoding="utf-8"))
    if manifest["package"]["version"] != expected:
        raise SystemExit(f"{path} was not updated to {expected}")

workspace_names = {
    "noor-crypto",
    "noor-domain",
    "noor-notes",
    "noor-storage",
    "noor-sync",
    "noor-windowing",
    "noor-xpad-import",
}
lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
actual = {
    package["name"]: package["version"]
    for package in lock["package"]
    if package["name"] in workspace_names
}
if actual != {name: expected for name in workspace_names}:
    raise SystemExit(f"Cargo.lock workspace versions were not synchronized: {actual}")

snapcraft = (root / "snapcraft.yaml").read_text(encoding="utf-8")
if f'version: "{expected}"' not in snapcraft:
    raise SystemExit("snapcraft.yaml version was not updated")

release = ET.parse(root / "data/io.github.saamaamr.NoorNotes.metainfo.xml").getroot().find("./releases/release")
if release is None or release.get("version") != expected or release.get("date") != "2026-08-19":
    raise SystemExit("AppStream release version/date were not updated")
PY
