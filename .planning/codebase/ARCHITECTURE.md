# Architecture

**Analysis Date:** 2026-02-24

## Pattern Overview

**Overall:** Modular CLI application with layered configuration resolution and shell orchestration.

**Key Characteristics:**
- Configuration-driven: All behavior derives from TOML config files
- Resolver pattern: Bundle references (`@ref`) expand recursively with cycle detection
- Shell command delegation: All tmux/SSH operations execute via subprocess calls
- Error propagation: Custom error types with detailed context via `thiserror`
- No side effects until execution: Config parsing separated from tmux operations

## Layers

**CLI Layer:**
- Purpose: Parse command-line arguments and route to appropriate handlers
- Location: `src/cli.rs`
- Contains: Argument parsing with `clap`, layout flag resolution
- Depends on: `config` module for Layout type
- Used by: `main.rs` for initial user input handling

**Configuration Layer:**
- Purpose: Deserialize TOML into type-safe Rust structures
- Location: `src/config.rs`
- Contains: Config, BundleEntry, Workspace, WindowDef, Layout, Cmd types
- Depends on: `serde` for deserialization
- Used by: `loader`, `resolver`, `main` for config access

**Loading Layer:**
- Purpose: Discover and load config files from filesystem
- Location: `src/loader.rs`
- Contains: Config file discovery (XDG_CONFIG_HOME, ~/.config, platform defaults), file I/O
- Depends on: `dirs` crate for home directory detection, `config` module for parsing
- Used by: `main.rs` for initial config load

**Resolution Layer:**
- Purpose: Expand bundle references (`@group.name`, `@group.*`) with cycle detection
- Location: `src/resolver.rs`
- Contains: Reference parsing, recursive expansion, pane assignment tracking
- Depends on: `config` module, `error` module
- Used by: `main.rs` before tmux operations

**Execution Layer:**
- Purpose: Interface with tmux and SSH via shell commands
- Locations: `src/tmux.rs`, `src/ssh.rs`
- Contains: Pane/window creation, command dispatch, layout application
- Depends on: `std::process::Command` for shell execution
- Used by: `main.rs` for realizing bundles/workspaces

**Utility Layers:**
- `src/interpolate.rs`: Variable substitution for {user}/{ip} in commands
- `src/error.rs`: Unified error type with contextual messages

## Data Flow

**Bundle Execution Flow:**

1. **Input** → CLI parses arguments (`-b dev.frontend -n 2 -v`)
2. **Load** → `loader::load_default_config()` discovers and reads TOML from filesystem
3. **Parse** → `config.from_str()` deserializes TOML into Config struct with nested hashmaps
4. **Lookup** → `config.get_bundle()` retrieves BundleEntry from `bundles[group][name]`
5. **Resolve** → `resolver::resolve_with_panes()` recursively expands `@ref` and collects commands by pane
6. **Layout** → CLI flag/bundle config/defaults resolved in precedence order
7. **Panes** → `tmux::create_panes()` calls `tmux split-window` N-1 times, applies layout
8. **Send** → `tmux::send_keys()` dispatches resolved commands to pane targets
9. **Exit** → Commands execute in panes; parent process exits

**Workspace Execution Flow:**

1. **Input** → CLI parses `-w myproject`
2. **Load/Parse** → Same as bundle flow
3. **Lookup** → `config.get_workspace()` retrieves Workspace with windows array
4. **For each window:**
   - Create new window (except first, reuses current)
   - Create panes with workspace layout
   - If `host` specified: send `ssh host` command
   - If `dir` specified: send `cd dir` command (combined with ssh if both)
   - Send window-specific commands if present
5. **Restore** → Switch back to original window

**State Management:**
- No persistent state; all state flows through function calls
- Cycle detection during resolution uses `HashSet<String>` visited set passed through recursion
- Pane index mapping handled by querying tmux directly (`list-panes -F #{pane_index}`)
- Layout application happens incrementally after each split via `select-layout`

## Key Abstractions

**Cmd (Enum):**
- Purpose: Normalize config syntax for single vs. multiple commands
- Examples: `src/config.rs` lines 35-60
- Pattern: `#[serde(untagged)]` allows deserializer to accept both `"cmd"` and `["cmd1", "cmd2"]`, exposed via `to_vec()` method

**Layout (Enum):**
- Purpose: Map user-friendly names to tmux layout algorithms
- Examples: `src/config.rs` lines 62-89
- Pattern: Enum with `to_tmux_layout()` method providing tmux command mapping
- Precedence: CLI flag (highest) > bundle config > defaults > tiled (lowest)

