#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workflow="$repo_root/.github/workflows/snap-cadence.yml"

if [ ! -f "$workflow" ]; then
    printf 'Missing Snap cadence workflow: %s\n' "$workflow" >&2
    exit 1
fi

python3 - "$workflow" <<'PY'
import re
import sys
from pathlib import Path

import yaml


path = Path(sys.argv[1])
source = path.read_text(encoding="utf-8")
workflow = yaml.load(source, Loader=yaml.BaseLoader)
if not isinstance(workflow, dict):
    raise SystemExit("Snap cadence workflow must be a YAML mapping")

on = workflow.get("on", {})
if on.get("schedule") != [{"cron": "0 6 * * 1"}]:
    raise SystemExit("Snap cadence must run Mondays at 06:00 UTC / 12:00 Bangladesh time")
dispatch = on.get("workflow_dispatch", {}).get("inputs", {}).get("release", {})
if dispatch.get("default") != "edge" or dispatch.get("options") != ["edge", "stable"]:
    raise SystemExit("Manual cadence runs must support explicit edge and stable releases")

permissions = workflow.get("permissions", {})
if permissions != {"contents": "read", "actions": "read"}:
    raise SystemExit("Snap cadence must use only contents: read and actions: read")
if workflow.get("concurrency", {}).get("group") != "snap-store-publication":
    raise SystemExit("Tag and cadence Store publications must share one concurrency lock")

jobs = workflow.get("jobs", {})
for name in ("decide", "build-and-publish-edge", "store-smoke", "promote-stable"):
    if not isinstance(jobs.get(name), dict):
        raise SystemExit(f"Snap cadence must define jobs.{name}")

decision_source = "\n".join(
    step.get("run", "")
    for step in jobs["decide"].get("steps", [])
    if isinstance(step, dict)
)
for fragment in (
    "actions/workflows/snap-cadence.yml/runs",
    "head_sha",
    "date -u +%d",
    "publish_edge",
    "promote_stable",
    "release_kind",
    "publish_edge=true",
):
    if fragment not in decision_source:
        raise SystemExit(f"Cadence decision gate must include: {fragment}")

edge = jobs["build-and-publish-edge"]
edge_steps = edge.get("steps", [])
steps_by_name = {
    step.get("name"): step for step in edge_steps if isinstance(step, dict)
}
if edge.get("if") != "needs.decide.outputs.publish_edge == 'true'":
    raise SystemExit("Edge build must skip when main has already been published")
for name in (
    "Set automatic build version",
    "Verify Snap runtime contract",
    "Lint built Snap",
    "Install and smoke-test built Snap",
    "Publish to edge",
):
    if name not in steps_by_name:
        raise SystemExit(f"Edge publication must include: {name}")
runtime_contract = steps_by_name["Verify Snap runtime contract"]
runtime_run = runtime_contract.get("run", "")
for fragment in ("tests/snap_runtime_contract.sh", "${{ steps.build-snap.outputs.snap }}"):
    if fragment not in runtime_run:
        raise SystemExit(f"Cadence Snap runtime contract must include: {fragment}")
versioning = steps_by_name["Set automatic build version"]
if versioning.get("env", {}).get("RELEASE_KIND") != "${{ needs.decide.outputs.release_kind }}":
    raise SystemExit("Automatic versioning must consume the decision job's release kind")
version_source = versioning.get("run", "")
for fragment in (
    "snap info noor-notes",
    "latest/edge",
    "latest/stable",
    "scripts/next-snap-version.py",
    "scripts/set-build-version.py",
):
    if fragment not in version_source:
        raise SystemExit(f"Automatic Store versioning must include: {fragment}")
publish = steps_by_name["Publish to edge"]
if publish.get("uses") != "canonical/action-publish@214b86e5ca036ead1668c79afb81e550e6c54d40":
    raise SystemExit("Store publication must use the reviewed canonical/action-publish commit")
if publish.get("with", {}).get("release") != "latest/edge":
    raise SystemExit("Weekly publication must target latest/edge")
if publish.get("env", {}).get("SNAPCRAFT_STORE_CREDENTIALS") != "${{ secrets.SNAPCRAFT_STORE_CREDENTIALS }}":
    raise SystemExit("Store publication must use the scoped repository secret")

smoke_source = "\n".join(
    step.get("run", "")
    for step in jobs["store-smoke"].get("steps", [])
    if isinstance(step, dict)
)
for command in ("snap install noor-notes --edge", "snap run noor-notes --help"):
    if command not in smoke_source:
        raise SystemExit(f"Store smoke gate must run: {command}")

promote = jobs["promote-stable"]
if promote.get("needs") != ["decide", "store-smoke"]:
    raise SystemExit("Stable promotion must wait for the Store smoke gate")
if promote.get("if") != "needs.decide.outputs.promote_stable == 'true' && needs.store-smoke.result == 'success'":
    raise SystemExit("Stable promotion must be explicitly gated by a successful smoke test")
promote_source = "\n".join(
    step.get("run", "")
    for step in promote.get("steps", [])
    if isinstance(step, dict)
)
if "snapcraft promote noor-notes --from-channel latest/edge --to-channel latest/stable --yes" not in promote_source:
    raise SystemExit("Monthly stable must promote the tested edge revision")

expected_actions = {
    "actions/checkout": "11d5960a326750d5838078e36cf38b85af677262",
    "canonical/action-build": "3bdaa03e1ba6bf59a65f84a751d943d549a54e79",
    "canonical/action-publish": "214b86e5ca036ead1668c79afb81e550e6c54d40",
}
for job_name, job in jobs.items():
    for step in job.get("steps", []):
        if not isinstance(step, dict) or "uses" not in step:
            continue
        match = re.fullmatch(r"(.+)@([0-9a-f]{40})", step["uses"])
        if match is None or expected_actions.get(match.group(1)) != match.group(2):
            raise SystemExit(f"{job_name} action must use its reviewed immutable commit: {step['uses']!r}")
PY
