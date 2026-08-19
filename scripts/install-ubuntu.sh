#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf '%s\n' \
        'Install Noor Notes Dev on Ubuntu' \
        '' \
        'Usage: ./scripts/install-ubuntu.sh [--help]' \
        '' \
        'Installs system dependencies with APT, installs Rust when missing,' \
        'then runs scripts/install-local.sh to install Noor Notes Dev for this user.'
}

if [[ ${1:-} == "--help" || ${1:-} == "-h" ]]; then
    usage
    exit 0
fi

if [[ $# -ne 0 ]]; then
    usage >&2
    exit 2
fi

if ! command -v apt-get >/dev/null 2>&1; then
    printf 'This installer supports Ubuntu and other APT-based systems only.\n' >&2
    exit 1
fi

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
apt_command=(apt-get)

if [[ $EUID -ne 0 ]]; then
    if ! command -v sudo >/dev/null 2>&1; then
        printf 'Administrator access is required. Install sudo or run this script as root.\n' >&2
        exit 1
    fi
    apt_command=(sudo apt-get)
fi

printf 'Installing Noor Notes system dependencies...\n'
"${apt_command[@]}" update
"${apt_command[@]}" install -y \
    build-essential \
    curl \
    desktop-file-utils \
    libadwaita-1-dev \
    libgtk-4-dev \
    libgtksourceview-5-dev \
    libspelling-1-dev \
    enchant-2 \
    hunspell-en-us \
    libsecret-tools \
    libsqlite3-dev \
    libssl-dev \
    libx11-dev \
    pkg-config

if ! command -v cargo >/dev/null 2>&1; then
    printf 'Installing the Rust toolchain for the current user...\n'
    rustup_file=$(mktemp)
    trap 'rm -f "$rustup_file"' EXIT
    curl --proto '=https' --tlsv1.2 --fail --show-error --location \
        https://sh.rustup.rs --output "$rustup_file"
    sh "$rustup_file" -y --profile minimal
    export PATH="$HOME/.cargo/bin:$PATH"
fi

printf 'Building and installing Noor Notes Dev for the current user...\n'
"$repo_root/scripts/install-local.sh"
