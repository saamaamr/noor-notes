#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

python3 - "$repo_root" <<'PY'
import sys
import tomllib
from pathlib import Path


root = Path(sys.argv[1])
expected = tomllib.loads(
    (root / "apps/noor-notes/Cargo.toml").read_text(encoding="utf-8")
)["package"]["version"]

manifests = sorted(root.glob("crates/*/Cargo.toml")) + [
    root / "apps/noor-notes/Cargo.toml"
]
for manifest_path in manifests:
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    actual = manifest.get("package", {}).get("version")
    if actual != expected:
        raise SystemExit(
            f"{manifest_path.relative_to(root)} package.version must equal {expected!r}; got {actual!r}"
        )

lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
workspace_packages = {
    "noor-crypto",
    "noor-domain",
    "noor-notes",
    "noor-storage",
    "noor-sync",
    "noor-windowing",
    "noor-xpad-import",
}
locked = {
    package["name"]: package["version"]
    for package in lock["package"]
    if package["name"] in workspace_packages
}
if locked != {name: expected for name in workspace_packages}:
    raise SystemExit(f"Workspace Cargo.lock versions must all equal {expected!r}; got {locked!r}")

readme = (root / "README.md").read_text(encoding="utf-8")
for fragment in (
    f"**Current release:** v{expected}",
    f"releases/download/v{expected}",
    f"noor-notes_{expected}_amd64.snap",
    "Every Monday at 12:00 Bangladesh time",
    "first Monday of each month",
):
    if fragment not in readme:
        raise SystemExit(f"README release documentation must include: {fragment}")
PY