**ResolvedRef (Enum):**
- Purpose: Distinguish command strings from reference syntax during resolution
- Examples: `src/resolver.rs` lines 30-45
- Pattern: Parse `@group.name` (BundleRef), `@group.*` (GroupAll), plain strings (Command)
- Used by: Recursive resolver to expand references depth-first while preventing cycles

**Config (HashMap of HashMaps):**
- Purpose: Organize bundles hierarchically by group and name
- Structure: `bundles: HashMap<group: String, HashMap<name: String, BundleEntry>>`
- Access: `config.get_bundle("group.name")` splits on `.` to navigate nested maps
- Used by: All lookup operations; allows fast O(1) access to bundles

**Workspace (Array of WindowDefs):**
- Purpose: Define multi-window layouts with shared SSH/directory context
- Structure: `windows: Vec<WindowDef>` with optional `host` and `dir` on parent
- Used by: Workspace execution to create multiple windows with inherited connection info

## Entry Points

**CLI Binary (`main.rs`):**
- Location: `src/main.rs`
- Triggers: User runs `panout` command with arguments
- Responsibilities:
  - Call `Cli::parse()` to parse command-line arguments
  - Call `loader::load_default_config()` to load TOML
  - Route to `run_bundle()` or `run_workspace()` based on flags
  - Call `print_listings()` if `--list` flag provided
  - Exit with code 1 on error

**Bundle Handler (`run_bundle`):**
- Location: `src/main.rs` lines 59-87
- Triggered by: `-b GROUP.NAME` argument
- Responsibilities:
  - Resolve layout precedence (CLI > bundle config > defaults)
  - Call `resolver::resolve_with_panes()` to expand `@refs` and collect commands
  - Call `tmux::create_panes()` to create and layout panes
  - Call `tmux::send_keys()` for each resolved command

**Workspace Handler (`run_workspace`):**
- Location: `src/main.rs` lines 90-100
- Triggered by: `-w NAME` argument
- Responsibilities:
  - Retrieve workspace from config
  - Save current window index
  - Call `run_workspace_windows()` to create all windows
  - Restore original window

**Window Creator (`run_workspace_windows`):**
- Location: `src/main.rs` lines 103-146
- Triggered by: Workspace execution
- Responsibilities:
  - Create new window for each workspace window (skip for first)
  - Apply layout to each window
  - Send SSH commands if `host` specified
  - Send directory change commands if `dir` specified
  - Send window-specific commands if present

## Error Handling

**Strategy:** Custom error enum with exhaustive variant coverage; all errors flow to top-level handler in `main()` that prints to stderr and exits with code 1.

**Patterns:**

**Configuration Errors:**
- `ConfigNotFound(PathBuf)`: Config file missing from expected location
- `NoConfigDir`: Home directory cannot be determined (no fallback)
- `ParseError(toml::de::Error)`: TOML syntax or schema mismatch
- `IoError(std::io::Error)`: File read failures

**Reference Errors:**
- `BundleNotFound(String)`: Requested bundle or group doesn't exist
- `CircularRef(String)`: Cycle detected during recursive expansion (e.g., A→B→A)
- `InvalidRef(String)`: Malformed reference syntax (currently unused; invalid refs treated as plain commands)

**Execution Errors:**
- `NotInTmux`: `$TMUX` environment variable not set; not running in tmux session
- `TmuxError(String)`: Subprocess call failed or returned non-zero status
- `WorkspaceNotFound(String)`: Named workspace missing from config
- `ServerNotFound(String)`: Named server missing from config (reserved for future use)

**Error propagation:** All functions return `Result<T> = std::result::Result<T, PanoutError>`. The top-level `run()` function uses `?` operator; errors bubble to `main()` which prints via `eprintln!` and exits.

## Cross-Cutting Concerns

**Logging:** None; no logging framework. Informational output only via `print_listings()` which uses `println!`.

**Validation:** Occurs at three stages:
- Deserialization: `serde` validates TOML schema during `Config::from_str()`
- Lookup: Missing bundles/workspaces fail at access time with clear errors
- Execution: Cycle detection during reference resolution prevents infinite recursion

**Authentication:** SSH handled entirely by system `ssh` command (no key management). Commands like `ssh -t user@host "cd dir && exec $SHELL -l"` passed directly to tmux panes.

**Pane Indexing:** Accounts for `pane-base-index` tmux config by querying actual indices from `tmux list-panes` rather than assuming 0-based.

---

*Architecture analysis: 2026-02-24*
