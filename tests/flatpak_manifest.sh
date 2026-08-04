#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
manifest="$repo_root/packaging/flatpak/io.github.saamaamr.NoorNotes.yml"
cargo_sources="$repo_root/packaging/flatpak/cargo-sources.json"
cargo_lock="$repo_root/Cargo.lock"
license_file="$repo_root/LICENSE"

if [ ! -f "$manifest" ]; then
    printf 'Missing Flatpak manifest: %s\n' "$manifest" >&2
    exit 1
fi

python3 - "$manifest" "$cargo_sources" "$cargo_lock" "$license_file" <<'PY'
import json
import hashlib
import re
import sys
import tomllib
from pathlib import Path

import yaml


def require_equal(path, actual, expected):
    if actual != expected:
        raise SystemExit(f"{path} must equal {expected!r}; got {actual!r}")


manifest_path = Path(sys.argv[1])
cargo_sources_path = Path(sys.argv[2])
cargo_lock_path = Path(sys.argv[3])
license_file_path = Path(sys.argv[4])
manifest = yaml.safe_load(manifest_path.read_text(encoding="utf-8"))

if not isinstance(manifest, dict):
    raise SystemExit("Flatpak manifest must be a YAML mapping")

require_equal("app-id", manifest.get("app-id"), "io.github.saamaamr.NoorNotes")
require_equal("runtime", manifest.get("runtime"), "org.gnome.Platform")
require_equal("runtime-version", manifest.get("runtime-version"), "50")
require_equal("sdk", manifest.get("sdk"), "org.gnome.Sdk")
require_equal(
    "sdk-extensions",
    manifest.get("sdk-extensions"),
    ["org.freedesktop.Sdk.Extension.rust-stable"],
)
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
require_equal(
    "modules[0].build-options.append-path",
    module.get("build-options", {}).get("append-path"),
    "/usr/lib/sdk/rust-stable/bin",
)
require_equal(
    "modules[0].build-options.env.CARGO_HOME",
    module.get("build-options", {}).get("env", {}).get("CARGO_HOME"),
    "/run/build/noor-notes/cargo",
)
if "cargo build --frozen --offline --release --package noor-notes" not in module.get("build-commands", []):
    raise SystemExit("Flatpak build must use Cargo's frozen offline mode")
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

if {
    "type": "file",
    "path": "../../LICENSE",
    "dest-filename": "LICENSE",
} not in sources:
    raise SystemExit("Flatpak application module must supply the canonical local LICENSE source")

for installed_path in (
    "/app/bin/noor-notes",
    "/app/share/applications/io.github.saamaamr.NoorNotes.desktop",
    "/app/share/metainfo/io.github.saamaamr.NoorNotes.metainfo.xml",
    "/app/share/icons/hicolor/scalable/apps/io.github.saamaamr.NoorNotes.svg",
    "/app/share/licenses/io.github.saamaamr.NoorNotes/noor-notes/GPL-3.0-or-later.txt",
):
    if not any(installed_path in command for command in module.get("build-commands", [])):
        raise SystemExit(f"Flatpak build must install {installed_path}")

if not cargo_sources_path.is_file():
    raise SystemExit(f"Missing generated Cargo sources: {cargo_sources_path}")

if not license_file_path.is_file():
    raise SystemExit(f"Missing canonical GPL-3.0-or-later license: {license_file_path}")

if hashlib.sha256(license_file_path.read_bytes()).hexdigest() != "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986":
    raise SystemExit("LICENSE must contain the canonical GPL-3.0 text")

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

lock = tomllib.loads(cargo_lock_path.read_text(encoding="utf-8"))
registry_packages = {}
for package in lock.get("package", []):
    source = package.get("source")
    if source is None:
        continue
    if not source.startswith("registry+"):
        raise SystemExit(f"Cargo.lock has unsupported non-registry source: {source}")
    key = (package["name"], package["version"])
    if key in registry_packages:
        raise SystemExit(f"Cargo.lock has duplicate registry package: {key}")
    registry_packages[key] = package.get("checksum")

if not registry_packages or any(not checksum for checksum in registry_packages.values()):
    raise SystemExit("Every registry package in Cargo.lock must have a checksum")

archive_sources = [source for source in cargo_sources if source.get("type") == "archive"]
if len(archive_sources) != len(registry_packages):
    raise SystemExit("Cargo sources must contain one archive for every registry package")

archive_packages = {}
for source in archive_sources:
    match = re.fullmatch(
        r"https://static\.crates\.io/crates/([^/]+)/\1-(.+)\.crate",
        source.get("url", ""),
    )
    if match is None:
        raise SystemExit(f"Cargo archive has an unexpected URL: {source.get('url')!r}")
    key = (match.group(1), match.group(2))
    if key in archive_packages:
        raise SystemExit(f"Cargo sources has duplicate archive: {key}")
    archive_packages[key] = source

if set(archive_packages) != set(registry_packages):
    missing = sorted(set(registry_packages) - set(archive_packages))
    extra = sorted(set(archive_packages) - set(registry_packages))
    raise SystemExit(f"Cargo archive set differs from Cargo.lock; missing={missing}, extra={extra}")

for key, checksum in registry_packages.items():
    source = archive_packages[key]
    require_equal(f"Cargo archive checksum for {key}", source.get("sha256"), checksum)
    require_equal(f"Cargo archive type for {key}", source.get("archive-type"), "tar-gzip")
    require_equal(f"Cargo archive destination for {key}", source.get("dest"), f"cargo/vendor/{key[0]}-{key[1]}")

checksum_sources = [
    source
    for source in cargo_sources
    if source.get("type") == "inline" and source.get("dest-filename") == ".cargo-checksum.json"
]
if len(checksum_sources) != len(registry_packages):
    raise SystemExit("Cargo sources must contain one checksum file for every registry package")

checksum_packages = {}
expected_destinations = {
    f"cargo/vendor/{name}-{version}": (name, version)
    for name, version in registry_packages
}
for source in checksum_sources:
    destination = source.get("dest")
    key = expected_destinations.get(destination)
    if key is None:
        raise SystemExit(f"Cargo checksum file has an unexpected destination: {source.get('dest')!r}")
    if key in checksum_packages:
        raise SystemExit(f"Cargo sources has duplicate checksum file: {key}")
    checksum_packages[key] = json.loads(source["contents"]).get("package")

if checksum_packages != registry_packages:
    raise SystemExit("Cargo checksum files must exactly reconcile with Cargo.lock")

if len(cargo_sources) != (len(registry_packages) * 2) + 1:
    raise SystemExit("Cargo sources must contain only registry archives, checksums, and vendor config")
PY
