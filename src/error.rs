//! Error types for panout.
//!
//! All errors in panout are represented by [`PanoutError`], which covers
//! configuration issues, tmux failures, and reference resolution problems.

use std::path::PathBuf;
use thiserror::Error;

/// All possible errors that can occur in panout.
#[derive(Error, Debug)]
pub enum PanoutError {
    /// Config file does not exist at the expected path.
    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    /// Could not determine the user's config directory.
    #[error("Could not determine config directory")]
    NoConfigDir,

    /// Failed to read a file from disk.
    #[error("Failed to read config: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML parsing failed.
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Requested bundle does not exist in config.
    #[error("Bundle not found: {0}")]
    BundleNotFound(String),

    /// Requested server does not exist in config.
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    /// Requested workspace does not exist in config.
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    /// Bundle reference (`@ref`) has invalid syntax.
    #[error("Invalid bundle reference: {0}")]
    InvalidRef(String),

    /// Bundle references form a cycle (A -> B -> A).
    #[error("Circular reference detected: {0}")]
    CircularRef(String),

    /// A tmux command failed to execute.
    #[error("Tmux error: {0}")]
    TmuxError(String),

    /// Command was run outside of a tmux session.
    #[error("Not running inside tmux")]
    NotInTmux,
}

/// Convenient Result type alias for panout operations.
pub type Result<T> = std::result::Result<T, PanoutError>;
