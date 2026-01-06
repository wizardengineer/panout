//! Configuration types for panout.
//!
//! This module defines the data structures that map to the TOML configuration format.
//! The config uses a simple structure where:
//!
//! - `defaults` and `servers` and `workspace` are reserved top-level keys
//! - Everything else is treated as a bundle group
//!
//! # Config Format
//!
//! ```toml
//! [defaults]
//! layout = "tiled"
//!
//! [dev.frontend]
//! cmd = "npm run dev"
//! pane = 0
//!
//! [dev.backend]
//! cmd = ["cd ~/api", "cargo run"]
//! pane = 1
//!
//! [workspace.myproject]
//! host = "user@server"
//! dir = "~/src/project"
//! windows = [
//!     { panes = 2, layout = "vertical" },
//!     { panes = 4 },
//! ]
//! ```

use serde::Deserialize;
use std::collections::HashMap;

/// Command field that accepts either a single string or array of strings.
///
/// This allows flexible config syntax:
/// ```toml
/// cmd = "single command"
/// # or
/// cmd = ["command 1", "command 2"]
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Cmd {
    /// A single command string.
    Single(String),
    /// Multiple commands executed in sequence.
    Multiple(Vec<String>),
}

impl Cmd {
    /// Convert to a `Vec<String>`, normalizing both variants.
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Cmd::Single(s) => vec![s.clone()],
            Cmd::Multiple(v) => v.clone(),
        }
    }
}

/// Layout options for tmux panes.
///
/// Maps to tmux's built-in layout algorithms:
/// - `Tiled`: Spread panes evenly in both directions
/// - `Vertical`: Side-by-side panes (tmux's "even-horizontal")
/// - `Horizontal`: Stacked panes (tmux's "even-vertical")
#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// Spread panes evenly (tmux: "tiled").
    #[default]
    Tiled,
    /// Side-by-side panes (tmux: "even-horizontal").
    Vertical,
    /// Stacked panes (tmux: "even-vertical").
    Horizontal,
}

impl Layout {
    /// Convert to the tmux layout name used by `select-layout`.
    pub fn to_tmux_layout(&self) -> &'static str {
        match self {
            Layout::Tiled => "tiled",
            Layout::Vertical => "even-horizontal",
            Layout::Horizontal => "even-vertical",
        }
    }
}

/// Global default settings applied when not overridden.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Defaults {
    /// Default layout for panes when not specified elsewhere.
    pub layout: Option<Layout>,
}

/// A single bundle entry defining commands for a pane.
///
/// Bundles are the basic unit of configuration. Each bundle specifies
/// commands to run and optionally which pane to target.
///
/// # Example
///
/// ```toml
/// [dev.frontend]
/// cmd = "npm run dev"
/// pane = 0
/// layout = "vertical"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct BundleEntry {
    /// Commands to execute. Can reference other bundles with `@group.name`.
    pub cmd: Cmd,
    /// Target pane index (0-based logical index, auto-assigned if omitted).
    #[serde(default)]
    pub pane: Option<u32>,
    /// Optional role identifier (e.g., "primary", "secondary").
    #[serde(default)]
    pub role: Option<String>,
    /// Layout override for this bundle.
    #[serde(default)]
    pub layout: Option<Layout>,
}

/// SSH server configuration for remote connections.
///
/// # Example
///
/// ```toml
/// [servers.prod]
/// host = "admin@192.168.1.100"
/// disconnect = true
/// cmd = "cd /var/log && tail -f app.log"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// SSH host in `user@ip` format.
    pub host: String,
    /// Whether to disconnect (send "exit") after commands complete.
    #[serde(default)]
    pub disconnect: bool,
    /// Commands to run after connecting.
    #[serde(default)]
    pub cmd: Option<Cmd>,
}

