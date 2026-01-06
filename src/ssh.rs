//! SSH session management via tmux.
//!
//! This module provides helpers for managing SSH connections within tmux panes.
//! SSH is handled by sending `ssh user@host` commands to panes via [`crate::tmux::send_keys`].

use crate::error::Result;
use crate::tmux;

/// Start an SSH session in the specified pane.
///
/// Sends `ssh <host>` to the pane. The host should be in `user@ip` format.
///
/// # Example
///
/// ```ignore
/// ssh::connect(0, "admin@192.168.1.100")?;
/// ```
pub fn connect(pane: u32, host: &str) -> Result<()> {
    let ssh_cmd = format!("ssh {}", host);
    tmux::send_keys(pane, &ssh_cmd)
}

/// Disconnect from SSH in the specified pane.
///
/// Sends `exit` to the pane to close the SSH session.
pub fn disconnect(pane: u32) -> Result<()> {
    tmux::send_keys(pane, "exit")
}
