# Codebase Structure

**Analysis Date:** 2026-02-24

## Directory Layout

```
panssh/
├── Cargo.toml                 # Rust project manifest with dependencies
├── Cargo.lock                 # Dependency lock file
├── README.md                  # User documentation and quick start
├── assets/                    # Non-code assets (images, etc.)
│   └── panda.png             # Mascot image
├── src/                       # All source code
│   ├── main.rs               # Binary entry point and orchestration
│   ├── lib.rs                # Public API exports
│   ├── cli.rs                # Command-line argument parsing (clap)
│   ├── config.rs             # TOML configuration types and parsing
│   ├── loader.rs             # Config file discovery and loading
│   ├── resolver.rs           # Bundle reference expansion with cycle detection
│   ├── tmux.rs               # Tmux pane and window operations
│   ├── ssh.rs                # SSH session helpers
│   ├── interpolate.rs        # Variable substitution ({user}, {ip})
│   └── error.rs              # Error types (thiserror)
├── examples/                  # Example configurations or scripts
├── target/                    # Build artifacts (ignored in git)
└── .planning/                 # Planning and documentation
    └── codebase/             # Architecture/structure analysis documents
```

## Directory Purposes

**src/:**
- Purpose: All Rust source code for the library and binary
- Contains: Modular implementation split by concern (config, CLI, tmux, resolution)
- Key files: `main.rs` (binary entry), `lib.rs` (public API), all other files are modules

**assets/:**
- Purpose: Non-code media files
- Contains: README mascot image (panda.png)

**examples/:**
- Purpose: Example configurations or helper scripts (currently minimal)

**target/:**
- Purpose: Build output directory created by `cargo build`
- Not committed to git

**.planning/:**
- Purpose: Documentation and planning artifacts
- Contains: ARCHITECTURE.md, STRUCTURE.md (this file), and other analysis documents

## Key File Locations

**Entry Points:**

- `src/main.rs`: Binary entry point
  - Contains `main()` function that calls `run()` with error handling
  - Contains orchestration: routing to bundle/workspace handlers
  - Contains print_listings(), run_bundle(), run_workspace(), run_workspace_windows()

**Configuration:**

- `Cargo.toml`: Project metadata, version, dependencies (toml, serde, clap, thiserror, dirs)
- `src/config.rs`: Config struct, BundleEntry, Workspace, WindowDef, Layout, Cmd types

**Core Logic:**

- `src/cli.rs`: Cli struct with clap derives, argument parsing, layout() method
- `src/loader.rs`: Config file discovery (XDG_CONFIG_HOME, ~/.config), file reading
- `src/resolver.rs`: Bundle reference parsing (@group.name, @group.*), recursive expansion, cycle detection
- `src/tmux.rs`: Tmux operations (create_panes, send_keys, set_layout, create_window, select_window, current_window, pane_indices)
- `src/ssh.rs`: SSH helpers (connect, disconnect) via tmux pane delegation
- `src/error.rs`: PanoutError enum with thiserror derives
- `src/interpolate.rs`: {user}/{ip} variable substitution for commands
- `src/lib.rs`: Public API exports (config types, error types, module declarations)

**Testing:**

- Tests embedded within modules: `src/resolver.rs` has #[cfg(test)] block with parse_ref tests
- Tests in `src/interpolate.rs` for parse_host and interpolate functions
- Run with: `cargo test`

## Naming Conventions

**Files:**
- `src/*.rs`: Lowercase module names matching Rust conventions (cli.rs, config.rs, etc.)
- Main module `main.rs` is entry point
- Utility modules named by function (loader.rs, resolver.rs, interpolate.rs, error.rs)
- Platform code: None; tmux/SSH abstracted as subprocess calls

**Directories:**
- `src/`: All source code (no internal subdirectories; flat module structure)
- `examples/`: Example configurations
- `assets/`: Media files
- `.planning/`: Documentation

**Functions:**
- Rust conventions: snake_case (create_panes, resolve_bundle, parse_host)
- Public functions documented with doc comments `///`
- Private helpers prefixed with reasonable names (e.g., resolve_bundle_inner)

