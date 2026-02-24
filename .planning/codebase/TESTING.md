# Testing Patterns

**Analysis Date:** 2026-02-24

## Test Framework

**Runner:**
- Rust built-in test runner (via `cargo test`)
- No external test framework (pytest, jest, vitest, etc.)
- Tests are compiled and executed by the Rust compiler

**Assertion Library:**
- Standard Rust assertions: `assert!()`, `assert_eq!()`, `assert_ne!()`
- No external assertion library (no `pretty_assertions`, `spectral`, etc.)

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test --lib       # Run library tests only
cargo test -- --nocapture  # Run with output printed
cargo test -- --test-threads=1  # Run sequentially
```

## Test File Organization

**Location:**
- Tests are **co-located with source code** in the same files
- Test modules use `#[cfg(test)]` attribute to exclude from release builds
- Pattern: Source code and tests in same `.rs` file

**Naming:**
- Test modules: `mod tests { ... }` (lowercase, conventional name)
- Test functions: `#[test] fn test_<function_name>() { ... }`
- Example: `test_parse_ref_command`, `test_parse_ref_bundle`, `test_parse_ref_group_all`

**Structure:**
```
src/
├── resolver.rs          # Contains resolver logic + test module
├── interpolate.rs       # Contains interpolate logic + test module
├── config.rs            # Contains config logic (no tests)
├── cli.rs               # Contains CLI logic (no tests)
└── [other modules]
```

## Test Structure

**Suite Organization:**
Tests in `src/resolver.rs` show the standard pattern:
```rust
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
```

**Patterns:**
- No explicit test setup/teardown (no `#[before_each]` or equivalent)
- Each test is independent and self-contained
- Tests use `assert_eq!()` for equality checks
- Tests in `src/interpolate.rs` follow same pattern with multiple assertions

## Mocking

**Framework:** No external mocking library

**Patterns:**
- Tests use concrete implementations, not mocks
- Example from `src/interpolate.rs`: Tests call pure functions directly with test inputs
```rust
#[test]
fn test_interpolate() {
    assert_eq!(
        interpolate("cd /home/{user}/src", "admin", "192.168.1.1"),
        "cd /home/admin/src"
    );
}
```

**What to Mock:**
- External system calls (tmux, SSH) - NOT currently mocked; tested manually
- File I/O operations - NOT currently mocked
- Environment variables - NOT currently mocked

**What NOT to Mock:**
- Pure functions that take inputs and return outputs (tested directly)
- Data structure parsing and manipulation
- Business logic without side effects

## Fixtures and Factories

**Test Data:**
- Tests use literal string values for test inputs
- Example from `src/resolver.rs`: Uses string literals like `"echo hello"`, `"@dev.frontend"`, `"@dev.*"`
- Example from `src/interpolate.rs`: Uses string literals for hosts and commands

**Location:**
- No centralized fixtures directory
- Test data defined inline within test functions
- No factory pattern or test data builders

## Coverage

**Requirements:** None enforced

**View Coverage:**
- No coverage tooling configured
- Can be done manually with `cargo tarpaulin` or `cargo llvm-cov` if desired

## Test Types

**Unit Tests:**
- Scope: Individual functions and modules
- Approach: Direct function calls with known inputs and expected outputs
- Examples: `test_parse_ref_command()`, `test_parse_host()`, `test_interpolate()`
- Location: `src/resolver.rs`, `src/interpolate.rs`

**Integration Tests:**
- Not implemented
- Would require actual tmux session to test tmux operations
- File I/O operations in `src/loader.rs` not tested
- Circular reference detection in `src/resolver.rs` not tested

**E2E Tests:**
- Not used
- Manual testing in tmux environment would be required

## Common Patterns

**Assertion Pattern:**
The codebase uses simple equality assertions:
```rust
assert_eq!(actual, expected);
```

No custom assertion macros or assertion libraries.

**String/Type Assertions:**
```rust
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
```

**Multiple Assertions in Single Test:**
```rust
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
```

## Tested Modules

**src/resolver.rs:**
- Tests the reference parsing logic
- 3 test functions covering:
  - Plain command parsing: `test_parse_ref_command()`
  - Bundle reference parsing: `test_parse_ref_bundle()`
  - Group wildcard parsing: `test_parse_ref_group_all()`
- Tests focus on the `parse_ref()` function
- Missing tests: Recursive resolution, circular reference detection, bundle not found cases

**src/interpolate.rs:**
- Tests the variable substitution logic
- 2 test functions covering:
  - Host parsing: `test_parse_host()` - Both valid and invalid inputs
  - Command interpolation: `test_interpolate()` - Both `{user}` and `{ip}` substitution
- Example test case: `parse_host("admin@192.168.1.1")` returns `Some(("admin", "192.168.1.1"))`

## Untested Modules

**src/config.rs:**
- No unit tests
- Complex TOML parsing logic not tested
- Risk: Config parsing errors could go undetected

**src/cli.rs:**
- No unit tests
- `Cli::layout()` method not tested
- Risk: CLI flag handling could be broken

**src/loader.rs:**
- No unit tests
- File I/O and path resolution not tested
- Risk: Config file discovery logic could be broken

**src/tmux.rs:**
- No unit tests
- Tmux command execution not tested
- Risk: All tmux operations are untested; requires manual verification

**src/main.rs:**
- No unit tests
- Main application logic not tested
- Risk: Bundle and workspace execution flow untested

**src/ssh.rs:**
- No unit tests
- SSH connection logic not tested
- Risk: SSH session management untested

## Testing Strategy

**Current Approach:**
- Unit test pure functions only
- Avoid testing functions with external dependencies (tmux, file I/O)
- Manual testing required for integration scenarios

**How to Add Tests:**
1. For pure functions like `config::Config::from_str()`: Write unit tests with TOML string literals
2. For CLI logic: Write tests with mock `Cli` structs
3. For tmux operations: Consider integration tests with actual tmux session, or mock Command execution
4. For file I/O: Use temporary directories (`tempfile` crate) for isolated tests

**Example to Add Config Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_bundle() {
        let toml = r#"
            [dev.frontend]
            cmd = "npm run dev"
            pane = 0
        "#;

        let config = Config::from_str(toml).unwrap();
        assert_eq!(config.list_bundles().len(), 1);
        assert!(config.get_bundle("dev.frontend").is_some());
    }
}
```

---

*Testing analysis: 2026-02-24*
