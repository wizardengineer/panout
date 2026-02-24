# Research Summary: Panout Milestone -- Remote Sessions, Self-Update, Sesh

**Domain:** Tmux session orchestration (remote SSH sessions, CLI self-update, session switcher)
**Researched:** 2026-02-24
**Overall confidence:** HIGH

## Executive Summary

This milestone adds three features to panout: remote tmux session management, self-update from GitHub Releases, and a standalone session switcher script. The research shows that all three are feasible with minimal new dependencies.

The remote tmux session feature requires no new Rust crates. The existing `std::process::Command`-based SSH approach extends naturally -- panout sends `ssh -t user@host "tmux new-session -A -s bundle_name"` instead of just `ssh user@host`. The `-A` flag on `tmux new-session` handles the attach-or-create logic atomically, available since tmux 1.8 (2013). This is the canonical pattern documented across tmux resources and blog posts.

The self-update feature has one clear winner: the `self_update` crate (v0.42) with the `ureq` backend. This combination avoids the tokio async runtime that the default reqwest backend would pull in, keeping panout's dependency tree lean. The crate handles version comparison, platform-specific asset matching, download progress, archive extraction, and binary replacement. It is battle-tested in production tools like `mise`. The companion requirement is a release pipeline (cargo-dist) to produce cross-platform `.tar.gz` assets on GitHub Releases.

The sesh session switcher is intentionally a shell script, not a Rust binary. It pipes `tmux list-sessions` through `fzf` and calls `tmux switch-client`. This approach keeps it installable via a single curl command on any remote machine, with zero compilation needed. The pattern is well-established across dozens of blog posts and tools.

## Key Findings

**Stack:** One new Rust dependency (`self_update` 0.42 with ureq backend), one CI tool (cargo-dist), one shell script. No async runtime, no SSH crates, no tmux plugins.

**Architecture:** The remote session feature changes panout's fundamental assumption -- it currently requires running inside tmux (`NotInTmux` guard). Creating remote sessions means panout may or may not be inside a local tmux session. The `in_tmux()` check needs to become context-dependent.

**Critical pitfall:** The self-update feature requires a release pipeline (GitHub Actions + cargo-dist) producing correctly-named `.tar.gz` assets before the `--update` flag can work. The release infrastructure must be built first, or the feature has nothing to download.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Release Infrastructure** - Set up cargo-dist, GitHub Actions, cross-platform builds
   - Addresses: `--update` prerequisite (release assets must exist before self-update can work)
   - Avoids: Building --update code with no releases to test against

2. **Remote Tmux Sessions** - Modify SSH handling to create/attach named sessions
   - Addresses: Core feature (bundle sessions on remote hosts)
   - Avoids: Over-engineering with SSH crates; keeps the subprocess approach

3. **Self-Update** - Add `--update` flag using self_update crate
   - Addresses: Binary distribution for non-Rust users
   - Avoids: Shipping with default reqwest (use ureq instead)

4. **Sesh Session Switcher** - Write shell script, host in repo, document curl install
   - Addresses: Session navigation on remote machines
   - Avoids: Over-engineering with a compiled binary

**Phase ordering rationale:**
- Release infrastructure must come before self-update (self-update downloads release assets that must exist)
- Remote tmux sessions should come before sesh (sesh is useful for navigating sessions that remote tmux creates)
- Self-update can be developed in parallel with remote sessions once releases are flowing

**Research flags for phases:**
- Phase 1 (Release Infrastructure): Needs deeper research on cargo-dist asset naming compatibility with self_update
- Phase 2 (Remote Sessions): Standard patterns, unlikely to need further research
- Phase 3 (Self-Update): Standard patterns once releases exist
- Phase 4 (Sesh): Standard shell scripting, no research needed

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | `self_update` verified on crates.io, docs.rs, and GitHub. ureq backend confirmed in Cargo.toml. tmux commands verified in official wiki. |
| Features | HIGH | All three features use well-established patterns with clear implementation paths. |
| Architecture | HIGH | Extensions to existing subprocess-based architecture. No fundamental redesign. |
| Pitfalls | MEDIUM | The cargo-dist + self_update asset naming interop is the one area with moderate uncertainty. Needs validation during implementation. |

## Gaps to Address

- **cargo-dist asset naming vs self_update expectations**: Both use `{name}-{version}-{target}.tar.gz` but exact format compatibility should be tested during Phase 1 (release infrastructure). The `self_update` crate's `bin_path_in_archive` config may be needed if cargo-dist nests the binary in a subdirectory within the archive.
- **macOS code signing**: If distributing macOS binaries via GitHub Releases, unsigned binaries trigger Gatekeeper warnings. This is a distribution concern, not a code concern, but worth noting for Phase 1.
- **Rate limiting**: GitHub API has rate limits (60 requests/hour unauthenticated). The `--update` command should handle 403/rate-limit responses gracefully with a clear error message.
