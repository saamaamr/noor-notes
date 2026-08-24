#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workflow="$repo_root/.github/workflows/snap.yml"

python3 - "$workflow" <<'PY'
import sys
from pathlib import Path
import re

import yaml


workflow = yaml.load(Path(sys.argv[1]).read_text(encoding="utf-8"), Loader=yaml.BaseLoader)
source = Path(sys.argv[1]).read_text(encoding="utf-8")
on = workflow.get("on", {})
if on.get("workflow_dispatch") != "" or "push" in on:
    raise SystemExit("Standalone Snap workflow must be manual-only; release.yml owns tag builds")
steps = workflow.get("jobs", {}).get("build", {}).get("steps", [])
if not isinstance(steps, list):
    raise SystemExit("Snap workflow must define jobs.build.steps")

steps_by_name = {step.get("name"): step for step in steps if isinstance(step, dict)}

expected_actions = {
    "actions/checkout": "11d5960a326750d5838078e36cf38b85af677262",
    "canonical/action-build": "3bdaa03e1ba6bf59a65f84a751d943d549a54e79",
    "actions/upload-artifact": "ea165f8d65b6e75b540449e92b4886f43607fa02",
}
for step in steps:
    if not isinstance(step, dict) or "uses" not in step:
        continue
    match = re.fullmatch(r"(.+)@([0-9a-f]{40})", step["uses"])
    if match is None or expected_actions.get(match.group(1)) != match.group(2):
        raise SystemExit(f"Snap workflow action must use its reviewed immutable commit: {step['uses']!r}")

for action, commit, version in (
    ("actions/checkout", expected_actions["actions/checkout"], "v4.4.0"),
    ("canonical/action-build", expected_actions["canonical/action-build"], "v1"),
    ("actions/upload-artifact", expected_actions["actions/upload-artifact"], "v4.6.2"),
):
    if f"{action}@{commit} # {version}" not in source:
        raise SystemExit(f"Snap action pin must identify its reviewed version: {action}")

lint = steps_by_name.get("Lint built Snap")
if not isinstance(lint, dict):
    raise SystemExit("Snap workflow must lint the built Snap")
lint_run = lint.get("run", "")
if 'sudo -u "$(id -un)" -E /snap/bin/snapcraft lint' not in lint_run:
    raise SystemExit("Snap lint must run in a fresh runner-user context with LXD group membership")

runtime_contract = steps_by_name.get("Verify Snap runtime contract")
if not isinstance(runtime_contract, dict):
    raise SystemExit("Snap workflow must verify the built artifact runtime contract")
runtime_run = runtime_contract.get("run", "")
for fragment in ("tests/snap_runtime_contract.sh", "${{ steps.build-snap.outputs.snap }}"):
    if fragment not in runtime_run:
        raise SystemExit(f"Snap runtime contract must include: {fragment}")

smoke = steps_by_name.get("Install and smoke-test built Snap")
if not isinstance(smoke, dict):
    raise SystemExit("Snap workflow must install and smoke-test the built Snap")
smoke_run = smoke.get("run", "")
for command in ("sudo snap install --dangerous", "snap run noor-notes --help"):
    if command not in smoke_run:
        raise SystemExit(f"Snap smoke test must run: {command}")

upload = steps_by_name.get("Upload Snap artifact")
if not isinstance(upload, dict):
    raise SystemExit("Snap workflow must upload the built artifact")
if upload.get("with", {}).get("name") != "noor-notes-snap":
    raise SystemExit("Snap workflow artifact name must not hard-code a product version")
PY
