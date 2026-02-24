# Stack Research

**Domain:** Tmux session orchestration -- remote SSH sessions, CLI self-update, session switcher
**Researched:** 2026-02-24
**Confidence:** HIGH

## Context

Panout is an existing Rust CLI (edition 2024) with clap 4, serde 1.0, toml 0.8, thiserror 2, and dirs 6. This research covers **new** dependencies needed for three milestone features:

1. Creating/attaching named tmux sessions on remote hosts via SSH
2. Self-update from GitHub Releases (`panout --update`)
3. An fzf-based tmux session switcher shell script (`sesh`)

## Recommended Stack

### Feature 1: Remote Tmux Sessions via SSH

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `ssh -t` (system binary) | N/A | Force pseudo-terminal allocation for remote tmux | Panout already delegates to system `ssh` via `std::process::Command`. Adding `-t` flag is required because tmux needs a TTY. No new crate needed. |
| `tmux new-session -A -s` (remote command) | tmux 1.8+ | Attach-or-create named session on remote host | The `-A` flag (available since tmux 1.8, March 2013) creates the session if it does not exist or attaches if it does. This single command replaces the need for `has-session` + conditional logic. |

**No new Rust crates required.** The implementation is entirely `std::process::Command` orchestration:
```bash
# What panout will execute via Command::new("ssh")
ssh -t user@host "tmux new-session -A -s bundle_name"
```

The key tmux commands to use on the remote side:
- `tmux new-session -A -s <name>` -- attach-or-create (idempotent, safe to retry)
- `tmux has-session -t <name>` -- check existence (returns exit code 0/1, useful for scripting)
- `tmux list-sessions` -- enumerate all sessions (used by the sesh switcher)

### Feature 2: CLI Self-Update from GitHub Releases

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `self_update` | 0.42 | Self-update from GitHub Releases | Battle-tested crate (used by `mise`, others). Handles version comparison, asset matching by target triple, download with progress, archive extraction, and binary replacement. Supports ureq backend to avoid pulling in tokio/async. |
| ureq (via self_update) | 3.x | HTTP client (blocking) | Pure Rust, blocking I/O, no async runtime, minimal dependency tree. Used through self_update's `ureq` feature flag -- not added as a direct dependency. |

**Cargo.toml addition:**
```toml
[dependencies]
self_update = { version = "0.42", default-features = false, features = [
    "ureq",
    "rustls",
    "archive-tar",
    "compression-flate2",
] }
```

**Why this specific feature set:**
- `default-features = false` -- disables the default `reqwest` backend, which pulls in tokio and adds ~200+ transitive dependencies
- `ureq` -- blocking HTTP client, pure Rust, ~30 transitive deps instead of ~200
- `rustls` -- pure Rust TLS, avoids OpenSSL system dependency (important for static linking and portability across Linux distros)
- `archive-tar` + `compression-flate2` -- GitHub release assets will be `.tar.gz` archives (standard for Linux/macOS)

**GitHub Release asset naming convention** required by `self_update`:
```
panout-v{version}-{target_triple}.tar.gz
```
Examples:
- `panout-v0.2.0-x86_64-unknown-linux-gnu.tar.gz`
- `panout-v0.2.0-x86_64-apple-darwin.tar.gz`
- `panout-v0.2.0-aarch64-apple-darwin.tar.gz`

### Feature 3: Sesh Session Switcher (Shell Script)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| POSIX shell (sh/bash) | N/A | Script runtime | Runs on remote machines, must work with minimal dependencies. POSIX sh for maximum portability, bash shebang acceptable since bash is ubiquitous on Linux servers. |
| fzf | 0.20+ | Fuzzy finder for session selection | The standard tool for interactive terminal selection. Widely installed, fast, configurable. The script requires fzf as a runtime dependency on the remote machine. |
| tmux CLI | 1.8+ | Session listing and switching | `tmux list-sessions` for enumeration, `tmux switch-client -t` for switching. No plugins needed. |

**No Rust crates needed.** Sesh is a standalone shell script, not part of the Rust binary.

