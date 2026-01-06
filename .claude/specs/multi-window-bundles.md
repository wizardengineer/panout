# Multi-Window Bundle Support

**Status**: Proposed
**Created**: 2026-01-06

---

## Problem

Currently `panout` only creates panes within a single window. User workflow requires running multiple commands across different windows:

```bash
# Window 1: 2 vertical panes
panssh -b instafix -n 2 -v

# Window 2: 4 tiled panes
panssh -b instafix -n 4
```

User wants to automate this into a single bundle invocation.

---

## Original JSON Config (for reference)

```json
{
    "default_host": "wizardengineer@192.168.5.25",
    "bundles": {
        "instafix": {"dir": "~/src/instafix"},
        "installvm": {"dir": "~/src/instafix-llvm"}
    }
}
```

---

## Proposed TOML Config Format

### Option A: Window blocks within bundles

```toml
[defaults]
layout = "tiled"
default_host = "wizardengineer@192.168.5.25"

# Simple bundle (single window, current behavior)
[instafix.main]
cmd = "cd ~/src/instafix"

# Multi-window bundle using [[windows]] array
[workspace.instafix]
windows = [
    { panes = 2, layout = "vertical", cmd = "cd ~/src/instafix" },
    { panes = 4, layout = "tiled", cmd = "cd ~/src/instafix" },
]

[workspace.installvm]
windows = [
    { panes = 2, layout = "vertical", cmd = "cd ~/src/instafix-llvm" },
    { panes = 4, cmd = "cd ~/src/instafix-llvm" },
]
```

### Option B: Explicit window definitions

```toml
[defaults]
layout = "tiled"
default_host = "wizardengineer@192.168.5.25"

# Window definitions
[windows.instafix-edit]
panes = 2
layout = "vertical"
cmd = "cd ~/src/instafix"

[windows.instafix-build]
panes = 4
layout = "tiled"
cmd = "cd ~/src/instafix"

# Workspace groups windows together
[workspace.instafix]
windows = ["instafix-edit", "instafix-build"]
```

### Option C: Inline window array (most concise)

```toml
[defaults]
default_host = "wizardengineer@192.168.5.25"

[workspace.instafix]
dir = "~/src/instafix"
windows = [
    { panes = 2, layout = "vertical" },
    { panes = 4 },
]

[workspace.installvm]
dir = "~/src/instafix-llvm"
windows = [
    { panes = 2, layout = "vertical" },
    { panes = 4 },
]
```

**Recommendation**: Option C - most concise, `dir` applies to all windows.

---

## CLI Changes

```bash
# Run a workspace (creates multiple windows)
panout -w instafix

# Current behavior unchanged
panout -b dev.frontend -n 2 -v
```

New flag: `-w, --workspace <name>` to run multi-window configs.

---

## Implementation Plan

### Step 1: Add tmux window operations to `tmux.rs`

```rust
/// Create a new window and return its index.
pub fn create_window(name: Option<&str>) -> Result<u32>

/// Switch to a specific window.
pub fn select_window(index: u32) -> Result<()>

/// Get current window index.
pub fn current_window() -> Result<u32>
```

tmux commands:
- `tmux new-window [-n name]` - create window
- `tmux select-window -t N` - switch to window
- `tmux display-message -p '#{window_index}'` - get current window

### Step 2: Add config types in `config.rs`

```rust
/// A window definition within a workspace.
#[derive(Debug, Clone, Deserialize)]
pub struct WindowDef {
    pub panes: u32,
    #[serde(default)]
    pub layout: Option<Layout>,
    #[serde(default)]
    pub cmd: Option<Cmd>,
    #[serde(default)]
    pub name: Option<String>,
}

/// A workspace with multiple windows.
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub dir: Option<String>,
    pub windows: Vec<WindowDef>,
}
```

### Step 3: Update `Config` parsing

Add `workspaces` as a reserved key alongside `defaults` and `servers`:

```rust
pub struct Config {
    pub defaults: Defaults,
    pub servers: HashMap<String, ServerConfig>,
    pub bundles: HashMap<String, HashMap<String, BundleEntry>>,
    pub workspaces: HashMap<String, Workspace>,  // NEW
}
```

### Step 4: Add CLI argument

```rust
/// Workspace name (creates multiple windows)
#[arg(short = 'w', long)]
pub workspace: Option<String>,
```

### Step 5: Implement workspace execution in `main.rs`

```rust
fn run_workspace(config: &Config, name: &str) -> Result<()> {
    let workspace = config.workspaces.get(name)
        .ok_or_else(|| PanoutError::WorkspaceNotFound(name.into()))?;

    let base_cmd = workspace.dir.as_ref()
        .map(|d| format!("cd {}", d));

    for (i, win) in workspace.windows.iter().enumerate() {
        // Create window (skip for first, use current)
        if i > 0 {
            tmux::create_window(win.name.as_deref())?;
        }

        // Create panes
        let layout = win.layout.unwrap_or(Layout::Tiled);
        tmux::create_panes(win.panes, layout)?;

        // Send commands to each pane
        for pane in 0..win.panes {
            if let Some(ref cmd) = base_cmd {
                tmux::send_keys(pane, cmd)?;
            }
            if let Some(ref cmd) = win.cmd {
                for c in cmd.to_vec() {
                    tmux::send_keys(pane, &c)?;
                }
            }
        }
    }

    // Return to first window
    tmux::select_window(0)?;

    Ok(())
}
```

### Step 6: Add error variant

```rust
#[error("Workspace not found: {0}")]
WorkspaceNotFound(String),
```

### Step 7: Update `--list` output

```rust
if !config.workspaces.is_empty() {
    println!("\nWorkspaces:");
    for name in config.workspaces.keys() {
        println!("  {}", name);
    }
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/config.rs` | Add `WindowDef`, `Workspace` structs; add `workspaces` to `Config` |
| `src/tmux.rs` | Add `create_window()`, `select_window()`, `current_window()` |
| `src/cli.rs` | Add `-w, --workspace` argument |
| `src/error.rs` | Add `WorkspaceNotFound` variant |
| `src/main.rs` | Add workspace execution path |
| `examples/config.toml` | Add workspace examples |

---

## Testing

1. Unit tests for new tmux functions (mock or skip in CI)
2. Manual test in tmux:
   ```bash
   panout -w instafix
   # Should create 2 windows: first with 2 vertical panes, second with 4 tiled
   ```

---

## Future Considerations

- Window naming: `tmux new-window -n "build"`
- Window-specific host (SSH into different servers per window)
- Ability to reference existing bundles in window `cmd`
