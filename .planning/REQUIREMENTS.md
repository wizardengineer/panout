# Requirements: Panout

**Defined:** 2026-02-24
**Core Value:** Panout must reliably create the exact tmux layout and commands the user configured -- bundles, workspaces, SSH connections, and pane arrangements all working from a single `panout` command.

## v1 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Remote Sessions

- [ ] **SESS-01**: User can create a named tmux session on a remote host when running a bundle with SSH (session name = sanitized bundle name)
- [ ] **SESS-02**: If remote tmux session already exists, user is reattached to it instead of creating a duplicate
- [x] **SESS-03**: Bundle names are sanitized for tmux compatibility (dots replaced with hyphens, special chars stripped)
- [x] **SESS-04**: SSH commands use TTY allocation (`-t` flag) for tmux to function
- [ ] **SESS-05**: After SSH disconnect, user can re-run the same panout command to reconnect to the persisted session with all windows/panes intact
- [ ] **SESS-06**: Existing bundle and workspace configs continue working without changes
- [ ] **SESS-07**: User can configure mosh as an alternative to SSH for remote connections (e.g., `protocol = "mosh"` on workspace, defaults to "ssh")

### Release Infrastructure

- [x] **RLSE-01**: GitHub Actions workflow builds cross-platform release binaries on tagged commits
- [x] **RLSE-02**: cargo-dist produces release assets for Linux x86_64, macOS x86_64, macOS aarch64
- [x] **RLSE-03**: Release archives follow naming convention compatible with self_update crate

### Self-Update

- [ ] **UPDT-01**: User can run `panout --update` to download and install the latest release binary
- [ ] **UPDT-02**: Update command compares current version against latest release and skips download if already up to date
- [ ] **UPDT-03**: Download shows progress indicator
- [ ] **UPDT-04**: Update reports version transition ("Updated from vX to vY") or "Already up to date"

### Sesh Session Switcher

- [ ] **SESH-01**: sesh script lists all tmux sessions and lets user fuzzy-search with fzf to switch
- [ ] **SESH-02**: sesh is installable via curl one-liner that places script at `~/.local/bin/sesh`
- [ ] **SESH-03**: If fzf is not installed, sesh displays a warning and falls back to numbered selection
- [ ] **SESH-04**: sesh handles edge cases: no sessions, user cancels fzf (no switch), empty selection

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Session Management

- **SESS-07**: Session preview in sesh (show pane content in fzf preview window)
- **SESS-08**: Kill sessions from sesh picker (ctrl-x binding)
- **SESS-09**: Create new sessions from sesh by typing a non-matching name

### Self-Update

- **UPDT-05**: Display changelog link after successful update
- **UPDT-06**: Version pinning (install specific version instead of latest)

### Documentation

- **DOCS-01**: Document nested tmux prefix key configuration (local vs remote)
- **DOCS-02**: Document SSH multiplexing recommendations for workspaces with many panes

## Out of Scope

| Feature | Reason |
|---------|--------|
| Auto-provisioning sesh on remote machines | SSH write access risk, couples sesh to panout lifecycle |
| Auto-update checks on every panout invocation | Slows every command with network call, annoying UX |
| Sesh as compiled Rust binary | Over-engineering for ~30 lines of shell |
| tmux plugin (TPM) distribution | Adds dependency, not available on minimal servers |
| Remote process monitoring/restart | Panout creates sessions, not a process manager |
| Windows support | tmux does not run natively on Windows |
| YAML/JSON config support | TOML is panout's deliberate choice |
| SSH connection multiplexing management | Couples panout to SSH config; document recommendations instead |
| macOS code signing for releases | Address if users report Gatekeeper issues, not preemptively |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SESS-01 | Phase 2 | Pending |
| SESS-02 | Phase 2 | Pending |
| SESS-03 | Phase 2 | Complete |
| SESS-04 | Phase 2 | Complete |
| SESS-05 | Phase 2 | Pending |
| SESS-06 | Phase 2 | Pending |
| SESS-07 | Phase 2 | Pending |
| RLSE-01 | Phase 1 | Complete |
| RLSE-02 | Phase 1 | Complete |
| RLSE-03 | Phase 1 | Complete |
| UPDT-01 | Phase 3 | Pending |
| UPDT-02 | Phase 3 | Pending |
| UPDT-03 | Phase 3 | Pending |
| UPDT-04 | Phase 3 | Pending |
| SESH-01 | Phase 4 | Pending |
| SESH-02 | Phase 4 | Pending |
| SESH-03 | Phase 4 | Pending |
| SESH-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18
- Unmapped: 0

---
*Requirements defined: 2026-02-24*
*Last updated: 2026-02-24 after roadmap creation*