**Types:**
- PascalCase: Config, BundleEntry, Workspace, WindowDef, Layout, Cmd, ServerConfig
- Enums: Layout (Tiled, Vertical, Horizontal), Cmd (Single, Multiple), ResolvedRef (Command, BundleRef, GroupAll)
- Error enum: PanoutError

**Constants:**
- None defined; layout names mapped via `to_tmux_layout()` method

**Imports:**
- Order: std library → external crates → internal modules
- Path style: Full paths for clarity (use crate::config::Config)
- Wildcard imports used sparingly (only in tests and match statements)

## Where to Add New Code

**New Feature (e.g., new command type):**
- Primary code: `src/main.rs` (add handler function)
- Config definition: `src/config.rs` (add struct variant or field)
- Error handling: `src/error.rs` (add error variant if needed)
- Tests: Inline in relevant module with #[cfg(test)]

**New Bundle Type or Config Option:**
- Add to `BundleEntry` or new struct in `src/config.rs`
- Update `Config::from_str()` parsing if new top-level config key
- Add to relevant handler in `src/main.rs` if execution logic needed

**New Tmux Operation:**
- Implementation: `src/tmux.rs`
- Pattern: Wrap Command execution, check status, map errors to PanoutError::TmuxError
- Example: Similar to `create_window()` or `send_keys()`

**New SSH Feature:**
- Simple delegation: Add to `src/ssh.rs` as thin wrapper around tmux::send_keys
- Complex logic: May warrant new module

**Utility Functions:**
- Variable expansion: Add to `src/interpolate.rs`
- General helpers: Consider if they belong in existing modules or warrant new utility module

**Testing:**
- Unit tests: #[cfg(test)] mod tests blocks within relevant .rs file
- Test data: Inline in test functions or consider fixtures if shared
- Run: `cargo test`

## Special Directories

**target/:**
- Purpose: Cargo build output (debug, release artifacts, dependencies)
- Generated: Yes (created by `cargo build`)
- Committed: No (in .gitignore)

**examples/:**
- Purpose: Example TOML configurations or scripts
- Generated: No
- Committed: Yes (if present; currently minimal)

**.claude/:**
- Purpose: Claude-specific project configuration and documentation
- Contains: specs, docs, sum (summaries), commands directories
- Committed: No (in .gitignore as of recent changes)

**.planning/codebase/:**
- Purpose: Architecture and structure documentation
- Contains: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md, CONCERNS.md
- Generated: Yes (by `gsd map-codebase`)
- Committed: Yes (design documentation)

## Module Dependencies Graph

```
main.rs
├── cli (command-line parsing)
│   └── config (Layout type)
├── loader (file discovery and reading)
│   ├── config (Config struct)
│   └── error (Result type)
├── resolver (reference expansion)
│   ├── config (Config struct)
│   └── error (PanoutError, Result)
├── tmux (pane/window operations)
│   ├── config (Layout type)
│   └── error (PanoutError, Result)
├── config (types)
└── error (error handling)

lib.rs (public exports)
├── cli
├── config
├── error
├── interpolate
├── loader
├── resolver
├── ssh
└── tmux

ssh.rs (SSH helpers)
└── tmux (delegation)

interpolate.rs (no internal dependencies)

error.rs (no internal dependencies)
```

## Build and Test Structure

**Build:**
- Binary target: `panout` (from src/main.rs)
- Library target: `panout` (from src/lib.rs, for test imports)
- Build command: `cargo build` or `cargo build --release`
- Output: `target/debug/panout` or `target/release/panout`

**Testing:**
- Test runner: `cargo test`
- Tests location: Inline with modules (resolver.rs, interpolate.rs have test blocks)
- Coverage: Unit tests for reference parsing and variable interpolation; integration testing limited
- Requires: Tmux to be available for tmux-dependent tests (skipped if TMUX env var not set)

---

*Structure analysis: 2026-02-24*
