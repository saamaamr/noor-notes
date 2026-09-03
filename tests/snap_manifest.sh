#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
manifest="$repo_root/snapcraft.yaml"

if [ ! -f "$manifest" ]; then
    printf 'Missing Snap manifest: %s\n' "$manifest" >&2
    exit 1
fi

python3 - "$manifest" <<'PY'
import sys
import tomllib
from pathlib import Path

import yaml


def require_equal(path, actual, expected):
    if actual != expected:
        raise SystemExit(f"{path} must equal {expected!r}; got {actual!r}")


manifest_path = Path(sys.argv[1])
source = manifest_path.read_text(encoding="utf-8")
manifest = yaml.safe_load(source)
workspace = tomllib.loads((manifest_path.parent / "Cargo.toml").read_text(encoding="utf-8"))
app_manifest = tomllib.loads(
    (manifest_path.parent / "apps/noor-notes/Cargo.toml").read_text(encoding="utf-8")
)
spelling_source = (manifest_path.parent / "apps/noor-notes/src/writing_assistance/spelling.rs").read_text(encoding="utf-8")

if not isinstance(manifest, dict):
    raise SystemExit("Snap manifest must be a YAML mapping")

require_equal("version", manifest.get("version"), app_manifest["package"]["version"])
require_equal("title", manifest.get("title"), "Noor Notes")
require_equal(
    "summary",
    manifest.get("summary"),
    "Private, encrypted notes with focused editing for Linux",
)
require_equal("license", manifest.get("license"), "GPL-3.0-or-later")
require_equal("website", manifest.get("website"), "https://github.com/saamaamr/noor-notes")
require_equal(
    "contact",
    manifest.get("contact"),
    "https://github.com/saamaamr/noor-notes/issues",
)
require_equal(
    "issues",
    manifest.get("issues"),
    "https://github.com/saamaamr/noor-notes/issues",
)
require_equal(
    "source-code",
    manifest.get("source-code"),
    "https://github.com/saamaamr/noor-notes",
)
require_equal("base", manifest.get("base"), "core24")
require_equal("confinement", manifest.get("confinement"), "strict")
require_equal("platforms", manifest.get("platforms"), {"amd64": None})

app = manifest.get("apps", {}).get("noor-notes")
if not isinstance(app, dict):
    raise SystemExit("apps.noor-notes must be a mapping")

require_equal("apps.noor-notes.command", app.get("command"), "usr/bin/noor-notes")
require_equal(
    "apps.noor-notes.desktop",
    app.get("desktop"),
    "usr/share/applications/io.github.saamaamr.NoorNotes.desktop",
)
require_equal("apps.noor-notes.common-id", app.get("common-id"), "io.github.saamaamr.NoorNotes")
require_equal("apps.noor-notes.extensions", app.get("extensions"), ["gnome"])
require_equal(
    "apps.noor-notes.plugs",
    app.get("plugs"),
    [
        "desktop",
        "wayland",
        "x11",
        "network",
        "network-bind",
        "password-manager-service",
    ],
)
require_equal("apps.noor-notes.slots", app.get("slots"), ["noor-notes-dbus"])

require_equal(
    "slots.noor-notes-dbus",
    manifest.get("slots", {}).get("noor-notes-dbus"),
    {
        "interface": "dbus",
        "bus": "session",
        "name": "io.github.saamaamr.NoorNotes",
    },
)

part = manifest.get("parts", {}).get("noor-notes")
if not isinstance(part, dict):
    raise SystemExit("parts.noor-notes must be a mapping")

require_equal("parts.noor-notes.plugin", part.get("plugin"), "rust")
require_equal("parts.noor-notes.source", part.get("source"), ".")
require_equal(
    "parts.noor-notes.after",
    part.get("after"),
    ["rust-deps", "libspelling-runtime"],
)
require_equal("parts.noor-notes.rust-channel", part.get("rust-channel"), "none")
require_equal("parts.noor-notes.rust-ignore-toolchain-file", part.get("rust-ignore-toolchain-file"), None)
for package in ("libspelling-1-dev",):
    if package not in part.get("build-packages", []):
        raise SystemExit(f"parts.noor-notes.build-packages must include {package}")
require_equal("parts.noor-notes.stage-packages", part.get("stage-packages"), None)
require_equal("parts.noor-notes.stage", part.get("stage"), None)
require_equal(
    "workspace.dependencies.sourceview5.features",
    workspace["workspace"]["dependencies"]["sourceview5"].get("features"),
    ["gtk_v4_14", "v5_16"],
)
if "language.name()" in spelling_source:
    raise SystemExit("Snap-compatible spelling language labels must not require libspelling 0.4 symbols")
require_equal(
    "parts.noor-notes.build-environment",
    part.get("build-environment"),
    [{"PATH": "$CRAFT_STAGE/usr/bin:${PATH}"}],
)

rust_deps = manifest.get("parts", {}).get("rust-deps")
if not isinstance(rust_deps, dict):
    raise SystemExit("parts.rust-deps must be a mapping")

require_equal("parts.rust-deps.plugin", rust_deps.get("plugin"), "nil")
require_equal(
    "parts.rust-deps.source",
    rust_deps.get("source"),
    "https://static.rust-lang.org/dist/rust-1.87.0-x86_64-unknown-linux-gnu.tar.xz",
)
require_equal("parts.rust-deps.source-type", rust_deps.get("source-type"), "tar")
require_equal(
    "parts.rust-deps.source-checksum",
    rust_deps.get("source-checksum"),
    "sha256/9720bf4ffdd5e6112f8fc93a645d50bfdc64f95cb76d41561be196e1721b4b69",
)
require_equal("parts.rust-deps.prime", rust_deps.get("prime"), ["-*"])
if "./install.sh --prefix=\"$CRAFT_PART_INSTALL/usr\" --disable-ldconfig" not in rust_deps.get("override-build", ""):
    raise SystemExit("parts.rust-deps.override-build must install Rust into the staged prefix")

spelling_runtime = manifest.get("parts", {}).get("libspelling-runtime")
if not isinstance(spelling_runtime, dict):
    raise SystemExit("parts.libspelling-runtime must be a mapping")
require_equal("parts.libspelling-runtime.plugin", spelling_runtime.get("plugin"), "nil")
require_equal(
    "parts.libspelling-runtime.stage-packages",
    spelling_runtime.get("stage-packages"),
    ["libspelling-1-1"],
)
require_equal(
    "parts.libspelling-runtime.stage",
    spelling_runtime.get("stage"),
    ["usr/lib/*/libspelling-1.so*"],
)
require_equal(
    "parts.libspelling-runtime.prime",
    spelling_runtime.get("prime"),
    ["usr/lib/*/libspelling-1.so*"],
)

for path in (
    "target/release/noor-notes",
    "data/io.github.saamaamr.NoorNotes.desktop",
    "data/io.github.saamaamr.NoorNotes.metainfo.xml",
    "data/io.github.saamaamr.NoorNotes.svg",
):
    if path not in part.get("override-build", ""):
        raise SystemExit(f"parts.noor-notes.override-build must install {path}")

if "extensions/gnome" in source:
    raise SystemExit("The Snap manifest must not bundle the GNOME Shell extension")
PY
