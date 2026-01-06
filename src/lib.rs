//! # Panout
//!
//! A tmux pane orchestrator that creates windows and panes from TOML configuration.
//!
//! Panout automates the creation of tmux layouts for development workflows. Define
//! your pane arrangements, commands, and SSH connections in a config file, then
//! spawn them with a single command.
//!
//! ## Features
//!
//! - **Bundles**: Named command groups that can reference each other with `@ref` syntax
//! - **Workspaces**: Multi-window configurations with SSH support
//! - **Layouts**: Tiled, vertical, or horizontal pane arrangements
//! - **SSH Integration**: Automatically connect to remote hosts and cd to directories
//!
//! ## Quick Example
//!
//! ```toml
//! # ~/.config/panout/config.toml
//!
//! [dev.frontend]
//! cmd = "npm run dev"
//!
//! [dev.backend]
//! cmd = "cargo watch -x run"
//!
//! [workspace.myproject]
//! host = "user@server.com"
//! dir = "~/src/myproject"
//! windows = [
//!     { panes = 2, layout = "vertical" },
//!     { panes = 4 },
//! ]
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into these modules:
//!
//! - [`config`]: TOML configuration parsing and data structures
//! - [`cli`]: Command-line argument parsing with clap
//! - [`loader`]: Config file discovery and loading
//! - [`resolver`]: Bundle reference (`@ref`) expansion
//! - [`tmux`]: Tmux pane and window operations
//! - [`ssh`]: SSH session management
//! - [`interpolate`]: Variable substitution (`{user}`, `{ip}`)
//! - [`error`]: Error types

pub mod cli;
pub mod config;
pub mod error;
pub mod interpolate;
pub mod loader;
pub mod resolver;
pub mod ssh;
pub mod tmux;

pub use config::{BundleEntry, Cmd, Config, Layout, WindowDef, Workspace};
pub use error::{PanoutError, Result};
