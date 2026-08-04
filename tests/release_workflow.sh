#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workflow="$repo_root/.github/workflows/release.yml"

if [ ! -f "$workflow" ]; then
    printf 'Missing release workflow: %s\n' "$workflow" >&2
    exit 1
fi

python3 - "$workflow" <<'PY'
import sys
from pathlib import Path

import yaml


workflow = yaml.load(Path(sys.argv[1]).read_text(encoding="utf-8"), Loader=yaml.BaseLoader)

if not isinstance(workflow, dict):
    raise SystemExit("Release workflow must be a YAML mapping")

on = workflow.get("on", {})
if on.get("push", {}).get("tags") != ["v*"]:
    raise SystemExit("Release workflow must build version tags")

jobs = workflow.get("jobs", {})
for job_name in ("snap", "flatpak", "release"):
    if not isinstance(jobs.get(job_name), dict):
        raise SystemExit(f"Release workflow must define jobs.{job_name}")

snap_steps = jobs["snap"].get("steps", [])
if not any(
    isinstance(step, dict) and step.get("uses") == "canonical/action-build@v1"
    for step in snap_steps
):
    raise SystemExit("Release workflow must build a Snap with canonical/action-build@v1")
if not any(
    isinstance(step, dict) and step.get("uses") == "actions/upload-artifact@v4"
    for step in snap_steps
):
    raise SystemExit("Release workflow must preserve the Snap artifact")

flatpak_steps = jobs["flatpak"].get("steps", [])
if not any(
    isinstance(step, dict)
    and step.get("uses") == "flatpak/flatpak-github-actions/flatpak-builder@v6"
    for step in flatpak_steps
):
    raise SystemExit("Release workflow must build a Flatpak with the official builder action")

release = jobs["release"]
if release.get("needs") != ["snap", "flatpak"]:
    raise SystemExit("Release job must wait for both package builds")
if release.get("if") != "github.ref_name == 'v0.1.0'":
    raise SystemExit("Release publication must be limited to the final v0.1.0 tag")
release_steps = release.get("steps", [])
if not any(
    isinstance(step, dict) and step.get("uses") == "actions/download-artifact@v4"
    for step in release_steps
):
    raise SystemExit("Release workflow must collect package artifacts")
if not any(
    isinstance(step, dict) and "sha256sum" in step.get("run", "")
    for step in release_steps
):
    raise SystemExit("Release workflow must produce SHA-256 checksums")
if not any(
    isinstance(step, dict) and step.get("uses") == "softprops/action-gh-release@v2"
    for step in release_steps
):
    raise SystemExit("Release workflow must attach artifacts to the GitHub release")

permissions = workflow.get("permissions", {})
if permissions.get("contents") != "write":
    raise SystemExit("Release workflow needs contents: write to create the GitHub release")
PY
