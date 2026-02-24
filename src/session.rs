//! Session name sanitization and remote SSH/tmux command builder.
//!
//! Provides utilities for creating valid tmux session names from
//! workspace and bundle names, and constructing SSH commands that
//! create or attach to remote tmux sessions.

/// Sanitize a workspace or bundle name for use as a tmux session name.
///
/// Replaces dots and colons with hyphens per SESS-03 requirement.
/// Strips characters that are not alphanumeric, hyphens, or underscores.
/// Trims leading and trailing hyphens for cleanliness.
pub fn sanitize_session_name(name: &str) -> String {
    todo!()
}

/// Build the SSH command string for creating/attaching a remote tmux session.
///
/// Uses `-t` for TTY allocation (SESS-04) and `new-session -A` for
/// idempotent attach-or-create (SESS-01, SESS-02). Calls
/// `sanitize_session_name` internally to ensure the session name is
/// tmux-compatible.
///
/// With `dir`: `ssh -t {host} "cd {dir} && tmux new-session -A -s {name}"`
/// Without `dir`: `ssh -t {host} "tmux new-session -A -s {name}"`
pub fn build_remote_session_cmd(
    host: &str,
    session_name: &str,
    dir: Option<&str>,
) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- sanitize_session_name tests ---

    #[test]
    fn sanitize_dots_to_hyphens() {
        assert_eq!(sanitize_session_name("dev.frontend"), "dev-frontend");
    }

    #[test]
    fn sanitize_colons_to_hyphens() {
        assert_eq!(sanitize_session_name("servers:prod"), "servers-prod");
    }

    #[test]
    fn sanitize_mixed_special_chars() {
        assert_eq!(
            sanitize_session_name("my.workspace:test!"),
            "my-workspace-test"
        );
    }

    #[test]
    fn sanitize_alphanumeric_unchanged() {
        assert_eq!(sanitize_session_name("simple"), "simple");
    }

    #[test]
    fn sanitize_underscores_and_hyphens_preserved() {
        assert_eq!(sanitize_session_name("a_b-c"), "a_b-c");
    }

    #[test]
    fn sanitize_leading_hyphens_trimmed() {
        assert_eq!(sanitize_session_name("--leading"), "leading");
    }

    #[test]
    fn sanitize_empty_string() {
        assert_eq!(sanitize_session_name(""), "");
    }

    #[test]
    fn sanitize_consecutive_dots() {
        assert_eq!(sanitize_session_name("a..b"), "a--b");
    }

    // --- build_remote_session_cmd tests ---

    #[test]
    fn build_cmd_without_dir() {
        assert_eq!(
            build_remote_session_cmd("host", "myws", None),
            "ssh -t host \"tmux new-session -A -s myws\""
        );
    }

    #[test]
    fn build_cmd_with_dir_and_sanitization() {
        assert_eq!(
            build_remote_session_cmd("user@server", "dev.app", Some("~/src")),
            "ssh -t user@server \"cd ~/src && tmux new-session -A -s dev-app\""
        );
    }

    #[test]
    fn build_cmd_always_includes_tty_flag() {
        let cmd = build_remote_session_cmd("host", "test", None);
        assert!(
            cmd.contains("ssh -t"),
            "SSH command must include -t for TTY allocation"
        );
    }

    #[test]
    fn build_cmd_always_uses_attach_or_create() {
        let cmd = build_remote_session_cmd("host", "test", None);
        assert!(
            cmd.contains("new-session -A -s"),
            "Must use new-session -A -s for atomic attach-or-create"
        );
    }
}
