#!/usr/bin/env python3
"""Calculate the next Noor Notes Snap version from a Store channel version."""

from __future__ import annotations

import re
import sys


VERSION_PATTERN = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")


def next_version(release_kind: str, current: str) -> str:
    match = VERSION_PATTERN.fullmatch(current)
    if match is None:
        raise ValueError(f"Store version must be MAJOR.MINOR.PATCH, got {current!r}")

    major, minor, patch = (int(part) for part in match.groups())
    if release_kind == "edge":
        patch += 1
    elif release_kind == "stable":
        minor += 1
        patch = 0
    else:
        raise ValueError(f"Release kind must be 'edge' or 'stable', got {release_kind!r}")
    return f"{major}.{minor}.{patch}"


def main() -> int:
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} edge|stable MAJOR.MINOR.PATCH", file=sys.stderr)
        return 2
    try:
        print(next_version(sys.argv[1], sys.argv[2]))
    except ValueError as error:
        print(error, file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
