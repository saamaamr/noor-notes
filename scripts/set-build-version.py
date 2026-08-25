#!/usr/bin/env python3
"""Synchronize an ephemeral Noor Notes build workspace to one release version."""

from __future__ import annotations

import argparse
import datetime as dt
import re
from pathlib import Path


VERSION_PATTERN = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
WORKSPACE_PACKAGES = (
    "noor-crypto",
    "noor-domain",
    "noor-notes",
    "noor-storage",
    "noor-sync",
    "noor-windowing",
    "noor-xpad-import",
)


def replace_once(path: Path, pattern: str, replacement: str, description: str) -> None:
    source = path.read_text(encoding="utf-8")
    updated, count = re.subn(pattern, replacement, source, count=1, flags=re.MULTILINE)
    if count != 1:
        raise ValueError(f"Could not find exactly one {description} in {path}")
    path.write_text(updated, encoding="utf-8")


def replace_count(
    path: Path, pattern: str, replacement: str, expected_count: int, description: str
) -> None:
    source = path.read_text(encoding="utf-8")
    updated, count = re.subn(pattern, replacement, source, flags=re.MULTILINE)
    if count != expected_count:
        raise ValueError(
            f"Expected {expected_count} {description} entries in {path}, found {count}"
        )
    path.write_text(updated, encoding="utf-8")


def synchronize(root: Path, version: str, release_date: str) -> None:
    if VERSION_PATTERN.fullmatch(version) is None:
        raise ValueError(f"Build version must be MAJOR.MINOR.PATCH, got {version!r}")
    dt.date.fromisoformat(release_date)

    manifests = sorted(root.glob("crates/*/Cargo.toml")) + [
        root / "apps/noor-notes/Cargo.toml"
    ]
    if len(manifests) != len(WORKSPACE_PACKAGES):
        raise ValueError(f"Expected {len(WORKSPACE_PACKAGES)} workspace manifests, found {len(manifests)}")
    for manifest in manifests:
        replace_once(
            manifest,
            r'^version\s*=\s*"[^"]+"$',
            f'version = "{version}"',
            "package version",
        )

    lock_path = root / "Cargo.lock"
    lock = lock_path.read_text(encoding="utf-8")
    for package in WORKSPACE_PACKAGES:
        pattern = rf'(\[\[package\]\]\nname = "{re.escape(package)}"\nversion = ")[^"]+("\n)'
        lock, count = re.subn(pattern, rf"\g<1>{version}\g<2>", lock, count=1)
        if count != 1:
            raise ValueError(f"Could not synchronize {package} in {lock_path}")
    lock_path.write_text(lock, encoding="utf-8")

    replace_once(
        root / "snapcraft.yaml",
        r'^version:\s*"[^"]+"$',
        f'version: "{version}"',
        "Snap version",
    )
    replace_once(
        root / "data/io.github.saamaamr.NoorNotes.metainfo.xml",
        r'<release version="[^"]+" date="[^"]+" type="stable">',
        f'<release version="{version}" date="{release_date}" type="stable">',
        "latest AppStream release",
    )
    cli_test = root / "apps/noor-notes/tests/cli.rs"
    replace_once(
        cli_test,
        r'"Noor Notes [0-9]+\.[0-9]+\.[0-9]+"',
        f'"Noor Notes {version}"',
        "production CLI version assertion",
    )
    replace_count(
        cli_test,
        r'"Noor Notes Dev [0-9]+\.[0-9]+\.[0-9]+"',
        f'"Noor Notes Dev {version}"',
        2,
        "development CLI version assertion",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--date", default=dt.datetime.now(dt.UTC).date().isoformat())
    args = parser.parse_args()
    try:
        synchronize(args.root.resolve(), args.version, args.date)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
