# Domain Pitfalls

**Domain:** Tmux session orchestration -- remote SSH sessions, CLI self-update, session switcher
**Researched:** 2026-02-24

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Building --update Before Release Infrastructure Exists
**What goes wrong:** You implement `panout --update` using `self_update`, but there are no GitHub Releases with properly-named `.tar.gz` assets. The update command either 404s or downloads the wrong thing.
**Why it happens:** The Rust code is easier to write than the CI pipeline. Natural instinct is to write code first. But self-update is useless without release assets.
**Consequences:** Feature is untestable. You ship broken `--update` that confuses users.
**Prevention:** Set up cargo-dist + GitHub Actions first. Create at least one real release before writing update code. Test against real assets.
**Detection:** If you find yourself mocking HTTP responses to test --update, you are doing this backwards.

### Pitfall 2: SSH Without `-t` Flag for Remote tmux
**What goes wrong:** Panout sends `ssh user@host "tmux new-session -A -s name"` without `-t`. tmux on the remote host fails with "open terminal failed: not a terminal".
**Why it happens:** SSH allocates a TTY for interactive sessions but NOT when a command is specified as argument. Developers test interactively and it works; they script it and it breaks.
**Consequences:** Remote session creation silently fails or errors out.
**Prevention:** Always use `ssh -t` when the remote command requires a terminal. tmux always does. This is the single most important implementation detail.
**Detection:** Test manually: `ssh -t host "tmux new-session -A -s test"` before writing any Rust code.

### Pitfall 3: The NotInTmux Guard Blocks Self-Update
**What goes wrong:** The current code checks `in_tmux()` early and errors if not in tmux. Users running `panout --update` from a regular terminal get `NotInTmux` error instead of updating.
**Why it happens:** The tmux check was written when panout only operated within tmux. Self-update has nothing to do with tmux.
**Consequences:** Users cannot update panout unless they are inside tmux. Defeats the purpose.
**Prevention:** Check `--update` flag before the tmux check. Self-update should be an early exit in `main()` that bypasses all config loading and tmux validation.
**Detection:** Try running `panout --update` outside of tmux during development.

### Pitfall 4: Session Name Collision with Tmux Target Syntax
**What goes wrong:** If a bundle name contains a colon (e.g., `servers:production`), tmux interprets it as a target specifier (`session:window.pane`). Session names are silently mangled.
**Why it happens:** Colons are special in tmux's target syntax. Dots may also cause issues with some tmux commands.
**Consequences:** Session names mangled. Commands targeting the session fail unpredictably.
**Prevention:** Sanitize bundle names for session use: replace colons and problematic chars. Document that bundle names used for sessions should avoid colons. Dots from the `@group.name` convention should be replaced with hyphens.
**Detection:** Unit test with bundle names containing colons, dots, spaces, and special characters.

### Pitfall 5: send-keys Race Condition with SSH Panes
**What goes wrong:** Panout sends SSH commands to panes via `tmux send-keys` after creating them with `split-window`. When the new pane's shell has not finished initializing, the keystrokes arrive before the shell is ready. The command appears in the pane but never executes.
**Why it happens:** `tmux split-window` creates a pane and spawns a shell, but returns immediately before shell finishes sourcing `.zshrc`/`.bashrc`. Zsh with oh-my-zsh can take 200-500ms to initialize; `send-keys` arrives in under 1ms.
**Consequences:** Remote session creation silently fails. User sees dead pane with command text printed but not executed.
**Prevention:** Use `tmux split-window "ssh -t host \"tmux new-session -A -s name\""` -- pass the command as an argument to `split-window` so it IS the pane command (no shell init race). For panes that need multiple commands, chain with `&&`.
**Detection:** Commands appearing as text in panes but not executing. Intermittent failures that "work sometimes."

## Moderate Pitfalls

### Pitfall 6: self_update Default Features Pull in Tokio
**What goes wrong:** Adding `self_update = "0.42"` to Cargo.toml without `default-features = false` pulls in reqwest + tokio. Compile time jumps from ~30s to ~90s. Binary grows ~2MB.
**Prevention:** Always specify: `self_update = { version = "0.42", default-features = false, features = ["ureq", "rustls", "archive-tar", "compression-flate2"] }`

### Pitfall 7: cargo-dist Asset Names Don't Match self_update Expectations
**What goes wrong:** cargo-dist produces archives named one way, `self_update` expects another format or different archive internal structure.
**Prevention:** After `cargo dist init`, create a test release and verify asset filenames. Configure `self_update`'s `bin_path_in_archive()` if the binary is nested in a subdirectory within the tar (cargo-dist typically puts binaries at `{name}-{target}/` inside the archive).

### Pitfall 8: GitHub API Rate Limiting
**What goes wrong:** Unauthenticated GitHub API is rate-limited to 60 requests/hour per IP. Users behind corporate NATs share an IP. `panout --update` fails with 403.
**Prevention:** Handle 403 responses with clear error: "GitHub API rate limit exceeded. Try again later or set GITHUB_TOKEN." The `self_update` crate supports `.auth_token()` -- check for `GITHUB_TOKEN` env var.

