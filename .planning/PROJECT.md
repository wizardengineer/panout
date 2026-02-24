# Panout

## What This Is

Panout is a tmux pane orchestrator that creates windows and panes from TOML configuration. Users define pane arrangements, commands, and SSH connections in a config file, then spawn them with a single command. It's built in Rust and targets developers who work across multiple tmux sessions and remote machines.

## Core Value

Panout must reliably create the exact tmux layout and commands the user configured — bundles, workspaces, SSH connections, and pane arrangements all working from a single `panout` command.

## Requirements

### Validated

- Create tmux panes from TOML config with layout control (tiled, vertical, horizontal)
- Bundle references with `@group.name` syntax and recursive expansion with cycle detection
- Workspace support creating multiple windows with optional SSH and directory context
- CLI with `-b`, `-w`, `-n`, `-v`, `-H`, `--list` flags
- Cross-platform config discovery (XDG_CONFIG_HOME, ~/.config)
- Variable interpolation for `{user}` and `{ip}` in commands

### Active

- [ ] Tmux session creation per bundle on remote machines — when SSHing into a machine via a bundle, panout creates a named tmux session (session name = bundle name) on the remote host. If the session already exists, auto-reattach instead of creating a new one. This keeps processes running if the SSH connection drops.
- [ ] `sesh` session switcher tool — a lightweight shell script that lists tmux sessions, pipes through fzf for fuzzy selection, and switches to the chosen session. Runs on the remote machine. Installed via curl one-liner that places the script at `~/.local/bin/sesh`. Hosted in the panout GitHub repo.
- [ ] `panout --update` self-update — downloads the latest release binary from GitHub Releases and replaces the current binary. Implemented as a new CLI flag that fetches and installs the latest version.

### Out of Scope

- Auto-provisioning sesh on remote machines — sesh is installed manually via curl one-liner, not auto-deployed by panout
- Tmux keybinding configuration — sesh is a standalone command, users configure their own keybindings
- GUI or TUI interface — panout remains a CLI tool
- Process monitoring or restart — panout creates sessions but doesn't manage running processes
- Windows support — Linux/macOS only

## Context

- Panout is a Rust binary (edition 2024) using clap, serde, toml, thiserror, and dirs
- Currently requires running inside an existing tmux session (`NotInTmux` error if not)
- SSH is handled by delegating to the system `ssh` command via subprocess
- The binary is named `panout` and installed via `cargo install --path .`
- GitHub repo: `wizardengineer/panout`
- No existing release infrastructure (no GitHub Actions, no pre-built binaries)
- The tmux session feature changes the fundamental assumption: panout won't just operate within an existing session, it will create sessions on remote machines

## Constraints

- **Language**: Rust — existing codebase, no language change
- **Dependencies**: Minimize new crate additions; prefer standard library where possible
- **Tmux**: Must work with standard tmux installations (no plugins required)
- **SSH**: Must work with standard SSH client (system `ssh` binary)
- **Backwards compatibility**: Existing bundle and workspace configs must continue working unchanged

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| One tmux session per bundle on remote | Keeps processes alive across SSH disconnects | -- Pending |
| Auto-reattach to existing sessions | Most convenient UX — no manual `tmux attach` | -- Pending |
| sesh installed via curl one-liner | Simple, universal, no auto-provisioning complexity | -- Pending |
| sesh at ~/.local/bin/sesh | Standard user-local binary location | -- Pending |
| GitHub Releases for --update | Works for all users, not just Rust developers | -- Pending |
| sesh is command-only, no keybinding | Users run it in terminal on the SSH machine | -- Pending |

---
*Last updated: 2026-02-24 after initialization*
