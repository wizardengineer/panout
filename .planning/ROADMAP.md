# Roadmap: Panout

## Overview

This milestone adds three capabilities to panout: persistent tmux sessions on remote machines via SSH bundles, binary self-update from GitHub Releases, and a standalone session switcher script (sesh). Release infrastructure comes first since self-update depends on published release assets. Remote sessions and sesh form a natural sequence -- sesh helps navigate the sessions that remote bundles create.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Release Infrastructure** - GitHub Actions + cargo-dist pipeline producing cross-platform release binaries
- [ ] **Phase 2: Remote Tmux Sessions** - SSH bundles create/reattach named tmux sessions on remote hosts
- [ ] **Phase 3: Self-Update** - `panout --update` downloads and replaces the binary from GitHub Releases
- [ ] **Phase 4: Sesh Session Switcher** - Shell script for fuzzy-searching and switching tmux sessions on remote machines

## Phase Details

### Phase 1: Release Infrastructure
**Goal**: Tagged commits produce downloadable release binaries for all supported platforms
**Depends on**: Nothing (first phase)
**Requirements**: RLSE-01, RLSE-02, RLSE-03
**Success Criteria** (what must be TRUE):
  1. Pushing a version tag to GitHub triggers an automated build that produces release assets
  2. Release artifacts exist for Linux x86_64, macOS x86_64, and macOS aarch64
  3. Release archive naming follows the `{name}-{version}-{target}.tar.gz` convention expected by the self_update crate
**Plans:** 1 plan

Plans:
- [x] 01-01-PLAN.md -- Configure cargo-dist for cross-platform release binaries (RLSE-01, RLSE-02, RLSE-03)

### Phase 2: Remote Tmux Sessions
**Goal**: Users can run a bundle with SSH and get a persistent, named tmux session on the remote host that survives disconnects
**Depends on**: Nothing (independent of Phase 1)
**Requirements**: SESS-01, SESS-02, SESS-03, SESS-04, SESS-05, SESS-06
**Success Criteria** (what must be TRUE):
  1. Running a bundle with SSH targets creates a named tmux session on each remote host (session name derived from bundle name)
  2. Re-running the same bundle command reattaches to the existing remote session instead of creating a duplicate
  3. After an SSH disconnect, re-running the bundle reconnects to the persisted session with all windows and panes intact
  4. Existing bundle and workspace configs that do not use remote sessions continue working without any changes
**Plans:** 2 plans

Plans:
- [ ] 02-01-PLAN.md -- Session name sanitization and remote command builder (SESS-03, SESS-04)
- [ ] 02-02-PLAN.md -- Wire remote sessions into workspace execution (SESS-01, SESS-02, SESS-05, SESS-06)

### Phase 3: Self-Update
**Goal**: Users can update panout to the latest version with a single command
**Depends on**: Phase 1 (release assets must exist on GitHub Releases)
**Requirements**: UPDT-01, UPDT-02, UPDT-03, UPDT-04
**Success Criteria** (what must be TRUE):
  1. Running `panout --update` downloads and installs the latest release binary, replacing the current one
  2. If the current version is already the latest, the command reports "Already up to date" without downloading
  3. The download displays a progress indicator and reports the version transition on completion
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: Sesh Session Switcher
**Goal**: Users on remote machines can fuzzy-search and switch between tmux sessions using a lightweight script
**Depends on**: Nothing (standalone script, but benefits from Phase 2 creating sessions to switch between)
**Requirements**: SESH-01, SESH-02, SESH-03, SESH-04
**Success Criteria** (what must be TRUE):
  1. Running `sesh` lists all tmux sessions and lets the user fuzzy-select one via fzf to switch to it
  2. Running `curl <one-liner>` installs sesh to `~/.local/bin/sesh` on any Linux/macOS machine
  3. If fzf is not installed, sesh falls back to numbered selection with a warning instead of failing
  4. Edge cases are handled gracefully: no sessions shows a message, cancelling fzf does nothing, empty selection does nothing
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Release Infrastructure | 1/1 | Complete | 2026-02-24 |
| 2. Remote Tmux Sessions | 0/2 | Planned | - |
| 3. Self-Update | 0/0 | Not started | - |
| 4. Sesh Session Switcher | 0/0 | Not started | - |
