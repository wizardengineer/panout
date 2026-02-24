---
phase: 01-release-infrastructure
plan: 01
subsystem: infra
tags: [cargo-dist, github-actions, ci-cd, release, cross-platform]

# Dependency graph
requires: []
provides:
  - GitHub Actions release workflow triggered by version tags
  - cargo-dist configuration for 3 platform targets with .tar.gz archives
  - Shell installer generation (curl|sh one-liner)
  - Release infrastructure ready for Phase 3 self-update integration
affects: [03-self-update]

# Tech tracking
tech-stack:
  added: [cargo-dist 0.31.0]
  patterns: [tag-triggered-release, dist-workspace-config]

key-files:
  created:
    - dist-workspace.toml
    - .github/workflows/release.yml
  modified:
    - Cargo.toml

key-decisions:
  - "Used dist-workspace.toml (standalone config) over [workspace.metadata.dist] in Cargo.toml -- cargo-dist v0.31.0 defaults to this format"
  - "Set unix-archive = .tar.gz explicitly for self_update crate compatibility (default is .tar.xz)"
  - "Included shell installer for curl|sh install one-liner"
  - "Removed Windows and ARM64 Linux targets from defaults -- only the 3 required targets (x86_64-linux, x86_64-macos, aarch64-macos)"

patterns-established:
  - "cargo-dist config: all customization goes through dist-workspace.toml, never hand-edit release.yml"
  - "Release flow: update Cargo.toml version -> commit -> tag vX.Y.Z -> push tag -> GitHub Actions builds and publishes"

requirements-completed: [RLSE-01, RLSE-02, RLSE-03]

# Metrics
duration: 3min
completed: 2026-02-24
---

# Phase 1 Plan 1: Configure cargo-dist Summary

**cargo-dist v0.31.0 configured with GitHub Actions workflow producing .tar.gz archives for Linux x86_64, macOS x86_64, and macOS aarch64 on version tag push**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-24T18:59:58Z
- **Completed:** 2026-02-24T19:02:57Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Configured cargo-dist v0.31.0 with dist-workspace.toml for 3 cross-platform targets
- Generated GitHub Actions release workflow that triggers on version tags (e.g., v1.0.0)
- Set .tar.gz archive format for self_update crate compatibility in Phase 3
- Verified local build produces valid archive with panout binary inside
- All 5 unit tests and 3 doc tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Configure Cargo.toml and initialize cargo-dist** - `1aa0df1` (feat)
2. **Task 2: Verify release plan and local build** - verification only, no file changes

## Files Created/Modified
- `Cargo.toml` - Bumped version to 1.0.0, added repository field, added [profile.dist] section
- `dist-workspace.toml` - cargo-dist configuration: targets, archive format, installers, CI backend
- `.github/workflows/release.yml` - Generated GitHub Actions workflow for tag-triggered releases
- `Cargo.lock` - Updated lockfile reflecting version bump

## Decisions Made
- Used `dist-workspace.toml` standalone config format (cargo-dist v0.31.0 default) rather than `[workspace.metadata.dist]` in Cargo.toml
- Set `unix-archive = ".tar.gz"` explicitly since cargo-dist defaults to `.tar.xz` which is incompatible with self_update crate's archive-tar + compression-flate2 features
- Included `shell` installer for convenient curl|sh installation
- Removed Windows (`x86_64-pc-windows-msvc`) and ARM64 Linux (`aarch64-unknown-linux-gnu`) targets from defaults -- only the 3 required Unix targets are configured
- cargo-dist installs as `dist` binary (not `cargo dist` subcommand) in v0.31.0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adjusted cargo-dist default targets and config**
- **Found during:** Task 1
- **Issue:** `dist init --yes` generated config with 5 targets (including Windows and ARM64 Linux) and empty installers, missing unix-archive setting
- **Fix:** Edited dist-workspace.toml to set exactly 3 required targets, add `unix-archive = ".tar.gz"`, and add `shell` installer. Ran `dist generate` to regenerate workflow.
- **Files modified:** dist-workspace.toml
- **Verification:** `dist plan` confirms 3 targets with .tar.gz format
- **Committed in:** 1aa0df1

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Expected deviation -- plan anticipated that `dist init` defaults would need adjustment. No scope creep.

## Issues Encountered
- cargo-dist v0.31.0 installs the binary as `dist` (not as a cargo subcommand `cargo dist`). The `cargo dist` invocation does not work; must use `dist` directly. This is a naming change in recent versions. CI workflow uses `dist` correctly.

## User Setup Required

None - no external service configuration required. Release infrastructure is ready; pushing a version tag (e.g., `git tag v1.0.0 && git push --tags`) will trigger the workflow.

## Next Phase Readiness
- Release infrastructure is complete and ready for Phase 3 (self-update) integration
- Archive naming (`panout-{target}.tar.gz`) is compatible with self_update crate's GitHub backend
- No release has been triggered yet -- first release (v1.0.0 tag) should be pushed when the user is ready
- Phase 2 (Remote Tmux Sessions) can proceed independently

## Self-Check: PASSED

- FOUND: Cargo.toml
- FOUND: dist-workspace.toml
- FOUND: .github/workflows/release.yml
- FOUND: 01-01-SUMMARY.md
- FOUND: commit 1aa0df1

---
*Phase: 01-release-infrastructure*
*Completed: 2026-02-24*
