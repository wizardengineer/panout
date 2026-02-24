---
phase: 02-remote-tmux-sessions
plan: 01
subsystem: ssh
tags: [tmux, ssh, session, sanitization]

# Dependency graph
requires:
  - phase: 01-release-infrastructure
    provides: "cargo-dist release binaries"
provides:
  - "sanitize_session_name function for tmux-safe names"
  - "build_remote_session_cmd function for SSH/tmux command strings"
  - "session module exported from lib.rs"
affects: [02-02 workspace SSH integration, remote session routing]

# Tech tracking
tech-stack:
  added: []
  patterns: ["char-map sanitization for tmux session names", "SSH -t with tmux new-session -A -s for atomic attach-or-create"]

key-files:
  created: [src/session.rs]
  modified: [src/lib.rs]

key-decisions:
  - "Strip non-alphanumeric chars (except hyphens/underscores) rather than replacing all with hyphens -- avoids noisy names"
  - "Sentinel char + filter approach for char removal in sanitizer -- clear and idiomatic Rust"

patterns-established:
  - "Session name sanitization: dots and colons to hyphens, strip other special chars, trim leading/trailing hyphens"
  - "Remote command format: ssh -t {host} \"tmux new-session -A -s {name}\" with optional cd prefix"

requirements-completed: [SESS-03, SESS-04]

# Metrics
duration: 2min
completed: 2026-02-24
---

# Phase 2 Plan 1: Session Name Sanitization and Remote Command Builder Summary

**Pure-function session module with tmux name sanitization (dots/colons to hyphens) and SSH command builder using -t TTY and new-session -A atomic attach-or-create**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-24T19:47:33Z
- **Completed:** 2026-02-24T19:49:12Z
- **Tasks:** 1 (TDD: RED + GREEN)
- **Files modified:** 2

## Accomplishments
- Created `sanitize_session_name` handling 8 edge cases (dots, colons, mixed special chars, empty string, leading hyphens, consecutive dots)
- Created `build_remote_session_cmd` constructing SSH commands with TTY allocation and optional directory cd prefix
- All 12 tests passing, zero warnings from cargo build

## Task Commits

Each task was committed atomically:

1. **Task 1 (RED): Failing tests for session module** - `16c82c8` (test)
2. **Task 1 (GREEN): Implement sanitization and command builder** - `f04d30e` (feat)

_TDD task: no refactor commit needed -- code was clean after GREEN phase._

## Files Created/Modified
- `src/session.rs` - Session name sanitization and SSH/tmux command builder with 12 inline tests
- `src/lib.rs` - Added `pub mod session` export

## Decisions Made
- Stripped non-alphanumeric chars entirely (rather than replacing all with hyphens) to avoid noisy double-hyphen sequences from arbitrary special characters
- Used sentinel char (`\0`) plus filter for char removal -- simpler than `flat_map` with `Option` for this use case
- Skipped refactor phase: code was already clean, well-documented, and clippy-clean after GREEN

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

Pre-existing clippy warning in `src/config.rs` (`should_implement_trait` for `from_str` method) -- out of scope, not caused by this plan's changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- `sanitize_session_name` and `build_remote_session_cmd` are ready for Plan 02 integration
- Plan 02 will import these functions to route workspace SSH through remote tmux sessions

---
*Phase: 02-remote-tmux-sessions*
*Completed: 2026-02-24*