### Pitfall 9: Binary Replacement Permission Errors
**What goes wrong:** User installed panout to `/usr/local/bin/panout` (requires sudo). `panout --update` fails with EACCES.
**Prevention:** Detect install path with `std::env::current_exe()`. If not writable, print clear error: "Cannot update: /usr/local/bin/panout requires root. Run with sudo or install to ~/.local/bin/"

### Pitfall 10: Nested tmux (Local + Remote) Prefix Key Confusion
**What goes wrong:** Panout runs inside local tmux, creates remote tmux sessions. Prefix key (`Ctrl-B`) is captured by local tmux, making remote session unusable.
**Prevention:** Document the nested tmux situation. Recommend different prefix keys for local and remote. Mention `send-prefix` (pressing prefix twice sends it to inner tmux). This is a UX issue, not a code issue.

## Minor Pitfalls

### Pitfall 11: fzf Not Installed on Remote Machine
**What goes wrong:** User installs sesh but does not have fzf. Running sesh fails with "fzf: command not found."
**Prevention:** Check for fzf at top of script: `command -v fzf >/dev/null 2>&1 || { echo "sesh requires fzf: https://github.com/junegunn/fzf"; exit 1; }`. Also check for tmux.

### Pitfall 12: Sesh Called Outside tmux
**What goes wrong:** User runs sesh from a regular terminal. `tmux switch-client` fails because there is no client to switch.
**Prevention:** Check `$TMUX` env var. If not in tmux, use `tmux attach-session -t $selected` instead of `switch-client`.

### Pitfall 13: sesh Script Not on PATH After Install
**What goes wrong:** `~/.local/bin` is not on `$PATH` by default on many systems (macOS, older Linux, minimal servers). User gets "command not found" after install.
**Prevention:** Install script should check if `~/.local/bin` is in PATH and warn if not. Print the full path so user can run it directly.

### Pitfall 14: Self-Update Continues Execution After Binary Replacement
**What goes wrong:** If panout does work after --update (config ops, etc.), the in-memory code and on-disk binary are different versions.
**Prevention:** `--update` should be an early exit. After successful update, `std::process::exit(0)`. Never continue normal operation after self-replacing.

### Pitfall 15: Sesh Bash-isms in POSIX Script
**What goes wrong:** Writing `#!/bin/sh` but using bash features like `[[ ]]` or arrays.
**Prevention:** Use `#!/bin/bash` explicitly. Bash is ubiquitous on Linux servers where sesh runs.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Release Infrastructure | Asset naming mismatch with self_update | Create test release, verify names match before writing update code |
| Remote Sessions | Missing `-t` flag on SSH | Test manually: `ssh -t host "tmux new-session -A -s test"` |
| Remote Sessions | NotInTmux guard blocks new behavior | Make tmux check conditional: only for local tmux ops, not remote session creation |
| Remote Sessions | send-keys race with shell init | Use `split-window "command"` instead of `split-window` + `send-keys` |
| Remote Sessions | Session name from dots in bundle name | Sanitize: replace dots/colons with hyphens |
| Remote Sessions | Nested tmux prefix confusion | Document, recommend different prefix keys |
| Self-Update | Default features bring in tokio | Always use `default-features = false` with `ureq` feature |
| Self-Update | No releases to download | Build release pipeline first |
| Self-Update | Permission errors replacing binary | Detect path, print clear error |
| Sesh Script | fzf not installed | Dependency check at script start |
| Sesh Script | Not on PATH after install | Check PATH in install script, warn if missing |
| Sesh Script | Called outside tmux | Detect `$TMUX`, fall back to `attach-session` |

## Sources

- [Starting tmux Sessions via SSH](https://blog.jbowen.dev/tips/ssh-tmux/) -- `-t` flag requirement (MEDIUM confidence)
- [tmux wiki - Getting Started](https://github.com/tmux/tmux/wiki/Getting-Started) -- Session name syntax, -A flag (HIGH confidence)
- [self_update GitHub](https://github.com/jaemk/self_update) -- Feature flags, asset naming (HIGH confidence)
- [self_update Cargo.toml](https://github.com/jaemk/self_update/blob/master/Cargo.toml) -- ureq backend deps (HIGH confidence)
- [Checking If tmux Session Exists](https://davidltran.com/blog/check-tmux-session-exists-script/) -- has-session patterns (MEDIUM confidence)
- [Howard Do - Fzf Tmux Session Manager](https://howarddo2208.github.io/posts/04-fzf-tmux-session-manager/) -- fzf integration (MEDIUM confidence)
- [mise self-update](https://mise.jdx.dev/cli/self-update.html) -- Real-world self-update reference (MEDIUM confidence)
- [Fully Automated Releases for Rust Projects](https://blog.orhun.dev/automated-rust-releases/) -- cargo-dist patterns (MEDIUM confidence)
