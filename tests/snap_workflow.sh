#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workflow="$repo_root/.github/workflows/snap.yml"

python3 - "$workflow" <<'PY'
import sys
from pathlib import Path

import yaml


workflow = yaml.load(Path(sys.argv[1]).read_text(encoding="utf-8"), Loader=yaml.BaseLoader)
on = workflow.get("on", {})
if on.get("workflow_dispatch") != "" or "push" in on:
    raise SystemExit("Standalone Snap workflow must be manual-only; release.yml owns tag builds")
steps = workflow.get("jobs", {}).get("build", {}).get("steps", [])
if not isinstance(steps, list):
    raise SystemExit("Snap workflow must define jobs.build.steps")

steps_by_name = {step.get("name"): step for step in steps if isinstance(step, dict)}

lint = steps_by_name.get("Lint built Snap")
if not isinstance(lint, dict):
    raise SystemExit("Snap workflow must lint the built Snap")
lint_run = lint.get("run", "")
if 'sudo -u "$(id -un)" -E /snap/bin/snapcraft lint' not in lint_run:
    raise SystemExit("Snap lint must run in a fresh runner-user context with LXD group membership")

smoke = steps_by_name.get("Install and smoke-test built Snap")
if not isinstance(smoke, dict):
    raise SystemExit("Snap workflow must install and smoke-test the built Snap")
smoke_run = smoke.get("run", "")
for command in ("sudo snap install --dangerous", "snap run noor-notes --help"):
    if command not in smoke_run:
        raise SystemExit(f"Snap smoke test must run: {command}")
PY
