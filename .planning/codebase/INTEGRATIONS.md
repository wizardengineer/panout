# External Integrations

**Analysis Date:** 2026-02-24

## APIs & External Services

**None Integrated**

Panout does not make network calls or connect to external APIs. All operations are local subprocess execution.

## System-Level Integrations

**Tmux:**
- Type: Terminal multiplexer integration
- Used for: Creating panes, windows, sending commands, managing layouts
- Implementation: Subprocess calls via `std::process::Command`
- Location: `src/tmux.rs`
- Operations:
  - `tmux split-window` - Create new panes
  - `tmux send-keys` - Send commands to panes
  - `tmux select-layout` - Apply pane layouts
  - `tmux new-window` - Create windows
  - `tmux select-window` - Switch active window
  - `tmux list-panes -F #{pane_index}` - Query pane indices
  - `tmux display-message -p #{window_index}` - Query window index
- Required: Must be running inside an active tmux session
- Error handling: Returns `PanoutError::NotInTmux` if not in tmux, `PanoutError::TmuxError` for command failures

**SSH:**
- Type: Remote host connectivity
- Used for: Connecting to remote servers from tmux panes
- Implementation: Sends `ssh user@host` commands via tmux send-keys
- Location: `src/ssh.rs` and `src/main.rs` (workspace execution)
- Operations:
  - SSH connections with user@ip format
  - Directory traversal via `ssh -t host "cd dir && exec $SHELL -l"`
  - No SSH key configuration (uses system SSH config)
- Required: SSH client binary available in PATH, valid SSH credentials
- Authentication: Uses system SSH authentication (keys, agents, passwords)

## Data Storage

**Configuration File (TOML):**
- Type: Local filesystem TOML
- Location: `~/.config/panout/config.toml` (or `$XDG_CONFIG_HOME/panout/config.toml`)
- Format: TOML with nested tables
- Parser: `toml` crate 0.8
- Parsing: `src/config.rs::Config::from_str()`
- Discovery: `src/loader.rs::default_config_path()` with fallback logic
- No database: Pure file-based configuration

**File I/O:**
- Reading: `std::fs::read_to_string()` in `src/loader.rs`
- No writing: Panout only reads config files, never modifies them
- No caching layer

## Environment Variables

**Read by Panout:**
- `XDG_CONFIG_HOME` - Checked first for config directory (optional)
  - If set: `$XDG_CONFIG_HOME/panout/config.toml`
  - If not set: Falls back to `~/.config/panout/config.toml`
  - Location: `src/loader.rs::default_config_path()`

- `TMUX` - Checked to verify running inside tmux session
  - Must be present (set by tmux automatically)
  - Location: `src/tmux.rs::in_tmux()`

**Config Variable Interpolation:**
- `{user}` and `{ip}` placeholders in commands (future feature)
- Location: `src/interpolate.rs` (module exists but not yet integrated)
- Status: Module defined but not used in current codebase

**No .env Files:**
- Panout does not use dotenv or environment variable files
- All configuration comes from TOML

## Command Execution

**Local Commands:**
- Method: `std::process::Command` for subprocess execution
- Used in: Bundle execution, tmux operations
- Examples:
  - `cd ~/directory` commands sent to panes
  - `npm run dev`, `cargo watch` commands
  - Any shell command in bundle definition

**Remote Commands:**
- Method: SSH via tmux send-keys
- No direct SSH library integration (uses system ssh binary)
- Example flow: `ssh user@host "cd dir && exec $SHELL -l"`
- Error propagation: SSH errors not directly caught (failures handled by tmux command exit codes)

## No Authentication Services

- No OAuth/OpenID Connect
- No JWT token management
- No API key handling in code (users manage SSH keys via system config)
- SSH uses system authentication (key files, agents, etc.)

## No Caching or Storage Services

- No Redis
- No Memcached
- No local SQLite
- All state is ephemeral (per-invocation)

## No Logging/Monitoring Services

- No error tracking (Sentry, etc.)
- No analytics
- Simple stderr output for errors
- All logging is to terminal via `eprintln!` macro

## No Message Queues

- No async task processing
- Synchronous execution only

## Webhooks & Callbacks

**None Implemented**

Panout is a CLI tool that responds to user invocation, not a service that handles inbound webhooks.

## Cross-Origin/Network Policy

**Not Applicable**

Panout is a local CLI tool with no web server, no API server, and no network listeners.

## Subprocess Integration Points

**Tmux Command Execution:**
```
User invokes: panout -b dev.frontend
→ Parses TOML config
→ Calls tmux commands:
  - tmux split-window (create panes)
  - tmux select-layout (apply layout)
  - tmux send-keys (send commands to panes)
```

**SSH Command Execution:**
```
User invokes: panout -w myproject
→ Reads workspace config with host="user@host"
→ For each pane:
  - Calls tmux send-keys with: ssh -t user@host "cd ~/dir && exec $SHELL -l"
  - Shell connects and opens SSH session
```

## Summary: Minimal Integration Surface

Panout is intentionally minimalist:
- **No external APIs** - Pure local operation
- **No credentials management** - Relies on system SSH config
- **No network dependencies** - Only SSH (system binary)
- **No background services** - Single synchronous execution
- **No persistent state** - Ephemeral per invocation
- **Single external system dependency** - tmux (required) + ssh (optional)

---

*Integration audit: 2026-02-24*
