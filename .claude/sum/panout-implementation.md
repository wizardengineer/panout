# Panout Implementation Session

**Date**: 2026-01-06
**Project**: `~/Projects/panssh/` (binary name: `panout`)
**Status**: Core implementation complete, ready for testing in tmux

---

## Summary

Implemented a Rust CLI tool called **Panout** that creates tmux panes and runs commands from a TOML config. This is a rewrite/evolution of a Python POC called `panssh`.

---

## Key Design Decisions

### Config Format Simplifications

| Original Design | Final Implementation |
|-----------------|---------------------|
| `[bundles.dev.frontend]` | `[dev.frontend]` (no `bundles.` prefix) |
| `commands = [...]` | `cmd = ...` (shorter name) |
| `commands` array only | `cmd` accepts string OR array |
| Manual `pane = N` | Auto-assign by definition order (optional override) |

### Reserved Top-Level Keys
- `defaults` - global settings (layout)
- `servers` - SSH configurations
- Everything else is treated as a bundle group

### Bundle References
- `@group.name` - reference a specific bundle
- `@group.*` - reference all bundles in a group
- References can be mixed with commands in `cmd` array
- Recursive expansion with circular reference detection

---

## Files Implemented

### `src/config.rs`
Configuration types that map to TOML structure:
- `Cmd` enum - accepts either `String` or `Vec<String>` via serde untagged
- `Layout` enum - `Tiled`, `Vertical`, `Horizontal` with tmux layout name conversion
- `BundleEntry` - single bundle with `cmd`, optional `pane`, `role`, `layout`
- `ServerConfig` - SSH server with `host`, `disconnect`, `cmd`
- `Config` - top-level struct with `defaults`, `servers`, `bundles` HashMap
- `Config::from_str()` - custom parsing that treats non-reserved keys as bundle groups
- Helper methods: `get_bundle()`, `get_group()`, `list_bundles()`, `list_servers()`

### `src/error.rs`
Error types using thiserror:
- `ConfigNotFound` - config file doesn't exist
- `NoConfigDir` - can't determine config directory
- `IoError` - file read errors
- `ParseError` - TOML parsing errors
- `BundleNotFound` - requested bundle doesn't exist
- `ServerNotFound` - requested server doesn't exist
- `InvalidRef` - malformed `@ref` syntax
- `CircularRef` - bundle references itself (directly or indirectly)
- `TmuxError` - tmux command failed
- `NotInTmux` - not running inside tmux session

### `src/loader.rs`
Config file discovery and loading:
- `default_config_path()` - checks XDG_CONFIG_HOME, ~/.config, then platform default
- `load_config()` - reads file and parses with `Config::from_str()`
- `load_default_config()` - convenience wrapper
- `ensure_config_dir()` - creates config directory if missing

### `src/cli.rs`
Clap argument parsing:
- `-b, --bundle` - bundle name in `group.name` format
- `-n, --num` - number of panes to create
- `-v` - vertical split (side by side, maps to `even-horizontal`)
- `-H` - horizontal split (stacked, maps to `even-vertical`)
- `-l, --list` - list available bundles and servers
- `layout()` method - converts CLI flags to `Layout` enum

### `src/tmux.rs`
Tmux pane and window management via `std::process::Command`:
- `in_tmux()` - checks if TMUX env var is set
- `create_panes()` - splits window N-1 times, applies layout after each split
- `send_keys()` - sends command to specific pane with Enter
- `set_layout()` - applies layout to current window
- `select_pane()` - focuses a specific pane
- `pane_count()` - gets number of panes in current window

### `src/resolver.rs`
Bundle reference resolution:
- `ResolvedRef` enum - `Command`, `BundleRef`, or `GroupAll`
- `parse_ref()` - parses string into `ResolvedRef`
- `resolve_bundle()` - recursively expands `@refs` to flat command list
- `resolve_with_panes()` - expands refs while preserving pane assignments
- Circular reference detection using HashSet of visited paths
- Unit tests for ref parsing

### `src/interpolate.rs`
Variable expansion for server commands:
- `parse_host()` - splits `user@ip` into tuple
- `interpolate()` - replaces `{user}` and `{ip}` in command strings
- Unit tests for both functions

### `src/ssh.rs`
SSH session handling (sends commands via tmux):
- `connect()` - sends `ssh host` to specified pane
- `disconnect()` - sends `exit` to specified pane

### `src/main.rs`
CLI entry point that wires everything together:
1. Parse CLI args
2. Load config from default path
3. Handle `--list` flag (print bundles/servers and exit)
4. Resolve bundle to get pane → commands mapping
5. Determine layout (CLI > bundle > defaults > tiled)
6. Create tmux panes
7. Send commands to each pane

### `src/lib.rs`
Public module exports - re-exports key types for external use.

---

## Example Config

```toml
# ~/.config/panout/config.toml

[defaults]
layout = "tiled"

[dev.frontend]
pane = 0
cmd = "npm run dev"

[dev.backend]
pane = 1
cmd = ["cd ~/api", "cargo watch -x run"]

[dev.all]
cmd = ["@dev.frontend", "@dev.backend"]

[servers.wizard]
host = "user@192.168.1.1"
cmd = ["cd {user}/src", "@dev.all"]
```

---

## CLI Usage

```bash
panout --list                    # List bundles and servers
panout -b dev.frontend -n 1      # Run single bundle
panout -b dev.all -n 3 -v        # 3 vertical panes, expand refs
```

---

## Key Technical Patterns

### Cmd Enum (String or Array)
```rust
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Cmd {
    Single(String),
    Multiple(Vec<String>),
}
```

### Dynamic Config Parsing
```rust
// Reserved keys handled explicitly, rest are bundle groups
for (key, value) in table {
    match key.as_str() {
        "defaults" => config.defaults = value.try_into()?,
        "servers" => config.servers = value.try_into()?,
        _ => { config.bundles.insert(key, value.try_into()?); }
    }
}
```

### Cross-Platform Config Path
```rust
// Check in order: XDG_CONFIG_HOME → ~/.config → platform default
if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") { ... }
if let Some(home) = dirs::home_dir() { ... }  // ~/.config/panout/
dirs::config_dir()  // fallback (~/Library/Application Support on macOS)
```

---

## Issues Encountered

1. **macOS config path**: `dirs::config_dir()` returns `~/Library/Application Support/` on macOS, not `~/.config/`. Fixed by checking `~/.config` first.

2. **Rename from panssh to panout**: Required updating error type names, imports, config paths, and CLI metadata across all files.

---

## Commands Reference

```bash
# Build
cargo build
cargo build --release

# Test
cargo test

# Run
cargo run -- --list
cargo run -- -b dev.frontend -n 2 -v

# Install
cargo install --path .
```

---

## Where We Left Off

- All core functionality implemented and compiling
- Tests passing (5 unit tests)
- Config file set up at `~/.config/panout/config.toml`
- Binary can be installed with `cargo install --path .`
- **Not yet tested in actual tmux session**

---

## TODO: Next Steps

1. [ ] Test in tmux session with real bundles
2. [ ] Test `@ref` expansion with nested bundles
3. [ ] Implement server/SSH workflow (connect → run commands → optional disconnect)
4. [ ] Add `--dry-run` flag to preview commands without executing
5. [ ] Add `panout init` subcommand to create default config
6. [ ] Consider publishing to crates.io (needs license, repo metadata)
7. [ ] Rename project directory from `panssh/` to `panout/`
8. [ ] Add more comprehensive tests for resolver edge cases

---

## Dependencies

```toml
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
thiserror = "2"
dirs = "6"
```
