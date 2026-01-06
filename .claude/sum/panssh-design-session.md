# PanSSH Design Session Summary

**Date**: 2026-01-06
**Project**: `~/Projects/panssh/`
**Status**: Project scaffolded, ready for implementation

---

## What Was Discussed

### Original Concept

User had a Python POC (`~/.local/bin/panssh`) that creates tmux panes with SSH connections from JSON config. They wanted to:

1. Rewrite in Rust
2. Use a custom TOML-like DSL with advanced features

### Original Custom DSL Idea (in `~/.local/bin/test.toml`)

```toml
[[project]]
  pane.0 = "echo 'TOML is great fun!'"

[[project]]
  pane.1 = ["cd ~/.config/nvim && nvim init.vim"]
  role = "primary"

[server:wizardenginer@192.21.11.1]
  "cd {host}/src"           # Bare string command
  [project.0]               # Reference by index
  [project.1]
  "cd {host}/src/app"
  [project.all]             # Run all in group

![database]                 # Disconnect marker
```

### Problems Identified

1. **Fragile indexing**: `[project.0]`, `[project.1]` breaks if entries reordered
2. **Not valid TOML**: Bare strings and nested table references aren't parseable
3. **Custom parser needed**: Would require `nom`, adding weeks of work
4. **No tooling**: Custom format = no syntax highlighting, formatters

### Solution Agreed Upon: Valid TOML

| Original | New (Valid TOML) |
|----------|------------------|
| `[[project]]` array | `[bundles.project.name]` named |
| `[server:user@ip]` | `[servers.name]` + `host = "user@ip"` |
| `"cd {host}/src"` bare | Inside `run = [...]` array |
| `[project.0]` reference | `"@project.name"` in run array |
| `![database]` | `disconnect = true` |

---

## Config Format Designed

**Location**: `~/.config/panssh/config.toml`

### Syntax Reference

| Feature | Syntax | Example |
|---------|--------|---------|
| Named bundle | `[bundles.group.name]` | `[bundles.project.nvim]` |
| Pane target | `pane = N` | `pane = 0` |
| Commands | `commands = [...]` | `commands = ["cd ~", "ls"]` |
| Layout | `layout = "..."` | `"tiled"`, `"vertical"`, `"horizontal"` |
| Server | `[servers.name]` | `[servers.wizard]` |
| Host | `host = "user@ip"` | `host = "admin@10.0.0.1"` |
| Run sequence | `run = [...]` | `run = ["cmd", "@bundle.ref"]` |
| Bundle ref | `"@group.name"` | `"@project.nvim-init"` |
| Run all | `"@group.*"` | `"@project.*"` |
| Disconnect | `disconnect = true` | Closes SSH after run |
| Template vars | `{user}`, `{ip}` | Extracted from `host` |

---

## CLI Interface

```
panssh -b <bundle> -n <num> [-v | -H] [-l]

Options:
  -b, --bundle   Bundle group or server name
  -n, --num      Number of panes
  -v             Vertical split (side by side)
  -H             Horizontal split (stacked)
  -l, --list     List available bundles/servers
```

**Layout precedence**: CLI flag > bundle property > defaults > tiled

---

## Project Structure

```
~/Projects/panssh/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Public exports
│   ├── cli.rs            # Clap argument parsing
│   ├── config.rs         # Config structs + serde
│   ├── loader.rs         # Config file discovery + loading
│   ├── resolver.rs       # @reference resolution (Phase 2)
│   ├── tmux.rs           # Tmux pane/window operations
│   ├── ssh.rs            # SSH session handling (Phase 2)
│   ├── interpolate.rs    # {user}, {ip} expansion (Phase 2)
│   └── error.rs          # Error types (thiserror)
└── examples/
    └── config.toml       # Example config for users
```

---

## All Scaffolded Code

### Cargo.toml

```toml
[package]
name = "panssh"
version = "0.1.0"
edition = "2024"
description = "Tmux pane orchestrator with SSH support"

[dependencies]
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
thiserror = "2"
dirs = "6"
```

---

### src/lib.rs

