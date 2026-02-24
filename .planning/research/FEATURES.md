# Feature Landscape

**Domain:** Tmux session orchestration -- remote SSH sessions, CLI self-update, session switcher
**Researched:** 2026-02-24

## Table Stakes

Features users expect for this milestone. Missing = the milestone feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Named tmux session per bundle on remote | Core promise: SSH processes survive disconnects. Every tmux workflow guide centers on named sessions for persistence. | Medium | `ssh -t host "tmux new-session -A -s name"` |
| Attach-or-create semantics | Users expect idempotent behavior -- running panout twice should not error or create duplicates. The `-A` flag on `new-session` is the standard pattern. | Low | Single tmux flag (`-A`) handles this atomically. No custom logic needed. |
| Session persistence across SSH disconnects | This is the entire reason to create tmux sessions on remotes. If sessions die when SSH drops, the feature is pointless. | Low | Inherent to tmux -- this is what tmux does. The key is creating a proper session (not just sending commands to a pane). |
| TTY allocation for SSH commands | `ssh -t` is required for tmux to work over SSH. Without it, tmux fails with "open terminal failed: not a terminal". | Low | Must use `ssh -t` (or `-tt` for scripts). |
| `panout --update` downloads latest binary | Users without Rust toolchain need a way to update. Every modern CLI offers this. | Medium | `self_update` crate with ureq backend handles the heavy lifting. |
| Version comparison before download | Don't re-download if already on latest. Users expect "Already up to date" feedback. | Low | `self_update` compares `cargo_crate_version!()` against GitHub release tag. |
| Download progress indicator | Users need feedback during multi-MB download. | Low | `self_update` supports `.show_download_progress(true)`. |
| Platform-correct binary selection | macOS users get macOS binary, Linux users get Linux binary. Must auto-detect. | Low | `self_update` auto-detects target triple from compilation target. |
| Sesh lists all tmux sessions | The fundamental operation. `tmux list-sessions` piped to fzf. | Low | Single tmux command, standard pattern. |
| Sesh switches to selected session | After selection, switch the tmux client to the chosen session. | Low | `tmux switch-client -t $selected`. |
| Sesh installable via curl one-liner | Must work on any remote machine without build tools. Standard for shell script distribution. | Low | Single file download to `~/.local/bin/sesh`. |
| Backwards compatibility with existing configs | Existing bundle and workspace TOML configs must keep working unchanged. | Low | New session behavior is additive, not breaking. |

## Differentiators

Features that set panout apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Config-driven remote sessions (TOML) | No other tool creates named tmux sessions on remotes from a declarative config. tmuxinator/tmuxp define local sessions. Panout's TOML config with `[servers.*]` sections is unique. | Med | This is panout's core differentiator for this milestone. |
| Bundle-as-session-name convention | Session names derived automatically from bundle names. Zero config, predictable naming. | Low | Convention, not code. Document it and make it the default. |
| Multi-host session setup in one command | `panout -b servers.all` creates sessions across multiple remotes in a single invocation via bundle expansion. | Med | Already supported by `@group.name` expansion. Session logic just applies per-host. |
| `--update` shows version transition | Display "Updated from v0.1.0 to v0.2.0" with changelog link. | Low | `self_update` returns the new version in `status.version()`. |
| Sesh with session preview | Show pane content in fzf preview window for faster context switching. | Med | `fzf --preview "tmux capture-pane -t {} -p"` |
| Sesh with kill binding | Delete sessions without leaving the picker via `ctrl-x`. | Low | `fzf --bind "ctrl-x:execute(tmux kill-session -t {})+reload(...)"` |
| Sesh with new session creation | Type a name that does not match -> create it. Turns switcher into creator. | Low | Check fzf output against existing sessions; if not found, `tmux new-session -s <name>`. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Auto-provisioning sesh on remote machines | SSH command injection risk, complexity, couples sesh lifecycle to panout. Writing to remote filesystems without explicit consent. | Provide clear curl one-liner in docs. Users install sesh themselves. |
| Auto-update check on every panout invocation | Slows down every command with a network check. Annoying and surprising. Many tools (npm, pip) are widely disliked for this. | Only check when user explicitly runs `--update`. |
| Background update notification | Adds complexity (timestamp files, background threads). Over-engineered for a CLI tool. | Users run `--update` when they want to update. |
| Sesh as a compiled binary | Over-engineering. Adds cross-compilation burden for a ~30 line script. Shell runs everywhere tmux runs. | Shell script, installable via curl, zero compilation. |
| tmux plugin (TPM) distribution for sesh | TPM requires specific directory structures, adds dependency. Not available on minimal servers. | Standalone script. Users set their own keybindings in `.tmux.conf`. |
| Remote session monitoring/health checks | Panout creates sessions, not a process manager. Monitoring is a different tool's job. | Document `tmux list-sessions` for manual checks. |
| Windows support for self-update | Panout is Linux/macOS only (tmux does not run natively on Windows). | Only produce Linux and macOS release assets. |
| Zoxide integration in sesh | Useful for local development but irrelevant on remote servers where sesh runs. | Keep sesh focused on tmux session switching only. |
| SSH connection multiplexing management | Managing ControlMaster sockets adds complexity and couples panout to SSH config. | Document recommended SSH config settings in docs. |
| YAML/JSON config format support | Splits ecosystem and documentation. TOML is panout's deliberate choice. | Stay with TOML. |

## Feature Dependencies

```
Release Infrastructure (cargo-dist + GitHub Actions)
  --> self_update (--update needs release assets to exist)

Remote tmux sessions (ssh -t + tmux new-session -A)
  --> sesh becomes useful (navigating sessions panout creates)

sesh script in repo
  --> curl one-liner install (hosted raw file URL)
```

## MVP Recommendation

Prioritize:
1. **Remote tmux session creation** -- Core value proposition. Use `tmux new-session -A -s <bundle_name>` pattern.
2. **Release infrastructure** -- Required before `--update` can be tested. Set up cargo-dist + GitHub Actions.
3. **Self-update** -- Implement with `self_update` crate (ureq backend). Test against real releases.
4. **Sesh shell script** -- Lightweight addition. Basic list + switch via fzf.

Defer:
- **Session preview in sesh**: Add after validating users actually use sesh
- **Changelog display in --update**: Can be added without breaking changes
- **Version pinning for --update**: Edge case for teams. "Give me latest" is the common case.

## Sources

- [tmux wiki - Getting Started](https://github.com/tmux/tmux/wiki/Getting-Started) -- Session management commands
- [self_update docs.rs](https://docs.rs/self_update/0.42.0/self_update/) -- Feature capabilities
- [joshmedeski/sesh](https://github.com/joshmedeski/sesh) -- Reference session manager (Go, full-featured)
- [Howard Do - Fzf Tmux Session Manager](https://howarddo2208.github.io/posts/04-fzf-tmux-session-manager/) -- fzf keybinding patterns
- [Waylon Walker - tmux fzf session jumper](https://waylonwalker.com/tmux-fzf-session-jump/) -- Session switcher patterns
