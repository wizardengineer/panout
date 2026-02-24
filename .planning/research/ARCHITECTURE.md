# Architecture Patterns

**Domain:** Tmux session orchestration -- remote SSH sessions, CLI self-update, session switcher
**Researched:** 2026-02-24

## Recommended Architecture

Panout's existing architecture extends cleanly for all three features. No fundamental redesign is needed.

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `src/cli.rs` | Parse `--update` flag alongside existing flags | `src/main.rs` (entry point) |
| `src/tmux.rs` | Existing pane/window operations (unchanged) | `std::process::Command` (local tmux) |
| `src/ssh.rs` | Extended: build SSH commands with `-t` and tmux session creation | `src/tmux.rs` (via send_keys) |
| `src/session.rs` (new) | Remote tmux session command builder -- pure functions returning command strings | `src/ssh.rs` (command strings) |
| `src/update.rs` (new) | Self-update logic via `self_update` crate with ureq backend | External: GitHub API |
| `scripts/sesh` (new) | Standalone shell script, not compiled into panout binary | External: tmux CLI, fzf |

### Data Flow

**Remote session creation:**
```
User runs: panout -w myproject
  |
  v
cli.rs parses args --> loader.rs loads config --> resolver.rs expands refs
  |
  v
For each workspace entry with host:
  session.rs builds command:
    "ssh -t user@host \"tmux new-session -A -s myproject\""
  |
  v
  tmux.rs sends command to pane via send_keys (or split-window with command)
  |
  v
  Remote: SSH connects -> tmux creates or attaches to named session
```

**Self-update:**
```
User runs: panout --update
  |
  v
cli.rs detects --update --> main.rs calls update::run() (early exit, no config load)
  |
  v
update.rs:
  self_update::backends::github::Update::configure()
    .repo_owner("wizardengineer")
    .repo_name("panout")
    .bin_name("panout")
    .show_download_progress(true)
    .current_version(cargo_crate_version!())
    .build()?
    .update()?
  |
  v
Prints: "Updated panout v0.1.0 -> v0.2.0"
```

**Sesh session switching (on remote machine, standalone):**
```
User runs: sesh
  |
  v
tmux list-sessions -F "#{session_name}"
  | (pipe)
  v
fzf (user selects)
  | (pipe)
  v
tmux switch-client -t $selected
```

## Patterns to Follow

### Pattern 1: Subprocess Delegation for SSH
**What:** Continue using `std::process::Command` for system ssh/tmux.
**When:** Always. Panout is an orchestrator, not an SSH client.
**Why:** Inherits user's SSH config, agent forwarding, known_hosts, ProxyJump. Zero maintenance burden.
**Example:**
```rust
fn create_remote_session(host: &str, session_name: &str) -> Result<()> {
    let tmux_cmd = format!(
        "tmux new-session -A -s {}",
        shell_escape(session_name)
    );
    let status = Command::new("ssh")
        .args(["-t", host, &tmux_cmd])
        .status()
        .map_err(|e| PanoutError::SshError(e.to_string()))?;
    if !status.success() {
        return Err(PanoutError::SshError(
            format!("Failed to create session '{}' on {}", session_name, host)
        ));
    }
    Ok(())
}
```

### Pattern 2: Early-Return for --update
**What:** The `--update` flag exits before config loading or tmux checks.
**When:** Self-update has no dependency on config files or tmux sessions.
**Why:** Avoids `NotInTmux` error when user just wants to update. Avoids config parse errors blocking updates.
**Example:**
```rust
fn main() {
    let cli = Cli::parse();

    if cli.update {
        match update::run() {
            Ok(status) => {
                println!("Updated to v{}", status.version());
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Update failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Normal flow: load config, check tmux, etc.
}
```

### Pattern 3: self_update with ureq Backend
**What:** Use the `self_update` crate (v0.42) with `default-features = false` and the `ureq` feature flag.
**When:** For the `--update` implementation.
**Why:** `self_update` handles version comparison, target triple matching, download progress, archive extraction, and binary replacement. The ureq backend avoids pulling in tokio (~200 transitive deps) -- ureq is blocking, pure Rust, ~30 deps. The crate is battle-tested in production tools like `mise`.
**Config:**
```toml
self_update = { version = "0.42", default-features = false, features = [
    "ureq", "rustls", "archive-tar", "compression-flate2",
] }
```

