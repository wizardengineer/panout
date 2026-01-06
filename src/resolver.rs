//! Bundle reference resolution.
//!
//! Handles the `@ref` syntax that allows bundles to reference other bundles.
//! References are expanded recursively with cycle detection.
//!
//! # Reference Syntax
//!
//! - `@group.name` - Reference a specific bundle
//! - `@group.*` - Reference all bundles in a group
//!
//! # Example
//!
//! ```toml
//! [dev.frontend]
//! cmd = "npm run dev"
//!
//! [dev.backend]
//! cmd = "cargo run"
//!
//! [dev.all]
//! cmd = ["@dev.frontend", "@dev.backend"]  # Expands to both bundles
//! ```

use crate::config::Config;
use crate::error::{PanoutError, Result};
use std::collections::HashSet;

/// A parsed reference from a command string.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedRef {
    /// A plain command (not a reference).
    Command(String),
    /// Reference to a specific bundle: `@group.name`
    BundleRef {
        /// The bundle group name.
        group: String,
        /// The bundle entry name within the group.
        name: String,
    },
    /// Reference to all bundles in a group: `@group.*`
    GroupAll {
        /// The bundle group name.
        group: String,
    },
}

/// Parse a string into a [`ResolvedRef`].
///
/// Strings starting with `@` are treated as references:
/// - `@group.name` -> `BundleRef`
/// - `@group.*` -> `GroupAll`
/// - Everything else -> `Command`
pub fn parse_ref(s: &str) -> ResolvedRef {
    if let Some(rest) = s.strip_prefix('@') {
        let parts: Vec<&str> = rest.splitn(2, '.').collect();
        if parts.len() == 2 {
            let group = parts[0].to_string();
            let name = parts[1].to_string();
            if name == "*" {
                ResolvedRef::GroupAll { group }
            } else {
                ResolvedRef::BundleRef { group, name }
            }
        } else {
            ResolvedRef::Command(s.to_string())
        }
    } else {
        ResolvedRef::Command(s.to_string())
    }
}

/// Resolve all commands for a bundle, recursively expanding `@ref`s.
///
/// Returns a flat list of commands in execution order.
///
/// # Errors
///
/// - [`PanoutError::BundleNotFound`] if a referenced bundle doesn't exist
/// - [`PanoutError::CircularRef`] if references form a cycle
pub fn resolve_bundle(config: &Config, bundle_path: &str) -> Result<Vec<String>> {
    let mut visited = HashSet::new();
    resolve_bundle_inner(config, bundle_path, &mut visited)
}

fn resolve_bundle_inner(
    config: &Config,
    bundle_path: &str,
    visited: &mut HashSet<String>,
) -> Result<Vec<String>> {
    if visited.contains(bundle_path) {
        return Err(PanoutError::CircularRef(bundle_path.to_string()));
    }
    visited.insert(bundle_path.to_string());

    let bundle = config
        .get_bundle(bundle_path)
        .ok_or_else(|| PanoutError::BundleNotFound(bundle_path.to_string()))?;

    let mut result = Vec::new();

    for cmd_str in bundle.cmd.to_vec() {
        match parse_ref(&cmd_str) {
            ResolvedRef::Command(cmd) => {
                result.push(cmd);
            }
            ResolvedRef::BundleRef { group, name } => {
                let ref_path = format!("{}.{}", group, name);
                let sub_cmds = resolve_bundle_inner(config, &ref_path, visited)?;
                result.extend(sub_cmds);
            }
            ResolvedRef::GroupAll { group } => {
                let group_entries = config.get_group(&group).ok_or_else(|| {
                    PanoutError::BundleNotFound(format!("group '{}'", group))
                })?;
                let mut names: Vec<_> = group_entries.keys().collect();
                names.sort();
                for name in names {
                    let ref_path = format!("{}.{}", group, name);
                    let sub_cmds = resolve_bundle_inner(config, &ref_path, visited)?;
                    result.extend(sub_cmds);
                }
            }
        }
    }

    visited.remove(bundle_path);
    Ok(result)
}

/// Resolve commands grouped by target pane.
///
/// Similar to [`resolve_bundle`] but preserves pane assignments from bundle configs.
/// Returns tuples of `(pane_index, commands)`.
pub fn resolve_with_panes(config: &Config, bundle_path: &str) -> Result<Vec<(u32, Vec<String>)>> {
    let mut visited = HashSet::new();
    let mut pane_cmds: Vec<(u32, Vec<String>)> = Vec::new();

    resolve_with_panes_inner(config, bundle_path, &mut visited, &mut pane_cmds, 0)?;

    Ok(pane_cmds)
}

fn resolve_with_panes_inner(
    config: &Config,
    bundle_path: &str,
    visited: &mut HashSet<String>,
    pane_cmds: &mut Vec<(u32, Vec<String>)>,
    default_pane: u32,
) -> Result<()> {
    if visited.contains(bundle_path) {
        return Err(PanoutError::CircularRef(bundle_path.to_string()));
    }
    visited.insert(bundle_path.to_string());

    let bundle = config
        .get_bundle(bundle_path)
        .ok_or_else(|| PanoutError::BundleNotFound(bundle_path.to_string()))?;

    let target_pane = bundle.pane.unwrap_or(default_pane);
    let mut direct_cmds = Vec::new();

    for cmd_str in bundle.cmd.to_vec() {
        match parse_ref(&cmd_str) {
            ResolvedRef::Command(cmd) => {
                direct_cmds.push(cmd);
            }
            ResolvedRef::BundleRef { group, name } => {
                let ref_path = format!("{}.{}", group, name);
                resolve_with_panes_inner(config, &ref_path, visited, pane_cmds, target_pane)?;
            }
            ResolvedRef::GroupAll { group } => {
                let group_entries = config.get_group(&group).ok_or_else(|| {
                    PanoutError::BundleNotFound(format!("group '{}'", group))
                })?;
                let mut names: Vec<_> = group_entries.keys().collect();
                names.sort();
                for name in names {
                    let ref_path = format!("{}.{}", group, name);
                    resolve_with_panes_inner(config, &ref_path, visited, pane_cmds, target_pane)?;
                }
            }
        }
    }

    if !direct_cmds.is_empty() {
        if let Some(entry) = pane_cmds.iter_mut().find(|(p, _)| *p == target_pane) {
            entry.1.extend(direct_cmds);
        } else {
            pane_cmds.push((target_pane, direct_cmds));
        }
    }

    visited.remove(bundle_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ref_command() {
        assert_eq!(
            parse_ref("echo hello"),
            ResolvedRef::Command("echo hello".to_string())
        );
    }

    #[test]
    fn test_parse_ref_bundle() {
        assert_eq!(
            parse_ref("@dev.frontend"),
            ResolvedRef::BundleRef {
                group: "dev".to_string(),
                name: "frontend".to_string()
            }
        );
    }

    #[test]
    fn test_parse_ref_group_all() {
        assert_eq!(
            parse_ref("@dev.*"),
            ResolvedRef::GroupAll {
                group: "dev".to_string()
            }
        );
    }
}
