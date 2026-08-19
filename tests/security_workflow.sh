#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workflow="$repo_root/.github/workflows/security.yml"
security_gate="$repo_root/scripts/security-check.sh"

if ! grep -Fq 'bash tests/automatic_versioning.sh' "$security_gate"; then
    printf 'Security gate must execute the automatic versioning contract\n' >&2
    exit 1
fi

python3 - "$workflow" <<'PY'
import sys
from pathlib import Path

import yaml


path = Path(sys.argv[1])
workflow = yaml.load(path.read_text(encoding="utf-8"), Loader=yaml.BaseLoader)
if not isinstance(workflow, dict):
    raise SystemExit("Security workflow must be a YAML mapping")

security = workflow.get("jobs", {}).get("security", {})
if security.get("runs-on") != "ubuntu-26.04":
    raise SystemExit("Security workflow needs Ubuntu 26.04 for GtkSourceView >= 5.16")

steps = security.get("steps", [])
by_name = {step.get("name"): step for step in steps if isinstance(step, dict)}
native = by_name.get("Install native dependencies", {}).get("run", "")
for package in (
    "libgtk-4-dev",
    "libadwaita-1-dev",
    "libx11-dev",
    "libspelling-1-dev",
    "libenchant-2-2",
    "hunspell-en-us",
    "gjs",
    "ripgrep",
    "xvfb",
):
    if package not in native:
        raise SystemExit(f"Security workflow must install {package}")

if by_name.get("Run security gate", {}).get("run") != "xvfb-run -a scripts/security-check.sh":
    raise SystemExit("Security workflow must execute the repository gate with a virtual display")
PY
