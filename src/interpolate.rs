//! Variable interpolation for commands.
//!
//! Provides `{user}` and `{ip}` placeholder expansion for commands
//! that need to reference parts of an SSH host string.
//!
//! # Example
//!
//! ```
//! use panout::interpolate::{parse_host, interpolate};
//!
//! let host = "admin@192.168.1.100";
//! let (user, ip) = parse_host(host).unwrap();
//!
//! let cmd = interpolate("cd /home/{user}/src", &user, &ip);
//! assert_eq!(cmd, "cd /home/admin/src");
//! ```

/// Parse a host string into (user, ip) components.
///
/// Expects format `user@ip` and returns `None` if the `@` is missing.
///
/// # Examples
///
/// ```
/// use panout::interpolate::parse_host;
///
/// assert_eq!(
///     parse_host("admin@192.168.1.1"),
///     Some(("admin".to_string(), "192.168.1.1".to_string()))
/// );
/// assert_eq!(parse_host("no-at-sign"), None);
/// ```
pub fn parse_host(host: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = host.splitn(2, '@').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Replace `{user}` and `{ip}` placeholders in a command string.
///
/// # Examples
///
/// ```
/// use panout::interpolate::interpolate;
///
/// let result = interpolate("ssh {user}@{ip}", "root", "10.0.0.1");
/// assert_eq!(result, "ssh root@10.0.0.1");
/// ```
pub fn interpolate(command: &str, user: &str, ip: &str) -> String {
    command.replace("{user}", user).replace("{ip}", ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host() {
        assert_eq!(
            parse_host("admin@192.168.1.1"),
            Some(("admin".to_string(), "192.168.1.1".to_string()))
        );
        assert_eq!(parse_host("no-at-sign"), None);
    }

    #[test]
    fn test_interpolate() {
        assert_eq!(
            interpolate("cd /home/{user}/src", "admin", "192.168.1.1"),
            "cd /home/admin/src"
        );
        assert_eq!(
            interpolate("ssh {user}@{ip}", "root", "10.0.0.1"),
            "ssh root@10.0.0.1"
        );
    }
}
