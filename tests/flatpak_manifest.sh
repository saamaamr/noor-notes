#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
manifest="$repo_root/packaging/flatpak/io.github.saamaamr.NoorNotes.yml"
cargo_sources="$repo_root/packaging/flatpak/cargo-sources.json"

if [ ! -f "$manifest" ]; then
    printf 'Missing Flatpak manifest: %s\n' "$manifest" >&2
    exit 1
fi

python3 - "$manifest" "$cargo_sources" <<'PY'
import json
import re
import sys
from pathlib import Path

import yaml


def require_equal(path, actual, expected):
    if actual != expected:
        raise SystemExit(f"{path} must equal {expected!r}; got {actual!r}")


manifest_path = Path(sys.argv[1])
cargo_sources_path = Path(sys.argv[2])
manifest = yaml.safe_load(manifest_path.read_text(encoding="utf-8"))

if not isinstance(manifest, dict):
    raise SystemExit("Flatpak manifest must be a YAML mapping")

require_equal("app-id", manifest.get("app-id"), "io.github.saamaamr.NoorNotes")
require_equal("runtime", manifest.get("runtime"), "org.gnome.Platform")
require_equal("sdk", manifest.get("sdk"), "org.gnome.Sdk")
require_equal("command", manifest.get("command"), "noor-notes")
require_equal(
    "finish-args",
    manifest.get("finish-args"),
    [
        "--share=ipc",
        "--share=network",
        "--socket=fallback-x11",
        "--socket=wayland",
        "--talk-name=org.freedesktop.secrets",
    ],
)

modules = manifest.get("modules")
if not isinstance(modules, list) or len(modules) != 1 or not isinstance(modules[0], dict):
    raise SystemExit("Flatpak manifest must contain exactly one application module")

module = modules[0]
require_equal("modules[0].name", module.get("name"), "noor-notes")
sources = module.get("sources")
if not isinstance(sources, list) or not sources:
    raise SystemExit("Flatpak application module must declare sources")

if any(isinstance(source, dict) and source.get("type") == "dir" for source in sources):
    raise SystemExit("Flatpak sources must not use the development-only type: dir source")

git_sources = [source for source in sources if isinstance(source, dict) and source.get("type") == "git"]
if len(git_sources) != 1:
    raise SystemExit("Flatpak application module must have exactly one immutable git source")

git_source = git_sources[0]
url = git_source.get("url")
commit = git_source.get("commit")
if not isinstance(url, str) or not url.startswith("https://github.com/saamaamr/noor-notes"):
    raise SystemExit("Flatpak git source must use the public Noor Notes GitHub URL")
if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}", commit):
    raise SystemExit("Flatpak git source must pin a full immutable 40-character commit")

if "cargo-sources.json" not in sources:
    raise SystemExit("Flatpak application module must include generated cargo-sources.json")

for installed_path in (
    "/app/bin/noor-notes",
    "/app/share/applications/io.github.saamaamr.NoorNotes.desktop",
    "/app/share/metainfo/io.github.saamaamr.NoorNotes.metainfo.xml",
    "/app/share/icons/hicolor/scalable/apps/io.github.saamaamr.NoorNotes.svg",
):
    if not any(installed_path in command for command in module.get("build-commands", [])):
        raise SystemExit(f"Flatpak build must install {installed_path}")

if not cargo_sources_path.is_file():
    raise SystemExit(f"Missing generated Cargo sources: {cargo_sources_path}")

try:
    cargo_sources = json.loads(cargo_sources_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as error:
    raise SystemExit(f"Generated Cargo sources must be valid JSON: {error}") from error

if not isinstance(cargo_sources, list) or not cargo_sources:
    raise SystemExit("Generated Cargo sources must be a non-empty source list")

if not any(
    isinstance(source, dict)
    and source.get("type") == "inline"
    and source.get("dest") == "cargo"
    and source.get("dest-filename") == "config"
    for source in cargo_sources
):
    raise SystemExit("Generated Cargo sources must configure the vendored Cargo registry")
PY
