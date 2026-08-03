#!/usr/bin/env bash
set -euo pipefail

cargo check --workspace --all-targets
cargo test --workspace
