//! Panout CLI entry point.
//!
//! This binary provides the `panout` command for creating tmux panes and
//! windows from TOML configuration.

use clap::Parser;
use panout::cli::Cli;
use panout::config::{Config, Layout, Workspace};
use panout::error::Result;
use panout::{loader, resolver, tmux, PanoutError};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main application logic.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = loader::load_default_config()?;

    if cli.list {
        print_listings(&config);
        return Ok(());
    }

    if let Some(ref ws_name) = cli.workspace {
        return run_workspace(&config, ws_name);
    }

    run_bundle(&cli, &config)
}

/// Print all available bundles, workspaces, and servers.
fn print_listings(config: &Config) {
    if !config.bundles.is_empty() {
        println!("Bundles:");
        for bundle in config.list_bundles() {
            println!("  {}", bundle);
        }
    }
    if !config.workspaces.is_empty() {
        println!("\nWorkspaces:");
        for ws in config.list_workspaces() {
            println!("  {}", ws);
        }
    }
    if !config.servers.is_empty() {
        println!("\nServers:");
        for server in config.list_servers() {
            println!("  {}", server);
        }
    }
}

/// Execute a bundle configuration.
fn run_bundle(cli: &Cli, config: &Config) -> Result<()> {
    let bundle_name = cli
        .bundle
        .as_ref()
        .ok_or_else(|| PanoutError::BundleNotFound("no bundle specified".into()))?;

    let num_panes = cli.num.unwrap_or(1);
    let pane_commands = resolver::resolve_with_panes(config, bundle_name)?;

    // Layout precedence: CLI flag > bundle config > defaults > tiled
    let bundle = config.get_bundle(bundle_name);
    let layout = cli
        .layout()
        .or_else(|| bundle.and_then(|b| b.layout))
        .or(config.defaults.layout)
        .unwrap_or(Layout::Tiled);

    let pane_indices = tmux::create_panes(num_panes, layout)?;

    for (i, commands) in pane_commands {
        if let Some(&actual_pane) = pane_indices.get(i as usize) {
            for cmd in commands {
                tmux::send_keys(actual_pane, &cmd)?;
            }
        }
    }

    Ok(())
}

/// Execute a workspace configuration (multiple windows with optional SSH).
fn run_workspace(config: &Config, name: &str) -> Result<()> {
    let workspace = config
        .get_workspace(name)
        .ok_or_else(|| PanoutError::WorkspaceNotFound(name.into()))?;

    let start_window = tmux::current_window()?;
    run_workspace_windows(workspace)?;
    tmux::select_window(start_window)?;

    Ok(())
}

/// Create all windows defined in a workspace.
fn run_workspace_windows(workspace: &Workspace) -> Result<()> {
    for (i, win) in workspace.windows.iter().enumerate() {
        if i > 0 {
            tmux::create_window(win.name.as_deref())?;
        }

        let layout = win.layout.unwrap_or(Layout::Tiled);
        let pane_indices = tmux::create_panes(win.panes, layout)?;

        for pane in pane_indices {
            match (&workspace.host, &workspace.dir) {
                // SSH + cd: single command that connects and changes directory
                (Some(host), Some(dir)) => {
                    let cmd = format!(
                        "ssh -t {} \"cd {} && exec \\$SHELL -l\"",
                        host, dir
                    );
                    tmux::send_keys(pane, &cmd)?;
                }
                // SSH only
                (Some(host), None) => {
                    let cmd = format!("ssh {}", host);
                    tmux::send_keys(pane, &cmd)?;
                }
                // Local cd only
                (None, Some(dir)) => {
                    let cmd = format!("cd {}", dir);
                    tmux::send_keys(pane, &cmd)?;
                }
                // No host or dir
                (None, None) => {}
            }

            // Window-specific commands
            if let Some(ref cmd) = win.cmd {
                for c in cmd.to_vec() {
                    tmux::send_keys(pane, &c)?;
                }
            }
        }
    }

    Ok(())
}
