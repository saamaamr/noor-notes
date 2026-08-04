#!/bin/sh

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
workflow="$repo_root/.github/workflows/flatpak.yml"

if [ ! -f "$workflow" ]; then
    printf 'Missing Flatpak workflow: %s\n' "$workflow" >&2
    exit 1
fi

python3 - "$workflow" <<'PY'
import sys
from pathlib import Path
import re

import yaml


workflow = yaml.load(Path(sys.argv[1]).read_text(encoding="utf-8"), Loader=yaml.BaseLoader)
source = Path(sys.argv[1]).read_text(encoding="utf-8")

if not isinstance(workflow, dict):
    raise SystemExit("Flatpak workflow must be a YAML mapping")

on = workflow.get("on", {})
if on.get("workflow_dispatch") != "":
    raise SystemExit("Flatpak workflow must support manual dispatch")
if "push" in on:
    raise SystemExit("Standalone Flatpak workflow must be manual-only; release.yml owns tag builds")

job = workflow.get("jobs", {}).get("build")
if not isinstance(job, dict):
    raise SystemExit("Flatpak workflow must define jobs.build")

container = job.get("container", {})
if container.get("image") != "ghcr.io/flathub-infra/flatpak-github-actions@sha256:ab91c589e30298efc3bca549141aa1672a250fefa57d50e11300276f2dfc558f":
    raise SystemExit("Flatpak workflow must use the reviewed GNOME 50 image digest")
if container.get("options") != "--privileged":
    raise SystemExit("Flatpak workflow container must allow Flatpak sandboxing")

steps = job.get("steps", [])
if not isinstance(steps, list):
    raise SystemExit("Flatpak workflow must define build steps")

builder_steps = [
    step
    for step in steps
    if isinstance(step, dict)
    and step.get("uses")
    == "flatpak/flatpak-github-actions/flatpak-builder@401fe28a8384095fc1531b9d320b292f0ee45adb"
]
if len(builder_steps) != 1:
    raise SystemExit("Flatpak workflow must use the official Flatpak builder action once")

expected_actions = {
    "actions/checkout": "11d5960a326750d5838078e36cf38b85af677262",
    "flatpak/flatpak-github-actions/flatpak-builder": "401fe28a8384095fc1531b9d320b292f0ee45adb",
}
for step in steps:
    if not isinstance(step, dict) or "uses" not in step:
        continue
    match = re.fullmatch(r"(.+)@([0-9a-f]{40})", step["uses"])
    if match is None or expected_actions.get(match.group(1)) != match.group(2):
        raise SystemExit(f"Flatpak workflow action must use its reviewed immutable commit: {step['uses']!r}")

for action, commit, version in (
    ("actions/checkout", expected_actions["actions/checkout"], "v4.4.0"),
    ("flatpak/flatpak-github-actions/flatpak-builder", expected_actions["flatpak/flatpak-github-actions/flatpak-builder"], "v6.7"),
):
    if f"{action}@{commit} # {version}" not in source:
        raise SystemExit(f"Flatpak action pin must identify its reviewed version: {action}")

builder = builder_steps[0].get("with", {})
if builder.get("manifest-path") != "packaging/flatpak/io.github.saamaamr.NoorNotes.yml":
    raise SystemExit("Flatpak builder must build the release manifest")
if builder.get("bundle") != "noor-notes.flatpak":
    raise SystemExit("Flatpak builder must produce the Noor Notes bundle")
if builder.get("repo-dir") != "flatpak-repo":
    raise SystemExit("Flatpak builder must export a dedicated test repository")
if builder.get("upload-artifact") != "true":
    raise SystemExit("Flatpak builder must upload the built bundle as an artifact")

steps_by_name = {step.get("name"): step for step in steps if isinstance(step, dict)}
smoke = steps_by_name.get("Install and smoke-test Flatpak bundle")
if not isinstance(smoke, dict):
    raise SystemExit("Flatpak workflow must install and smoke-test the built bundle")

smoke_run = smoke.get("run", "")
for command in (
    'flatpak --user remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo',
    'flatpak --user remote-add --if-not-exists --no-gpg-verify noor-notes-test "file://$GITHUB_WORKSPACE/flatpak-repo"',
    'flatpak --user install --noninteractive noor-notes-test io.github.saamaamr.NoorNotes//master',
    "flatpak --user run --command=sh io.github.saamaamr.NoorNotes -c 'test -f /app/share/licenses/io.github.saamaamr.NoorNotes/noor-notes/GPL-3.0-or-later.txt && test -x /usr/bin/secret-tool'",
    'flatpak --user run --command=noor-notes io.github.saamaamr.NoorNotes --help',
):
    if command not in smoke_run:
        raise SystemExit(f"Flatpak smoke test must run: {command}")
PY