```rust
//! PanSSH - Tmux pane orchestrator with SSH support.

pub mod cli;
pub mod config;
pub mod error;
pub mod interpolate;
pub mod loader;
pub mod resolver;
pub mod ssh;
pub mod tmux;

pub use config::{BundleEntry, Config, Layout};
pub use error::{PansshError, Result};
```

---

### src/error.rs

```rust
//! Error types for panssh.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PansshError {
    // TODO: Add error variants
    // Examples:
    // #[error("Config file not found: {0}")]
    // ConfigNotFound(std::path::PathBuf),
    //
    // #[error("Failed to parse config: {0}")]
    // ParseError(#[from] toml::de::Error),
    //
    // #[error("Bundle not found: {0}")]
    // BundleNotFound(String),
    //
    // #[error("Tmux error: {0}")]
    // TmuxError(String),
}

pub type Result<T> = std::result::Result<T, PansshError>;
```

---

### src/config.rs

```rust
//! Configuration types for panssh.
//!
//! These structs map directly to the TOML config format.

use serde::Deserialize;
use std::collections::HashMap;

/// Top-level config structure.
#[derive(Debug, Deserialize)]
pub struct Config {
    // TODO: Implement fields
    // pub defaults: Option<Defaults>,
    // pub bundles: HashMap<String, HashMap<String, BundleEntry>>,
    // pub servers: Option<HashMap<String, ServerConfig>>,
}

/// Global defaults.
#[derive(Debug, Deserialize, Default)]
pub struct Defaults {
    // TODO: Implement fields
    // pub layout: Option<Layout>,
}

/// A single bundle entry.
#[derive(Debug, Deserialize)]
pub struct BundleEntry {
    // TODO: Implement fields
    // pub pane: Option<u32>,
    // pub commands: Vec<String>,
    // pub role: Option<String>,
    // pub layout: Option<Layout>,
}

/// Layout options for tmux panes.
#[derive(Debug, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    #[default]
    Tiled,
    Vertical,
    Horizontal,
}

/// Server/SSH configuration (Phase 2).
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    // TODO: Implement fields
    // pub host: String,
    // pub disconnect: Option<bool>,
    // pub run: Vec<String>,
}
```

---

### src/loader.rs

```rust
//! Config file loading and parsing.

use crate::config::Config;
use crate::error::Result;
use std::path::PathBuf;

/// Returns the default config path: ~/.config/panssh/config.toml
pub fn default_config_path() -> Option<PathBuf> {
    // TODO: Use dirs::config_dir() to get ~/.config
    // Then append "panssh/config.toml"
    todo!()
}

/// Load and parse the config file.
pub fn load_config(path: &PathBuf) -> Result<Config> {
    // TODO:
    // 1. Read file contents with std::fs::read_to_string
    // 2. Parse with toml::from_str
    // 3. Return Config or error
    todo!()
}
```

---

### src/cli.rs

```rust
//! CLI argument parsing with clap.

use clap::Parser;

/// Tmux pane orchestrator with SSH support.
#[derive(Parser, Debug)]
#[command(name = "panssh")]
#[command(about = "Create tmux panes with SSH bundles")]
pub struct Cli {
    // TODO: Add arguments matching original interface:
    //
    // /// Bundle name from config
    // #[arg(short, long)]
    // pub bundle: Option<String>,
    //
    // /// Number of panes
    // #[arg(short, long)]
    // pub num: Option<u32>,
    //
    // /// Vertical split (side by side)
    // #[arg(short = 'v')]
    // pub vertical: bool,
    //
    // /// Horizontal split (stacked)
    // #[arg(short = 'H')]
    // pub horizontal: bool,
    //
    // /// List available bundles
    // #[arg(short, long)]
    // pub list: bool,
}

impl Cli {
    /// Determine the effective layout from CLI flags.
    pub fn layout(&self) -> Option<crate::config::Layout> {
        // TODO: Return Vertical if -v, Horizontal if -H, None otherwise
        todo!()
    }
}
```

---

### src/tmux.rs

