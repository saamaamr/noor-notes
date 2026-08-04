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

import yaml


workflow = yaml.load(Path(sys.argv[1]).read_text(encoding="utf-8"), Loader=yaml.BaseLoader)

if not isinstance(workflow, dict):
    raise SystemExit("Flatpak workflow must be a YAML mapping")

on = workflow.get("on", {})
if on.get("workflow_dispatch") != "":
    raise SystemExit("Flatpak workflow must support manual dispatch")
if on.get("push", {}).get("tags") != ["v*"]:
    raise SystemExit("Flatpak workflow must build version tags")

job = workflow.get("jobs", {}).get("build")
if not isinstance(job, dict):
    raise SystemExit("Flatpak workflow must define jobs.build")

container = job.get("container", {})
if container.get("image") != "ghcr.io/flathub-infra/flatpak-github-actions:gnome-50":
    raise SystemExit("Flatpak workflow must use the GNOME 50 Flatpak builder image")
if container.get("options") != "--privileged":
    raise SystemExit("Flatpak workflow container must allow Flatpak sandboxing")

steps = job.get("steps", [])
if not isinstance(steps, list):
    raise SystemExit("Flatpak workflow must define build steps")

builder_steps = [
    step
    for step in steps
    if isinstance(step, dict)
    and step.get("uses") == "flatpak/flatpak-github-actions/flatpak-builder@v6"
]
if len(builder_steps) != 1:
    raise SystemExit("Flatpak workflow must use the official Flatpak builder action once")

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
    'flatpak --user run --command=noor-notes io.github.saamaamr.NoorNotes --help',
):
    if command not in smoke_run:
        raise SystemExit(f"Flatpak smoke test must run: {command}")
PY
