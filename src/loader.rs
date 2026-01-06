//! Config file discovery and loading.
//!
//! Handles finding the config file across different platforms and loading it.
//! The search order is:
//!
//! 1. `$XDG_CONFIG_HOME/panout/config.toml`
//! 2. `~/.config/panout/config.toml`
//! 3. Platform default (e.g., `~/Library/Application Support` on macOS)

use crate::config::Config;
use crate::error::{PanoutError, Result};
use std::path::PathBuf;

/// Determine the config file path.
///
/// Checks locations in order of preference:
/// 1. `$XDG_CONFIG_HOME/panout/config.toml` (if XDG_CONFIG_HOME is set)
/// 2. `~/.config/panout/config.toml` (common on Linux, often used on macOS)
/// 3. Platform default via `dirs::config_dir()`
///
/// If no existing config is found, returns `~/.config/panout/config.toml`
/// as the default location for new configs.
///
/// # Errors
///
/// Returns [`PanoutError::NoConfigDir`] if the home directory cannot be determined.
pub fn default_config_path() -> Result<PathBuf> {
    // Check XDG_CONFIG_HOME first
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg).join("panout").join("config.toml");
        if path.exists() {
            return Ok(path);
        }
    }

    // Check ~/.config (common on Linux and often used on macOS)
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".config").join("panout").join("config.toml");
        if path.exists() {
            return Ok(path);
        }
    }

    // Fall back to platform default
    let config_dir = dirs::config_dir().ok_or(PanoutError::NoConfigDir)?;
    let _path = config_dir.join("panout").join("config.toml");

    // Return ~/.config path as default location for new configs
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(".config").join("panout").join("config.toml"))
    } else {
        Err(PanoutError::NoConfigDir)
    }
}

/// Load and parse a config file from the given path.
///
/// # Errors
///
/// - [`PanoutError::ConfigNotFound`] if the file doesn't exist
/// - [`PanoutError::IoError`] if reading fails
/// - [`PanoutError::ParseError`] if TOML parsing fails
pub fn load_config(path: &PathBuf) -> Result<Config> {
    if !path.exists() {
        return Err(PanoutError::ConfigNotFound(path.clone()));
    }
    let contents = std::fs::read_to_string(path)?;
    let config = Config::from_str(&contents)?;
    Ok(config)
}

/// Load config from the default path.
///
/// Convenience wrapper that combines [`default_config_path`] and [`load_config`].
pub fn load_default_config() -> Result<Config> {
    let path = default_config_path()?;
    load_config(&path)
}

/// Ensure the config directory exists, creating it if necessary.
///
/// Returns the path where the config file should be located.
pub fn ensure_config_dir() -> Result<PathBuf> {
    let path = default_config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(path)
}
