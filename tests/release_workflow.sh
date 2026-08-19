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
import re

import yaml


workflow = yaml.load(Path(sys.argv[1]).read_text(encoding="utf-8"), Loader=yaml.BaseLoader)
source = Path(sys.argv[1]).read_text(encoding="utf-8")

for package in ("libspelling-1-dev", "libenchant-2-2", "hunspell-en-us"):
    if package not in source:
        raise SystemExit(f"Release workflow must install {package} before native checks")

if not isinstance(workflow, dict):
    raise SystemExit("Release workflow must be a YAML mapping")

on = workflow.get("on", {})
if on.get("push", {}).get("tags") != ["v*"]:
    raise SystemExit("Release workflow must build version tags")

jobs = workflow.get("jobs", {})
for job_name in ("snap", "flatpak", "release", "store-edge", "store-smoke", "store-stable"):
    if not isinstance(jobs.get(job_name), dict):
        raise SystemExit(f"Release workflow must define jobs.{job_name}")

snap_steps = jobs["snap"].get("steps", [])
if not any(
    isinstance(step, dict)
    and step.get("uses") == "canonical/action-build@3bdaa03e1ba6bf59a65f84a751d943d549a54e79"
    for step in snap_steps
):
    raise SystemExit("Release workflow must build a Snap with the reviewed canonical/action-build commit")
if not any(
    isinstance(step, dict)
    and step.get("uses") == "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"
    for step in snap_steps
):
    raise SystemExit("Release workflow must preserve the Snap artifact")
if not any(
    isinstance(step, dict) and step.get("name") == "Verify package version matches tag"
    for step in snap_steps
):
    raise SystemExit("Release workflow must reject a Snap version that differs from the tag")

flatpak_steps = jobs["flatpak"].get("steps", [])
if not any(
    isinstance(step, dict)
    and step.get("uses")
    == "flatpak/flatpak-github-actions/flatpak-builder@401fe28a8384095fc1531b9d320b292f0ee45adb"
    for step in flatpak_steps
):
    raise SystemExit("Release workflow must build a Flatpak with the official builder action")
if not any(
    isinstance(step, dict) and step.get("name") == "Verify package version matches tag"
    for step in flatpak_steps
):
    raise SystemExit("Release workflow must reject a Flatpak version that differs from the tag")
flatpak_steps_by_name = {
    step.get("name"): step for step in flatpak_steps if isinstance(step, dict)
}
pin_source = flatpak_steps_by_name.get("Pin Flatpak source to checked-out commit")
if not isinstance(pin_source, dict):
    raise SystemExit("Release workflow must pin the Flatpak source to the checked-out tag commit")
pin_run = pin_source.get("run", "")
for fragment in ("commit: ${GITHUB_SHA}", 'grep -Fq "        commit: ${GITHUB_SHA}"'):
    if fragment not in pin_run:
        raise SystemExit(f"Release Flatpak source pin must include: {fragment}")
if flatpak_steps.index(pin_source) > next(
    index for index, step in enumerate(flatpak_steps) if isinstance(step, dict) and "uses" in step and "flatpak-builder@" in step["uses"]
):
    raise SystemExit("Release workflow must pin the Flatpak source before building")

release = jobs["release"]
if release.get("needs") != ["snap", "flatpak"]:
    raise SystemExit("Release job must wait for both package builds")
if "if" in release:
    raise SystemExit("Every validated v* tag must be eligible for release publication")
release_steps = release.get("steps", [])
if not any(
    isinstance(step, dict)
    and step.get("uses") == "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093"
    for step in release_steps
):
    raise SystemExit("Release workflow must collect package artifacts")
if not any(
    isinstance(step, dict) and "sha256sum" in step.get("run", "")
    for step in release_steps
):
    raise SystemExit("Release workflow must produce SHA-256 checksums")
if not any(
    isinstance(step, dict)
    and step.get("uses") == "softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65"
    for step in release_steps
):
    raise SystemExit("Release workflow must attach artifacts to the GitHub release")

permissions = workflow.get("permissions", {})
if permissions.get("contents") != "read":
    raise SystemExit("Release workflow build jobs must use contents: read")
if release.get("permissions", {}).get("contents") != "write":
    raise SystemExit("Only the release job needs contents: write")
if workflow.get("concurrency", {}).get("group") != "snap-store-publication":
    raise SystemExit("Tag and cadence Store publications must share one concurrency lock")

store_edge = jobs["store-edge"]
if store_edge.get("needs") != ["snap", "release"]:
    raise SystemExit("Store edge publication must wait for the validated Snap and GitHub release")
