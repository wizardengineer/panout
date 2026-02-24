# Coding Conventions

**Analysis Date:** 2026-02-24

## Naming Patterns

**Files:**
- Module files use lowercase snake_case: `cli.rs`, `config.rs`, `error.rs`, `loader.rs`, `resolver.rs`, `tmux.rs`, `ssh.rs`, `interpolate.rs`
- Binary entry point: `main.rs`
- Library root: `lib.rs`

**Functions:**
- All functions use lowercase snake_case: `resolve_bundle()`, `parse_ref()`, `create_panes()`, `send_keys()`, `default_config_path()`
- Private helper functions follow the same convention: `resolve_bundle_inner()`, `resolve_with_panes_inner()`

**Variables:**
- Local variables and parameters use lowercase snake_case: `bundle_path`, `pane_cmds`, `num_panes`, `target_pane`, `direct_cmds`
- Enum variants in non-unit enums are PascalCase: `Cmd::Single()`, `Cmd::Multiple()`, `Layout::Tiled`, `ResolvedRef::Command()`

**Types:**
- Structs and enums use PascalCase: `Cli`, `Config`, `BundleEntry`, `ServerConfig`, `WindowDef`, `Workspace`, `Layout`, `Cmd`, `ResolvedRef`, `Defaults`, `PanoutError`
- Type aliases use PascalCase: `Result<T>` (public type alias for `std::result::Result<T, PanoutError>`)

## Code Style

**Formatting:**
- Rust standard formatting via `rustfmt` (implicit, no custom config file present)
- 4-space indentation (Rust default)
- Line length: Standard (no explicit limit enforced via configuration)

**Linting:**
- No explicit clippy configuration file present
- Code follows Rust idioms and best practices

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate imports (`use serde::...`, `use clap::...`, `use thiserror::...`)
3. Internal crate imports (`use crate::...`)

**Example from `src/resolver.rs`:**
```rust
use crate::config::Config;
use crate::error::{PanoutError, Result};
use std::collections::HashSet;
```

**Path Aliases:**
- Standard library modules imported with full paths: `std::path::PathBuf`, `std::io::Error`, `std::process::Command`
- Crate-relative imports use `crate::` prefix: `crate::config::Config`, `crate::error::Result`, `crate::tmux`

## Error Handling

**Pattern:**
- All errors use the custom `PanoutError` enum from `src/error.rs`
- Result type is aliased as `Result<T>` = `std::result::Result<T, PanoutError>`
- Functions that can fail return `Result<T>` (never use unwrap/expect in library code)

**Error Variants with Documentation:**
- `ConfigNotFound(PathBuf)` - Config file not found
- `NoConfigDir` - Cannot determine config directory
- `IoError` - File I/O failures (via `#[from]`)
- `ParseError` - TOML parsing failures (via `#[from]`)
- `BundleNotFound(String)` - Missing bundle/group
- `ServerNotFound(String)` - Missing server
- `WorkspaceNotFound(String)` - Missing workspace
- `InvalidRef(String)` - Malformed bundle reference
- `CircularRef(String)` - Circular reference in bundles
- `TmuxError(String)` - Tmux command failures
- `NotInTmux` - Not running inside tmux session

**Usage in main.rs:**
```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

## Logging

**Framework:** Uses `println!()` and `eprintln!()` macros only

**Patterns:**
- Error output: `eprintln!("Error: {}", e)`
- User output: `println!("...")` (see `print_listings()` in `src/main.rs`)
- No structured logging framework (serde_json, tracing, etc.)

## Comments

**When to Comment:**
- Module-level documentation: Every module has `//!` doc comments (see all `*.rs` files)
- Function documentation: Every public function has `///` doc comments with descriptions
- Complex logic: Inline comments explain non-obvious implementations
- Example in `src/tmux.rs`: Comments explain how `pane-base-index` configuration is handled

**JSDoc/TSDoc:**
- Rust uses `///` for doc comments (not JSDoc)
- Doc comments include examples, error conditions, and argument descriptions
- Example from `src/config.rs`:
```rust
/// Parse config from a TOML string.
///
/// Reserved keys (`defaults`, `servers`, `workspace`) are parsed into their
/// respective fields. All other keys are treated as bundle groups.
///
/// # Errors
///
/// Returns `toml::de::Error` if the TOML is malformed or doesn't match
/// the expected structure.
pub fn from_str(toml_str: &str) -> Result<Self, toml::de::Error> {
```

## Function Design

**Size:** Functions are concise and focused. Largest functions are around 50 lines (`run_workspace_windows()` at 43 lines, `Config::from_str()` at 25 lines)

**Parameters:** Most functions take 1-3 parameters
- `create_panes(num: u32, layout: Layout)` - 2 params
- `send_keys(pane: u32, command: &str)` - 2 params
- `resolve_bundle_inner(config: &Config, bundle_path: &str, visited: &mut HashSet<String>)` - 3 params

**Return Values:**
- Public functions return `Result<T>` or concrete types
- Most operations return `Result<()>` (unit result indicating success/failure)
- Some query operations return actual data: `pane_indices() -> Result<Vec<u32>>`, `current_window() -> Result<u32>`

## Module Design

**Exports:**
- `src/lib.rs` re-exports public API: `pub use config::...`, `pub use error::...`
- Each module exposes only necessary public items via `pub` keyword
- Example from `src/lib.rs`:
```rust
pub mod cli;
pub mod config;
pub mod error;
// ... more modules
pub use config::{BundleEntry, Cmd, Config, Layout, WindowDef, Workspace};
pub use error::{PanoutError, Result};
```

**Barrel Files:**
- `src/lib.rs` acts as the crate's public interface
- Re-exports key types from submodules for convenience
- No dedicated barrel files within submodules

## Type System

**Enums for Variants:**
- `Layout` enum for layout options (Tiled, Vertical, Horizontal)
- `Cmd` enum for single vs. multiple commands
- `ResolvedRef` enum for reference parsing (Command, BundleRef, GroupAll)

**Impl Blocks:**
- Methods on types use `impl Type { ... }`
- Example: `impl Cmd { pub fn to_vec(&self) -> Vec<String> { ... } }`
- Conversion methods: `impl Layout { pub fn to_tmux_layout(&self) -> &'static str { ... } }`

## Trait Usage

**Serde Derives:**
- `#[derive(Deserialize)]` for config structs
- `#[serde(untagged)]` for enum deserialization (Cmd enum)
- `#[serde(rename_all = "lowercase")]` for Layout enum
- `#[serde(default)]` for optional config fields

**Error Trait:**
- All errors derive `#[derive(Error, Debug)]` from `thiserror` crate
- Use `#[error(...)]` attributes for error messages
- Use `#[from]` for automatic error conversion (e.g., `IoError`, `ParseError`)

---

*Convention analysis: 2026-02-24*
