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
    "desktop-file-utils",
    "gjs",
    "ripgrep",
    "xvfb",
):
    if package not in native:
        raise SystemExit(f"Security workflow must install {package}")

gate = by_name.get("Run security gate", {})
if gate.get("run") != "xvfb-run -a -s '-screen 0 1920x1080x24' scripts/security-check.sh":
    raise SystemExit("Security workflow must provide a virtual display large enough for desktop layout tests")
if gate.get("env", {}) != {"GDK_BACKEND": "x11", "GSK_RENDERER": "cairo"}:
    raise SystemExit("Security workflow must use the isolated X11 software-rendered test display")
PY

fake_root=$(mktemp -d)
trap 'rm -rf "$fake_root"' EXIT
cargo_log="$fake_root/cargo.log"
desktop_log="$fake_root/desktop.log"

cat >"$fake_root/cargo" <<'SH'
#!/bin/sh
printf '%s\n' "$*" >>"$SECURITY_CARGO_LOG"
SH
cat >"$fake_root/rg" <<'SH'
#!/bin/sh
exit 1
SH
cat >"$fake_root/desktop-file-validate" <<'SH'
#!/bin/sh
printf '%s\n' "$*" >>"$SECURITY_DESKTOP_LOG"
SH
for command in bash gjs; do
    cat >"$fake_root/$command" <<'SH'
#!/bin/sh
exit 0
SH
done
chmod +x "$fake_root/cargo" "$fake_root/rg" "$fake_root/desktop-file-validate" \
    "$fake_root/bash" "$fake_root/gjs"

SECURITY_CARGO_LOG="$cargo_log" SECURITY_DESKTOP_LOG="$desktop_log" \
    PATH="$fake_root:$PATH" /bin/sh "$security_gate"

if ! grep -Fxq \
    'test -p noor-notes --features development --test cli --test development_identity' \
    "$cargo_log"; then
    printf 'Security gate must execute the Noor Notes Dev identity tests\n' >&2
    exit 1
fi

if ! grep -Fxq 'data/io.github.saamaamr.NoorNotes.Devel.desktop' "$desktop_log"; then
    printf 'Security gate must validate the Noor Notes Dev desktop launcher\n' >&2
    exit 1
fi