### Supporting Libraries (CI/Release Infrastructure)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| cargo-dist | 0.30+ | Build cross-platform release binaries + installers | For GitHub Actions CI to produce `.tar.gz` release assets. Generates shell/powershell install scripts. Set up once when implementing `--update`. |
| release-plz | 0.3+ | Automated release PRs from conventional commits | Optional. Automates version bumping and changelog. Use if release frequency justifies the setup cost. |

**cargo-dist** is the recommended release pipeline tool because:
- It generates asset names compatible with `self_update`'s expected format
- It handles cross-compilation for Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
- It creates GitHub Actions workflows automatically
- One-time `cargo dist init` setup, then releases are automated on git tag push

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| `self_update` (ureq backend) | Manual implementation with `ureq` + `self-replace` + `semver` | `self_update` handles all the edge cases: version comparison, target triple matching, archive extraction, progress display, binary replacement. Manual implementation would reimplement ~500 lines of well-tested logic for no benefit. |
| `self_update` (ureq backend) | `self_update` (reqwest backend, default) | Default reqwest backend pulls in tokio, adding ~200 transitive dependencies and ~15s compile time. Panout is a synchronous CLI -- async runtime is pure waste. ureq backend achieves the same result with ~30 deps. |
| `self_update` | `axoupdater` (cargo-dist's updater) | `axoupdater` is tightly coupled to cargo-dist's installer model (generates a separate updater binary). `self_update` is simpler: one function call in your existing binary. `axoupdater` adds complexity without benefit for a single-binary CLI. |
| `self_update` | `patchify` | `patchify` requires running a custom update server. Panout uses GitHub Releases. Wrong model. |
| `self_update` | `updater-lp` | Less mature, fewer downloads, less documentation. `self_update` has broader adoption and is proven in production tools like `mise`. |
| Shell script (sesh) | Go binary (like joshmedeski/sesh) | Panout's sesh is a lightweight remote-machine utility. A compiled Go binary requires a build step and is harder to install via curl one-liner. A shell script is a single file, zero dependencies beyond fzf and tmux, installable with `curl -o`. |
| Shell script (sesh) | Rust binary | Same reasoning. Compiling and distributing a separate Rust binary per-platform for a 20-line script is over-engineering. Shell is the right tool for this. |
| `ssh -t` (subprocess) | `russh` / `ssh2` Rust crates | Panout already uses system ssh via subprocess. SSH crates add complexity (key management, agent forwarding, known_hosts) and ~50+ transitive deps. Subprocess approach is simpler, inherits user's SSH config, and is already working. |
| cargo-dist | Manual GitHub Actions | cargo-dist generates the CI workflow, handles cross-compilation matrix, and produces consistent asset names. Manual workflow is ~100 lines of YAML to maintain. |
| `rustls` (via self_update feature) | `native-tls` / `default-tls` | `rustls` avoids requiring OpenSSL development headers on the build machine. Produces a fully statically-linkable binary. Important for cross-compilation and CI reproducibility. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `reqwest` (directly or via self_update default features) | Pulls in tokio async runtime (~200 transitive deps). Panout is synchronous. Adds ~15s to clean compile time. | `self_update` with `ureq` feature, `default-features = false` |
| `russh` / `ssh2` Rust crates | Massive dependency tree, complex key management, duplicates what system SSH already does. Panout's subprocess SSH works and inherits user config. | `std::process::Command::new("ssh")` with `-t` flag |
| `openssh` Rust crate | Requires tokio runtime for multiplexed connections. Overkill for sending one command. | System `ssh` subprocess |
| `tokio` (directly) | Panout has no async requirements. Adding an async runtime increases binary size ~2MB and compile time ~15s for zero benefit. | Keep everything synchronous |
| `libssh2-sys` / `openssl-sys` | C library bindings requiring system development headers. Breaks cross-compilation and complicates CI. | `rustls` for TLS, system SSH for connections |
| tmux plugins (for sesh) | Requires TPM (tmux plugin manager) on remote machines. Extra dependency, harder to install via curl. | Standalone shell script |

## Installation

```toml
# Add to Cargo.toml [dependencies]
self_update = { version = "0.42", default-features = false, features = [
    "ureq",
    "rustls",
    "archive-tar",
    "compression-flate2",
] }
```

```bash
# CI: Install cargo-dist for release infrastructure (one-time setup)
cargo install cargo-dist
cargo dist init
```

```bash
# Sesh install one-liner (for remote machines)
curl -fsSL https://raw.githubusercontent.com/wizardengineer/panout/master/scripts/sesh \
  -o ~/.local/bin/sesh && chmod +x ~/.local/bin/sesh
```

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| self_update 0.42 | Rust edition 2024 (MSRV 1.85) | Verified: self_update requires Rust 1.85+, panout uses edition 2024 which requires 1.85+. Compatible. |
| self_update 0.42 (ureq feature) | ureq 3.x | self_update pins ureq to `3.0.6` minimum with features `["gzip", "json", "socks-proxy", "charset"]`. |
| self_update 0.42 | cargo-dist release assets | Both use `{name}-{version}-{target_triple}.tar.gz` naming. cargo-dist's default asset naming is compatible with self_update's asset matching. |
| fzf 0.20+ | tmux 1.8+ | fzf's `--preview` and `--bind` features work with tmux's `list-sessions` format. No version conflicts. |
| tmux `-A` flag | tmux 1.8+ | Added March 2013. Any tmux version in active use supports this. |

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Remote tmux sessions (no new deps) | HIGH | Standard tmux commands, verified in official tmux wiki. `ssh -t` + `tmux new-session -A -s` is the canonical pattern. |
| self_update with ureq backend | HIGH | Verified on crates.io (v0.42.0, Dec 2024), GitHub (Cargo.toml confirms ureq feature), docs.rs (API confirmed). Used by mise in production. |
| cargo-dist for release assets | MEDIUM | cargo-dist generates compatible asset names, but exact naming format interop with self_update was not directly tested. May need `bin_path_in_archive` config. |
| sesh as shell script with fzf | HIGH | Dozens of blog posts and tools demonstrate this exact pattern. `tmux list-sessions | fzf | tmux switch-client -t` is the standard approach. |
| ureq 3.x as blocking HTTP client | HIGH | Verified on crates.io (v3.2.0, Feb 2026). Widely used, 73M+ downloads. Pure Rust, blocking API. |

## Sources

- [jaemk/self_update GitHub](https://github.com/jaemk/self_update) -- Crate source, Cargo.toml feature flags, usage examples (HIGH confidence)
- [self_update on crates.io](https://crates.io/crates/self_update) -- Version 0.42.0, last updated Dec 2024 (HIGH confidence)
- [self_update docs.rs](https://docs.rs/self_update/0.42.0/self_update/) -- API reference, UpdateBuilder methods, asset naming (HIGH confidence)
- [ureq on crates.io](https://crates.io/crates/ureq) -- Version 3.2.0, last updated Feb 2026 (HIGH confidence)
- [self-replace on crates.io](https://crates.io/crates/self-replace) -- Version 1.5.0, Sep 2024 (HIGH confidence)
- [tmux wiki - Getting Started](https://github.com/tmux/tmux/wiki/Getting-Started) -- `new-session -A` flag documentation (HIGH confidence)
- [Starting tmux Sessions via SSH](https://blog.jbowen.dev/tips/ssh-tmux/) -- `ssh -t` pattern for remote tmux (MEDIUM confidence)
- [Waylon Walker - tmux fzf session jumper](https://waylonwalker.com/tmux-fzf-session-jump/) -- fzf session switcher patterns (MEDIUM confidence)
- [Howard Do - Fzf Tmux Session Manager](https://howarddo2208.github.io/posts/04-fzf-tmux-session-manager/) -- fzf keybinding patterns (MEDIUM confidence)
- [cargo-dist documentation](https://axodotdev.github.io/cargo-dist/) -- Release infrastructure (MEDIUM confidence)
- [axoupdater crate](https://crates.io/crates/axoupdater) -- cargo-dist's updater library, considered and rejected (MEDIUM confidence)
- [joshmedeski/sesh](https://github.com/joshmedeski/sesh) -- Reference for what a full session manager looks like (Go), informed decision to keep panout's sesh as a simple shell script (MEDIUM confidence)

---
*Stack research for: Panout milestone -- remote sessions, self-update, session switcher*
*Researched: 2026-02-24*