/// A window definition within a workspace.
///
/// Each window in a workspace can have its own pane count, layout, and commands.
#[derive(Debug, Clone, Deserialize)]
pub struct WindowDef {
    /// Number of panes to create in this window.
    pub panes: u32,
    /// Layout for panes (defaults to workspace default or tiled).
    #[serde(default)]
    pub layout: Option<Layout>,
    /// Commands to run in each pane of this window.
    #[serde(default)]
    pub cmd: Option<Cmd>,
    /// Optional tmux window name.
    #[serde(default)]
    pub name: Option<String>,
}

/// A workspace with multiple windows, optionally connected via SSH.
///
/// Workspaces allow defining multi-window tmux layouts in a single config block.
/// When `host` is specified, each pane will SSH to the remote server.
///
/// # Example
///
/// ```toml
/// [workspace.dev]
/// host = "user@devserver"
/// dir = "~/src/myproject"
/// windows = [
///     { panes = 2, layout = "vertical" },  # Window 1: 2 side-by-side panes
///     { panes = 4 },                        # Window 2: 4 tiled panes
/// ]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    /// SSH host (`user@ip`). If set, each pane will SSH to this host.
    #[serde(default)]
    pub host: Option<String>,
    /// Base directory. Combined with `host`, creates: `ssh -t host "cd dir && exec $SHELL -l"`
    #[serde(default)]
    pub dir: Option<String>,
    /// Window definitions for this workspace.
    pub windows: Vec<WindowDef>,
}

/// Top-level configuration structure.
///
/// Parsed from `~/.config/panout/config.toml` (or XDG equivalent).
/// Reserved keys are `defaults`, `servers`, and `workspace`.
/// All other top-level keys are treated as bundle groups.
#[derive(Debug, Default)]
pub struct Config {
    /// Global default settings.
    pub defaults: Defaults,
    /// Named SSH server configurations.
    pub servers: HashMap<String, ServerConfig>,
    /// Bundle groups: `group_name` -> `entry_name` -> `BundleEntry`.
    pub bundles: HashMap<String, HashMap<String, BundleEntry>>,
    /// Named workspaces for multi-window configurations.
    pub workspaces: HashMap<String, Workspace>,
}

impl Config {
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
        let raw: toml::Value = toml::from_str(toml_str)?;
        let table = raw.as_table().cloned().unwrap_or_default();

        let mut config = Config::default();

        for (key, value) in table {
            match key.as_str() {
                "defaults" => {
                    config.defaults = value.try_into()?;
                }
                "servers" => {
                    config.servers = value.try_into()?;
                }
                "workspace" => {
                    config.workspaces = value.try_into()?;
                }
                _ => {
                    let entries: HashMap<String, BundleEntry> = value.try_into()?;
                    config.bundles.insert(key, entries);
                }
            }
        }

        Ok(config)
    }

    /// Look up a bundle by its `group.name` path.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bundle = config.get_bundle("dev.frontend");
    /// ```
    pub fn get_bundle(&self, path: &str) -> Option<&BundleEntry> {
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        let (group, name) = (parts[0], parts[1]);
        self.bundles.get(group).and_then(|g| g.get(name))
    }

    /// Get all bundles in a group.
    pub fn get_group(&self, group: &str) -> Option<&HashMap<String, BundleEntry>> {
        self.bundles.get(group)
    }

    /// List all bundle paths in `group.name` format, sorted alphabetically.
    pub fn list_bundles(&self) -> Vec<String> {
        let mut result = Vec::new();
        for (group, entries) in &self.bundles {
            for name in entries.keys() {
                result.push(format!("{}.{}", group, name));
            }
        }
        result.sort();
        result
    }

    /// List all server names, sorted alphabetically.
    pub fn list_servers(&self) -> Vec<String> {
        let mut result: Vec<_> = self.servers.keys().cloned().collect();
        result.sort();
        result
    }

    /// Get a workspace by name.
    pub fn get_workspace(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.get(name)
    }

    /// List all workspace names, sorted alphabetically.
    pub fn list_workspaces(&self) -> Vec<String> {
        let mut result: Vec<_> = self.workspaces.keys().cloned().collect();
        result.sort();
        result
    }
}