### Pattern 4: Idempotent Remote Operations
**What:** All remote operations safe to retry.
**When:** Always, for remote session operations.
**Why:** Network disconnects, SSH timeouts, and user re-runs mean commands will execute multiple times. `tmux new-session -A` is inherently idempotent.

### Pattern 5: Shell Script as Standalone Artifact
**What:** Sesh lives in `scripts/sesh`, not compiled into the Rust binary.
**When:** For the session switcher.
**Why:** Different lifecycle (runs on remote machines), different install method (curl), different dependencies (fzf).

## Recommended Project Structure

```
src/
  cli.rs              # CLI argument parsing (add --update flag)
  config.rs           # TOML config types (unchanged or add session field)
  error.rs            # Error types (add UpdateError variant)
  interpolate.rs      # Variable substitution (unchanged)
  lib.rs              # Module exports (add session, update)
  loader.rs           # Config file discovery (unchanged)
  main.rs             # Entry point (add update routing, session routing)
  resolver.rs         # Bundle reference expansion (unchanged)
  session.rs          # NEW: Remote tmux session command builder
  ssh.rs              # SSH helpers (may extend for -t flag)
  tmux.rs             # Tmux pane/window operations (unchanged)
  update.rs           # NEW: Self-update via self_update crate
scripts/
  sesh                # NEW: Shell script for tmux session switching
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: SSH Library in Rust
**What:** Using `russh`, `ssh2`, or `openssh` crates.
**Why bad:** 50+ transitive deps, reimplements SSH config parsing, does not inherit user's `~/.ssh/config`. Every edge case the system SSH handles becomes your bug.
**Instead:** `Command::new("ssh")` with `-t` flag.

### Anti-Pattern 2: Async Runtime for a Synchronous CLI
**What:** Adding tokio via default `self_update` features or direct `reqwest` dependency.
**Why bad:** Panout executes tmux commands sequentially. No concurrent I/O to benefit from async. Adds ~2MB binary, ~15s compile time.
**Instead:** `self_update` with `ureq` feature (blocking HTTP).

### Anti-Pattern 3: Embedding Sesh in the Panout Binary
**What:** Building sesh as `panout sesh` subcommand.
**Why bad:** Sesh runs on remote machines where panout may not be installed. Coupling them means installing panout on every remote.
**Instead:** Standalone shell script, installable via curl.

### Anti-Pattern 4: Complex Session Name Sanitization
**What:** Over-sanitizing session names (stripping all non-alphanumeric).
**Why bad:** tmux handles most characters in session names. Over-sanitizing loses meaningful names.
**Instead:** Only handle colons (tmux target separators) and shell metacharacters. Replace dots with hyphens for safety.

### Anti-Pattern 5: has-session Check Before new-session
**What:** Running `tmux has-session -t name` then conditionally creating.
**Why bad:** TOCTOU race condition. Session could be created/destroyed between check and act.
**Instead:** `tmux new-session -A -s name` is atomic (attach if exists, create if not).

## Scalability Considerations

| Concern | Small Scale (5 hosts) | Medium Scale (50 hosts) | Notes |
|---------|----------------------|------------------------|-------|
| SSH connections | Sequential, fast | May feel slow (~1s/host) | Could parallelize with `std::thread` later if needed |
| Session creation | Near-instant | ~1s per host serial | tmux creation is sub-100ms; SSH handshake dominates |
| Self-update | Single HTTP request | Single HTTP request | Scales by definition |
| Sesh listing | Instant | Instant | `tmux list-sessions` handles thousands |

## Sources

- Existing panout source: `src/tmux.rs`, `src/ssh.rs`, `src/cli.rs`
- [self_update docs.rs](https://docs.rs/self_update/0.42.0/self_update/) -- UpdateBuilder API
- [self_update Cargo.toml](https://github.com/jaemk/self_update/blob/master/Cargo.toml) -- ureq feature confirmed
- [tmux wiki - Advanced Use](https://github.com/tmux/tmux/wiki/Advanced-Use) -- Scripting patterns
- [tmux wiki - Getting Started](https://github.com/tmux/tmux/wiki/Getting-Started) -- new-session -A flag
