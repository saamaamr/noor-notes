# Ubuntu Installer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe Ubuntu dependency installer and document the three-command installation flow in the public README.

**Architecture:** A new Ubuntu-specific shell entry point installs system build dependencies, bootstraps Rust only when it is missing, and delegates all user-local application deployment to the existing local installer. A shell contract test exercises help output and unsupported-host behavior without modifying the system.

**Tech Stack:** Bash, APT, rustup, Git, Markdown

## Global Constraints

- Do not use `curl | bash`.
- Request administrator authorization only for APT package installation.
- Install the application itself under the current user's `~/.local` paths.
- Stop with a clear error on systems without `apt-get`.
- Preserve `scripts/install-local.sh` as the owner of build and user-local deployment.

---

### Task 1: Ubuntu dependency installer

**Files:**
- Create: `scripts/install-ubuntu.sh`
- Create: `tests/install_ubuntu.sh`

**Interfaces:**
- Consumes: `scripts/install-local.sh`, `apt-get`, optional `sudo`, optional `curl`, optional `cargo`
- Produces: `scripts/install-ubuntu.sh [--help]`, returning zero for help and a nonzero status with a clear message when APT is unavailable

- [ ] **Step 1: Write the failing shell contract test**

Create `tests/install_ubuntu.sh` to assert that `--help` mentions Ubuntu, dependencies, and `scripts/install-local.sh`; assert that a PATH containing required shell utilities but no `apt-get` fails with `This installer supports Ubuntu and other APT-based systems only.`

- [ ] **Step 2: Run the test to verify it fails**

Run: `bash tests/install_ubuntu.sh`

Expected: FAIL because `scripts/install-ubuntu.sh` does not exist.

- [ ] **Step 3: Implement the minimal installer**

Create a strict-mode Bash script that:

1. Prints non-mutating usage for `--help`.
2. Requires `apt-get` and selects `sudo` only when `EUID != 0`.
3. Runs `apt-get update` and installs `build-essential curl pkg-config libgtk-4-dev libadwaita-1-dev libsqlite3-dev libssl-dev libx11-dev libsecret-tools desktop-file-utils`.
4. If `cargo` is missing, downloads `https://sh.rustup.rs` to a `mktemp` file, registers a trap to delete it, runs `sh "$rustup_file" -y --profile minimal`, and prepends `$HOME/.cargo/bin` to PATH.
5. Executes the repository's `scripts/install-local.sh`.

- [ ] **Step 4: Run installer checks**

Run: `bash -n scripts/install-ubuntu.sh tests/install_ubuntu.sh && bash tests/install_ubuntu.sh`

Expected: PASS with `Ubuntu installer contract checks passed.`

- [ ] **Step 5: Commit**

```bash
git add scripts/install-ubuntu.sh tests/install_ubuntu.sh
git commit -m "feat: add Ubuntu installation command"
```

### Task 2: README installation flow and publication

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: public repository URL `https://github.com/saamaamr/noor-notes.git` and `scripts/install-ubuntu.sh`
- Produces: copyable clone, change-directory, and install commands for Ubuntu users

- [ ] **Step 1: Update README installation instructions**

Replace the current prerequisite-first section with a recommended installation block:

```bash
git clone https://github.com/saamaamr/noor-notes.git
cd noor-notes
./scripts/install-ubuntu.sh
```

Explain that the script installs Ubuntu dependencies, installs Rust only when missing, builds Noor Notes, and installs it for the current user. Retain `./scripts/install-local.sh` as the documented path for contributors who already have dependencies.

- [ ] **Step 2: Verify documentation and scripts**

Run: `bash -n scripts/install-ubuntu.sh scripts/install-local.sh && bash tests/install_ubuntu.sh && rg -n 'git clone https://github.com/saamaamr/noor-notes.git|./scripts/install-ubuntu.sh|./scripts/install-local.sh' README.md && git diff --check`

Expected: all commands exit zero and README contains all three required references.

- [ ] **Step 3: Commit README**

```bash
git add README.md
git commit -m "docs: add Ubuntu installation command"
```

- [ ] **Step 4: Push and verify GitHub**

Run: `git push origin main && git rev-parse HEAD && git rev-parse origin/main`

Expected: both commit hashes are identical and GitHub displays the updated installation section.
