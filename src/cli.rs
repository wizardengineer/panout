//! Command-line interface for panout.
//!
//! Parses arguments using clap and provides the [`Cli`] struct containing
//! all user-specified options.

use crate::config::Layout;
use clap::Parser;

/// Command-line arguments for panout.
///
/// # Examples
///
/// ```bash
/// # Run a bundle with 3 vertical panes
/// panout -b dev.frontend -n 3 -v
///
/// # Run a workspace (multiple windows)
/// panout -w myproject
///
/// # List all available bundles and workspaces
/// panout --list
/// ```
#[derive(Parser, Debug)]
#[command(name = "panout")]
#[command(version)]
#[command(about = "Tmux pane orchestrator - create panes and windows from config")]
#[command(long_about = "Panout creates tmux panes and windows based on TOML configuration.\n\n\
    Define bundles for local commands or workspaces for multi-window SSH setups,\n\
    then spawn them with a single command.")]
pub struct Cli {
    /// Bundle to run (format: group.name).
    ///
    /// Bundles are defined in your config as `[group.name]` sections.
    #[arg(short, long, value_name = "GROUP.NAME")]
    pub bundle: Option<String>,

    /// Workspace to run (creates multiple windows).
    ///
    /// Workspaces are defined under `[workspace.name]` and can include
    /// SSH host and directory settings.
    #[arg(short = 'w', long, value_name = "NAME")]
    pub workspace: Option<String>,

    /// Number of panes to create.
    #[arg(short, long, value_name = "COUNT")]
    pub num: Option<u32>,

    /// Use vertical layout (side-by-side panes).
    #[arg(short = 'v', help = "Vertical split (panes side by side)")]
    pub vertical: bool,

    /// Use horizontal layout (stacked panes).
    #[arg(short = 'H', help = "Horizontal split (panes stacked)")]
    pub horizontal: bool,

    /// List all available bundles, workspaces, and servers.
    #[arg(short, long)]
    pub list: bool,
}

impl Cli {
    /// Determine the layout from CLI flags.
    ///
    /// Returns `Some(Layout)` if `-v` or `-H` was specified, `None` otherwise.
    /// When `None`, the layout falls back to bundle config or defaults.
    pub fn layout(&self) -> Option<Layout> {
        if self.vertical {
            Some(Layout::Vertical)
        } else if self.horizontal {
            Some(Layout::Horizontal)
        } else {
            None
        }
    }
}