store_edge_steps = store_edge.get("steps", [])
if not any(
    isinstance(step, dict)
    and step.get("uses") == "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093"
    and step.get("with", {}).get("name") == "noor-notes-snap"
    for step in store_edge_steps
):
    raise SystemExit("Store publication must download the exact validated release Snap")
publish_step = next(
    (
        step
        for step in store_edge_steps
        if isinstance(step, dict)
        and step.get("uses") == "canonical/action-publish@214b86e5ca036ead1668c79afb81e550e6c54d40"
    ),
    None,
)
if not isinstance(publish_step, dict):
    raise SystemExit("Release workflow must publish with the reviewed canonical action")
if publish_step.get("with", {}).get("release") != "latest/edge":
    raise SystemExit("Validated tag Snap must enter latest/edge first")
if publish_step.get("env", {}).get("SNAPCRAFT_STORE_CREDENTIALS") != "${{ secrets.SNAPCRAFT_STORE_CREDENTIALS }}":
    raise SystemExit("Store publication must use the scoped Snapcraft credentials secret")

store_smoke = jobs["store-smoke"]
if store_smoke.get("needs") != "store-edge":
    raise SystemExit("Store smoke test must wait for edge publication")
store_smoke_source = "\n".join(
    step.get("run", "") for step in store_smoke.get("steps", []) if isinstance(step, dict)
)
for fragment in (
    "snap install noor-notes --edge",
    "snap refresh noor-notes --edge",
    'expected="Noor Notes ${GITHUB_REF_NAME#v}"',
    "snap run noor-notes --version",
    "snap run noor-notes --help",
):
    if fragment not in store_smoke_source:
        raise SystemExit(f"Store smoke test must include: {fragment}")

store_stable = jobs["store-stable"]
if store_stable.get("needs") != "store-smoke":
    raise SystemExit("Stable promotion must wait for the Store-installed edge smoke test")
store_stable_source = "\n".join(
    step.get("run", "") for step in store_stable.get("steps", []) if isinstance(step, dict)
)
if "snapcraft promote noor-notes --from-channel latest/edge --to-channel latest/stable --yes" not in store_stable_source:
    raise SystemExit("Stable must promote the tested edge revision without rebuilding it")

expected_actions = {
    "actions/checkout": "11d5960a326750d5838078e36cf38b85af677262",
    "actions/upload-artifact": "ea165f8d65b6e75b540449e92b4886f43607fa02",
    "actions/download-artifact": "d3f86a106a0bac45b974a628896c90dbdf5c8093",
    "canonical/action-build": "3bdaa03e1ba6bf59a65f84a751d943d549a54e79",
    "canonical/action-publish": "214b86e5ca036ead1668c79afb81e550e6c54d40",
    "flatpak/flatpak-github-actions/flatpak-builder": "401fe28a8384095fc1531b9d320b292f0ee45adb",
    "softprops/action-gh-release": "3bb12739c298aeb8a4eeaf626c5b8d85266b0e65",
}
for job_name, job in jobs.items():
    for step in job.get("steps", []):
        if not isinstance(step, dict) or "uses" not in step:
            continue
        uses = step["uses"]
        match = re.fullmatch(r"(.+)@([0-9a-f]{40})", uses)
        if match is None:
            raise SystemExit(f"{job_name} action must use a full immutable commit SHA: {uses!r}")
        if expected_actions.get(match.group(1)) != match.group(2):
            raise SystemExit(f"{job_name} action pin is not the reviewed commit: {uses!r}")

container_image = jobs["flatpak"].get("container", {}).get("image")
if container_image != "ghcr.io/flathub-infra/flatpak-github-actions@sha256:ab91c589e30298efc3bca549141aa1672a250fefa57d50e11300276f2dfc558f":
    raise SystemExit("Flatpak release container must use the reviewed immutable digest")

for action, commit, version in (
    ("actions/checkout", expected_actions["actions/checkout"], "v4.4.0"),
    ("actions/upload-artifact", expected_actions["actions/upload-artifact"], "v4.6.2"),
    ("actions/download-artifact", expected_actions["actions/download-artifact"], "v4.3.0"),
    ("canonical/action-build", expected_actions["canonical/action-build"], "v1"),
    ("canonical/action-publish", expected_actions["canonical/action-publish"], "v1"),
    ("flatpak/flatpak-github-actions/flatpak-builder", expected_actions["flatpak/flatpak-github-actions/flatpak-builder"], "v6.7"),
    ("softprops/action-gh-release", expected_actions["softprops/action-gh-release"], "v2.6.2"),
):
    if f"{action}@{commit} # {version}" not in source:
        raise SystemExit(f"Release action pin must identify its reviewed version: {action}")
PY