```rust
//! Tmux pane and window management.

use crate::config::Layout;
use crate::error::Result;

/// Create N panes with the specified layout.
pub fn create_panes(num: u32, layout: Layout) -> Result<()> {
    // TODO:
    // 1. First pane is current (no split needed)
    // 2. For remaining panes, run: tmux split-window [-h|-v]
    // 3. After each split, run: tmux select-layout <layout>
    //
    // Use std::process::Command to run tmux
    //
    // Layout mapping:
    //   Tiled -> "tiled"
    //   Vertical -> "even-horizontal" (side by side)
    //   Horizontal -> "even-vertical" (stacked)
    todo!()
}

/// Send keys (command) to a specific pane.
pub fn send_keys(pane: u32, command: &str) -> Result<()> {
    // TODO: Run: tmux send-keys -t {pane} "{command}" Enter
    todo!()
}

/// Apply a layout to the current window.
pub fn set_layout(layout: Layout) -> Result<()> {
    // TODO: Run: tmux select-layout <layout>
    todo!()
}
```

---

### src/main.rs

```rust
//! PanSSH CLI entry point.

use clap::Parser;
use panssh::cli::Cli;

fn main() {
    let cli = Cli::parse();

    // TODO: Implement main flow:
    //
    // 1. Load config
    //    let config_path = loader::default_config_path()
    //        .expect("Could not determine config path");
    //    let config = loader::load_config(&config_path)
    //        .expect("Failed to load config");
    //
    // 2. Handle --list flag
    //    if cli.list {
    //        // Print available bundles
    //        return;
    //    }
    //
    // 3. Get bundle name and pane count
    //    let bundle = cli.bundle.expect("-b/--bundle required");
    //    let num = cli.num.expect("-n/--num required");
    //
    // 4. Look up bundle group in config.bundles
    //
    // 5. Determine layout (CLI override > bundle > default)
    //
    // 6. Create tmux panes
    //    tmux::create_panes(num, layout)?;
    //
    // 7. Send commands to each pane
    //    for (name, entry) in bundle_group {
    //        if let Some(pane) = entry.pane {
    //            for cmd in &entry.commands {
    //                tmux::send_keys(pane, cmd)?;
    //            }
    //        }
    //    }

    println!("panssh - not yet implemented");
}
```

---

### src/resolver.rs (Phase 2)

```rust
//! Bundle reference resolution (Phase 2).
//!
//! Parses @group.name and @group.* syntax from run arrays.

/// A resolved reference from a run array.
#[derive(Debug)]
pub enum ResolvedRef {
    /// A plain command string.
    Command(String),
    /// Reference to a specific bundle: @group.name
    BundleRef { group: String, name: String },
    /// Reference to all bundles in group: @group.*
    GroupAll { group: String },
}

/// Parse a string from a run array into a ResolvedRef.
pub fn parse_ref(s: &str) -> ResolvedRef {
    // TODO:
    // 1. If starts with "@", it's a reference
    // 2. Split on "." to get group and name
    // 3. If name is "*", return GroupAll
    // 4. Otherwise return BundleRef
    // 5. If not starting with "@", return Command
    todo!()
}
```

---

### src/interpolate.rs (Phase 2)

```rust
//! Variable interpolation (Phase 2).
//!
//! Handles {user} and {ip} expansion in commands.

/// Extract user and ip from "user@ip" host string.
pub fn parse_host(host: &str) -> Option<(String, String)> {
    // TODO: Split on "@", return (user, ip)
    todo!()
}

/// Interpolate {user} and {ip} in a command string.
pub fn interpolate(command: &str, user: &str, ip: &str) -> String {
    // TODO: Replace {user} with user, {ip} with ip
    todo!()
}
```

---

### src/ssh.rs (Phase 2)

```rust
//! SSH session handling (Phase 2).
//!
//! For now, SSH is handled by sending `ssh user@host` to a tmux pane.

use crate::error::Result;

/// Start an SSH session in the current pane.
pub fn connect(host: &str) -> Result<()> {
    // TODO: Use tmux send-keys to run: ssh {host}
    todo!()
}

/// Disconnect from SSH (send exit or Ctrl-D).
pub fn disconnect() -> Result<()> {
    // TODO: Send "exit" to current pane
    todo!()
}
```

