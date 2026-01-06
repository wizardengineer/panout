# Panout

A tmux pane orchestrator that creates windows and panes from TOML configuration.

Panout automates the creation of tmux layouts for development workflows. Define your pane arrangements, commands, and SSH connections in a config file, then spawn them with a single command.

## Features

- **Bundles**: Named command groups with optional pane targeting
- **Bundle References**: Use `@group.name` to compose bundles from other bundles
- **Workspaces**: Multi-window configurations with automatic SSH connection
- **Layouts**: Tiled, vertical (side-by-side), or horizontal (stacked) pane arrangements
- **SSH Integration**: Connect to remote hosts and cd to directories in one command
- **Cross-Platform Config**: Respects XDG_CONFIG_HOME and works on Linux/macOS

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
cp target/release/panout ~/.local/bin/
```

## Quick Start

1. Create a config file at `~/.config/panout/config.toml`:

```toml
[defaults]
layout = "tiled"

# Simple bundle - runs a command
[dev.frontend]
cmd = "npm run dev"

# Bundle with multiple commands
[dev.backend]
cmd = ["cd ~/api", "cargo watch -x run"]

# Bundle that references other bundles
[dev.all]
cmd = ["@dev.frontend", "@dev.backend"]
```

2. Run from inside tmux:

```bash
# Create 2 vertical panes running the frontend bundle
panout -b dev.frontend -n 2 -v

# List all available bundles
panout --list
```

## Usage

```
panout [OPTIONS]

Options:
  -b, --bundle <GROUP.NAME>    Bundle to run
  -w, --workspace <NAME>       Workspace to run (creates multiple windows)
  -n, --num <COUNT>            Number of panes to create
  -v                           Vertical split (panes side by side)
  -H                           Horizontal split (panes stacked)
  -l, --list                   List available bundles, workspaces, and servers
  -h, --help                   Print help
  -V, --version                Print version
```

## Configuration

Config file location (checked in order):
1. `$XDG_CONFIG_HOME/panout/config.toml`
2. `~/.config/panout/config.toml`

### Bundles

Bundles define commands to run in panes:

```toml
[group.name]
cmd = "command"           # Single command
cmd = ["cmd1", "cmd2"]    # Multiple commands
pane = 0                  # Target pane (optional, auto-assigned)
layout = "vertical"       # Layout override (optional)
```

### Bundle References

Bundles can reference other bundles using `@group.name` syntax:

```toml
[dev.frontend]
cmd = "npm run dev"

[dev.backend]
cmd = "cargo run"

[dev.all]
cmd = ["@dev.frontend", "@dev.backend"]

# Reference all bundles in a group
[dev.everything]
cmd = ["@dev.*"]
```

### Workspaces

Workspaces create multiple windows, optionally with SSH:

```toml
[workspace.myproject]
host = "user@server.com"      # SSH host (optional)
dir = "~/src/myproject"       # Directory to cd into
windows = [
    { panes = 2, layout = "vertical" },
    { panes = 4 },
]
```

Run with:
```bash
panout -w myproject
```

This creates:
- Window 1: 2 vertical panes, each SSH'd to server and cd'd to directory
- Window 2: 4 tiled panes, same SSH + cd

### Layouts

| Layout | Flag | Description |
|--------|------|-------------|
| `tiled` | (default) | Spread panes evenly |
| `vertical` | `-v` | Side-by-side panes |
| `horizontal` | `-H` | Stacked panes |

Layout precedence: CLI flag > bundle config > defaults > tiled

### Full Example Config

```toml
[defaults]
layout = "tiled"

# Development bundles
[dev.frontend]
pane = 0
cmd = "cd ~/app && npm run dev"

[dev.backend]
pane = 1
cmd = ["cd ~/api", "cargo watch -x run"]

[dev.logs]
pane = 2
cmd = "tail -f /var/log/app.log"

[dev.all]
cmd = ["@dev.frontend", "@dev.backend", "@dev.logs"]

# Quick commands
[quick.htop]
cmd = "htop"

[quick.logs]
cmd = "journalctl -f"

# Workspaces for remote development
[workspace.staging]
host = "deploy@staging.example.com"
dir = "~/app"
windows = [
    { panes = 2, layout = "vertical", name = "code" },
    { panes = 4, name = "services" },
]

[workspace.production]
host = "deploy@prod.example.com"
dir = "~/app"
windows = [
    { panes = 2, layout = "vertical" },
]

# Server definitions (for future use)
[servers.staging]
host = "deploy@staging.example.com"
disconnect = true
cmd = "cd ~/app && git status"
```

## Requirements

- Rust 1.70+ (for building)
- tmux (must be run inside a tmux session)
- SSH client (for remote workspaces)

## Architecture

```
src/
├── lib.rs          # Public API exports
├── main.rs         # CLI entry point
├── cli.rs          # Argument parsing (clap)
├── config.rs       # TOML configuration types
├── loader.rs       # Config file discovery
├── resolver.rs     # @ref expansion with cycle detection
├── tmux.rs         # Tmux pane/window operations
├── ssh.rs          # SSH session helpers
├── interpolate.rs  # {user}/{ip} variable expansion
└── error.rs        # Error types (thiserror)
```

## Documentation

Generate and view the API documentation:

```bash
cargo doc --open
```

## License

MIT
