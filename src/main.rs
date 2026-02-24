//! Panout CLI entry point.
//!
//! This binary provides the `panout` command for creating tmux panes and
//! windows from TOML configuration.

use clap::Parser;
use panout::cli::Cli;
use panout::config::{Config, Layout, Workspace};
use panout::error::Result;
use panout::{loader, resolver, session, tmux, PanoutError};

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
///
/// Bundles targeting SSH servers (detected via resolved commands matching
/// known server hosts) route through the session module to create
/// persistent remote tmux sessions. Other bundles send commands to
/// local panes as before.
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

    // Check if pane 0 commands include an SSH connection to a known server
    let server_host = find_server_host(&pane_commands, config);

    if let Some(host) = server_host {
        // Remote bundle: create named tmux session on remote host
        let cmd = session::build_remote_session_cmd(
            &host,
            bundle_name,
            None,
        );
        if let Some(&pane) = pane_indices.first() {
            tmux::send_keys(pane, &cmd)?;
        }
    } else {
        // Local bundle: send commands to panes as before
        for (i, commands) in pane_commands {
            if let Some(&actual_pane) = pane_indices.get(i as usize) {
                for cmd in commands {
                    tmux::send_keys(actual_pane, &cmd)?;
                }
            }
        }
    }

    Ok(())
}

/// Check if resolved pane commands contain an SSH command targeting a known server.
///
/// Scans all resolved pane commands for `ssh <host>` patterns where `<host>`
/// matches a server host from the config. Returns the first matching host.
fn find_server_host(
    pane_commands: &[(u32, Vec<String>)],
    config: &Config,
) -> Option<String> {
    let known_hosts: Vec<&str> = config
        .servers
        .values()
        .map(|s| s.host.as_str())
        .collect();

    for (_, commands) in pane_commands {
        for cmd in commands {
            if let Some(rest) = cmd.strip_prefix("ssh ") {
                let target = rest.trim();
                if known_hosts
                    .iter()
                    .any(|h| target == *h || target.ends_with(h))
                {
                    return Some(target.to_string());
                }
            }
        }
    }
    None
}

/// Execute a workspace configuration (multiple windows with optional SSH).
///
/// Remote workspaces (with `host` set) create a persistent named tmux
/// session on the remote host via SSH. Local workspaces create windows
/// and panes as before.
fn run_workspace(config: &Config, name: &str) -> Result<()> {
    let workspace = config
        .get_workspace(name)
        .ok_or_else(|| PanoutError::WorkspaceNotFound(name.into()))?;

    match &workspace.host {
        Some(host) => {
            // Remote session: SSH into host with named tmux session
            let cmd = session::build_remote_session_cmd(
                host,
                name,
                workspace.dir.as_deref(),
            );
            let panes = tmux::pane_indices()?;
            tmux::send_keys(panes[0], &cmd)?;
        }
        None => {
            // Local workspace: create windows/panes as before
            let start_window = tmux::current_window()?;
            run_workspace_windows(workspace)?;
            tmux::select_window(start_window)?;
        }
    }

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