---

### examples/config.toml

```toml
# Example PanSSH config
# Copy to: ~/.config/panssh/config.toml

[defaults]
layout = "tiled"

# ─────────────────────────────────────────────
# BUNDLES - Local workflows (no SSH)
# ─────────────────────────────────────────────

[bundles.dev.frontend]
pane = 0
commands = ["cd ~/Projects/app && npm run dev"]
layout = "horizontal"

[bundles.dev.backend]
pane = 1
commands = ["cd ~/Projects/api && cargo watch -x run"]

[bundles.dev.logs]
pane = 2
commands = ["tail -f /var/log/app.log"]

[bundles.project.nvim-init]
pane = 0
commands = ["cd ~/.config/nvim && nvim init.vim"]
role = "primary"
layout = "vertical"

[bundles.project.nvim-keymaps]
pane = 1
commands = ["cd ~/.config/nvim", "nvim keymaps.lua"]

# ─────────────────────────────────────────────
# SERVERS - SSH contexts (Phase 2)
# ─────────────────────────────────────────────

[servers.wizard]
host = "wizardenginer@192.21.11.1"
disconnect = true
run = [
  "cd {user}/src",
  "@project.nvim-init",
  "@project.nvim-keymaps",
]

[servers.database]
host = "admin@192.21.23.1"
disconnect = true
run = ["cd /var/lib/postgres"]
```

---

## Implementation Phases

### Phase 1: MVP (Local bundles + tmux)

| Step | File | Task |
|------|------|------|
| 1 | `config.rs` | Uncomment struct fields |
| 2 | `error.rs` | Add `ConfigNotFound`, `ParseError`, `TmuxError`, `BundleNotFound` |
| 3 | `loader.rs` | Use `dirs::config_dir()` + `toml::from_str()` |
| 4 | `cli.rs` | Uncomment clap `#[arg]` attributes |
| 5 | `tmux.rs` | Implement with `std::process::Command` |
| 6 | `main.rs` | Wire CLI → loader → tmux |

**MVP delivers**: `panssh -b dev -n 3 -v` creates 3 vertical panes, runs commands

### Phase 2: References + Servers

| Step | File | Task |
|------|------|------|
| 7 | `resolver.rs` | Parse `@group.name` and `@group.*` strings |
| 8 | `interpolate.rs` | `{user}`, `{ip}` variable expansion |
| 9 | `ssh.rs` | SSH connection via tmux send-keys |
| 10 | `main.rs` | Add server execution path |

### Phase 3: Polish

| Step | File | Task |
|------|------|------|
| 11 | `cli.rs` | Add `init`, `check`, `--dry-run` subcommands |
| 12 | `tmux.rs` | Multi-window support |
| 13 | - | Install script for `~/.local/bin` |

---

## Where We Left Off

**All files scaffolded with TODO comments**. Project compiles with warnings (expected since all functions are `todo!()`).

User wants to implement the code themselves.

---

## TODO: Next Steps

1. [ ] `config.rs` - Uncomment struct fields, add derives
2. [ ] `error.rs` - Add error variants
3. [ ] `loader.rs` - Implement `default_config_path()` and `load_config()`
4. [ ] `cli.rs` - Uncomment clap arguments
5. [ ] `tmux.rs` - Implement tmux commands with `std::process::Command`
6. [ ] `main.rs` - Wire everything together
7. [ ] Create `~/.config/panssh/config.toml` from examples/

---

## Key Files to Reference

- **Full design plan**: `~/.claude/plans/frolicking-jingling-sketch.md`
- **Example config**: `~/Projects/panssh/examples/config.toml`
- **Original Python POC**: `~/.local/bin/panssh`
- **Original DSL idea**: `~/.local/bin/test.toml`

---

## Commands to Resume

```bash
cd ~/Projects/panssh
cargo check        # Verify compiles
cargo run -- -l    # Test (will hit todo!() until implemented)

# Set up config for testing
mkdir -p ~/.config/panssh
cp examples/config.toml ~/.config/panssh/config.toml
```
