# Technology Stack

**Analysis Date:** 2026-02-24

## Languages

**Primary:**
- Rust 2024 edition - Core application language for CLI binary and library

## Runtime

**Environment:**
- Linux/macOS (tested on macOS, designed for cross-platform use)
- Requires tmux to be running (enforced by runtime check in `src/tmux.rs`)
- Requires SSH client binary for remote workspace functionality

**Build System:**
- Cargo (Rust's package manager)
- Lockfile: `Cargo.lock` present

## Frameworks & Core Libraries

**CLI:**
- clap 4.5.54 - Command-line argument parsing with derive macros
  - Location: `src/cli.rs` - Argument parsing and validation
  - Features used: `derive` feature for struct-based CLI definition

**Serialization:**
- serde 1.0 - Serialization/deserialization framework
  - Features used: `derive` feature for derive macros
  - serde integration via `toml` crate for TOML parsing

**Configuration:**
- toml 0.8 - TOML parsing and deserialization
  - Location: `src/config.rs` - Parses `~/.config/panout/config.toml`
  - Handles dynamic table structure for bundle groups

**Error Handling:**
- thiserror 2 - Derive macro for Error types
  - Location: `src/error.rs` - Custom PanoutError enum with Display impls

**System/Path Utilities:**
- dirs 6 - Cross-platform directory discovery
  - Location: `src/loader.rs` - Finds config directory via XDG_CONFIG_HOME, ~/.config, or platform defaults

## Key Dependencies

**Critical:**
- clap - Enables declarative CLI with validation and help generation
- serde + toml - Enables flexible TOML config parsing (key feature of panout)
- dirs - Enables cross-platform config file discovery (Linux/macOS/Windows support)
- thiserror - Enables ergonomic error handling with automatic Display/Debug impls

**Development/Integration:**
- None declared in Cargo.toml (no dev-dependencies or test frameworks)

## Configuration

**Environment Variables:**
- `XDG_CONFIG_HOME` - Checked first for config file location (optional)
- `TMUX` - Checked to verify runtime is inside a tmux session (set by tmux automatically)

**Build Configuration:**
- `Cargo.toml` - Package metadata, version 0.1.0, MIT license

**Runtime Config:**
- TOML file at `~/.config/panout/config.toml` (or `$XDG_CONFIG_HOME/panout/config.toml`)
- No other configuration files (no .env, yaml, or JSON configs)
- Discovery logic: `src/loader.rs::default_config_path()`

## Platform Requirements

**Development:**
- Rust 1.70+ (mentioned in README)
- Cargo for building

**Runtime:**
- tmux (must be running - enforced in `src/tmux.rs::in_tmux()`)
- SSH client binary (for workspace feature with remote hosts)
- POSIX shell (/bin/sh or compatible) - used in SSH commands

**Installation:**
```bash
cargo install --path .
# or
cargo build --release
cp target/release/panout ~/.local/bin/
```

## Binary Output

**Name:** `panout` (as declared in `Cargo.toml`)
**Entry Point:** `src/main.rs`
**Library:** Also exposed as library via `src/lib.rs` with public modules

## Architecture Notes

The codebase is organized as both a binary and library:

**Binary (`main.rs`):**
- Parses CLI args via `clap` in `src/cli.rs`
- Loads config via `src/loader.rs` (finds + parses TOML)
- Resolves bundle references in `src/resolver.rs`
- Executes tmux operations in `src/tmux.rs`
- Handles SSH via `src/ssh.rs`

**Core Modules:**
- `config.rs` - Serde-derived types for TOML structure (BundleEntry, Workspace, etc.)
- `loader.rs` - File I/O and path discovery
- `resolver.rs` - Bundle reference expansion (@group.name syntax)
- `tmux.rs` - Process execution via Command for tmux operations
- `error.rs` - Custom error type using thiserror

**No External Service Dependencies:**
- No network calls (pure local/SSH)
- No database connections
- No API integrations
- All operations are local subprocess execution (tmux, ssh)

---

*Stack analysis: 2026-02-24*
