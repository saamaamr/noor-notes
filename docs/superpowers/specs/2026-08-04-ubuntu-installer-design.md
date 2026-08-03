# Ubuntu Installer Design

## Goal

Provide beginner-friendly commands in the public README that install Noor Notes on supported Ubuntu systems without requiring users to discover build dependencies themselves.

## User Experience

The README will present these commands:

```bash
git clone https://github.com/saamaamr/noor-notes.git
cd noor-notes
./scripts/install-ubuntu.sh
```

The installer will explain what it is doing, request administrator authorization only for APT packages, install Rust through rustup only when `cargo` is unavailable, and delegate the user-local application installation to the existing `scripts/install-local.sh` script.

## Components

- `scripts/install-ubuntu.sh` validates that the host provides `apt-get`, installs the Ubuntu build and runtime packages, installs Rust when necessary, and runs the existing local installer.
- `scripts/install-local.sh` remains responsible for compiling and placing the binary, desktop metadata, icon, and GNOME extension under the current user's home directory.
- `README.md` distinguishes the recommended complete Ubuntu installation from the existing developer/manual path.

## Safety and Errors

The installer uses strict shell error handling and stops with a clear message on non-Ubuntu/non-APT systems. It does not use `curl | bash`; rustup is downloaded to a temporary file, executed, and then automatically removed with a shell trap. System package installation is the only step requiring elevated privileges.

## Verification

- Validate scripts with `bash -n` and ShellCheck when available.
- Test the non-destructive help path and dependency-detection behavior.
- Run `git diff --check` and confirm the README commands match the script name.
- Push the committed documentation and installer to `origin/main`.
