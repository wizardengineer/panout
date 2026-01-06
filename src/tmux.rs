//! Tmux pane and window management.
//!
//! Provides functions to create and manage tmux panes and windows via
//! shell commands. All operations require being run inside a tmux session.
//!
//! # Pane Indices
//!
//! Tmux allows configuring `pane-base-index`, so panes might start at 0 or 1.
//! Functions in this module handle this by querying actual pane indices from tmux.

use crate::config::Layout;
use crate::error::{PanoutError, Result};
use std::process::Command;

/// Check if we're running inside a tmux session.
///
/// Checks for the `TMUX` environment variable, which tmux sets when active.
pub fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Create N panes in the current window with the specified layout.
///
/// The first pane is the existing pane; additional panes are created via `split-window`.
/// Layout is applied after each split to maintain balance.
///
/// Returns the actual pane indices (accounting for `pane-base-index` config).
///
/// # Errors
///
/// - [`PanoutError::NotInTmux`] if not running inside tmux
/// - [`PanoutError::TmuxError`] if a tmux command fails
pub fn create_panes(num: u32, layout: Layout) -> Result<Vec<u32>> {
    if !in_tmux() {
        return Err(PanoutError::NotInTmux);
    }

    for _ in 1..num {
        let status = Command::new("tmux")
            .args(["split-window"])
            .status()
            .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

        if !status.success() {
            return Err(PanoutError::TmuxError("split-window failed".into()));
        }

        set_layout(layout)?;
    }

    pane_indices()
}

/// Send keystrokes to a specific pane.
///
/// Sends the command string followed by Enter to execute it.
///
/// # Arguments
///
/// * `pane` - The pane index (as returned by [`pane_indices`])
/// * `command` - The command string to send
pub fn send_keys(pane: u32, command: &str) -> Result<()> {
    let pane_target = format!("{}", pane);
    let status = Command::new("tmux")
        .args(["send-keys", "-t", &pane_target, command, "Enter"])
        .status()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !status.success() {
        return Err(PanoutError::TmuxError(format!(
            "send-keys to pane {} failed",
            pane
        )));
    }

    Ok(())
}

/// Apply a layout to the current window.
///
/// Uses tmux's `select-layout` command with the appropriate layout name.
pub fn set_layout(layout: Layout) -> Result<()> {
    let layout_name = layout.to_tmux_layout();
    let status = Command::new("tmux")
        .args(["select-layout", layout_name])
        .status()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !status.success() {
        return Err(PanoutError::TmuxError(format!(
            "select-layout {} failed",
            layout_name
        )));
    }

    Ok(())
}

/// Select (focus) a specific pane.
pub fn select_pane(pane: u32) -> Result<()> {
    let pane_target = format!("{}", pane);
    let status = Command::new("tmux")
        .args(["select-pane", "-t", &pane_target])
        .status()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !status.success() {
        return Err(PanoutError::TmuxError(format!(
            "select-pane {} failed",
            pane
        )));
    }

    Ok(())
}

/// Get the number of panes in the current window.
pub fn pane_count() -> Result<u32> {
    Ok(pane_indices()?.len() as u32)
}

/// Get the actual pane indices in the current window.
///
/// This queries tmux directly and handles configurations where
/// `pane-base-index` is set to 1 instead of 0.
pub fn pane_indices() -> Result<Vec<u32>> {
    let output = Command::new("tmux")
        .args(["list-panes", "-F", "#{pane_index}"])
        .output()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !output.status.success() {
        return Err(PanoutError::TmuxError("list-panes failed".into()));
    }

    let indices: Vec<u32> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(indices)
}

/// Create a new tmux window, optionally with a name.
///
/// The new window becomes the active window.
pub fn create_window(name: Option<&str>) -> Result<()> {
    let mut args = vec!["new-window"];
    if let Some(n) = name {
        args.push("-n");
        args.push(n);
    }

    let status = Command::new("tmux")
        .args(&args)
        .status()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !status.success() {
        return Err(PanoutError::TmuxError("new-window failed".into()));
    }

    Ok(())
}

/// Switch to a specific window by index.
pub fn select_window(index: u32) -> Result<()> {
    let target = format!("{}", index);
    let status = Command::new("tmux")
        .args(["select-window", "-t", &target])
        .status()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !status.success() {
        return Err(PanoutError::TmuxError(format!(
            "select-window {} failed",
            index
        )));
    }

    Ok(())
}

/// Get the index of the currently active window.
pub fn current_window() -> Result<u32> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{window_index}"])
        .output()
        .map_err(|e| PanoutError::TmuxError(e.to_string()))?;

    if !output.status.success() {
        return Err(PanoutError::TmuxError("display-message failed".into()));
    }

    let index_str = String::from_utf8_lossy(&output.stdout);
    index_str
        .trim()
        .parse::<u32>()
        .map_err(|_| PanoutError::TmuxError("failed to parse window index".into()))
}
