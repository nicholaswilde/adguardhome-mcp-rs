# Implementation Plan: Version Command Line Switch

## Phase 1: Implementation and Verification

- [ ] Task: Write Integration Tests
    - [ ] Create a new test in `tests/` (or add to an existing one) that executes the binary with `--version` and `-V`.
    - [ ] Verify the output matches `adguardhome-mcp-rs <version>` and the exit code is 0.
    - [ ] Verify the tests fail (Red Phase).

- [ ] Task: Implement Version Flag
    - [ ] Update the `clap` configuration in `src/main.rs` (or where arguments are defined) to include the version information.
    - [ ] Use `env!("CARGO_PKG_VERSION")` to pull the version from `Cargo.toml`.
    - [ ] Ensure the output format is exactly `adguardhome-mcp-rs <version>`.
    - [ ] Verify the tests pass (Green Phase).

- [ ] Task: Quality Gate Verification
    - [ ] Run `task test` to ensure all tests pass.
    - [ ] Run `task lint` and `task format` to ensure code quality.
    - [ ] Verify code coverage remains above 80%.

- [ ] Task: Conductor - User Manual Verification 'Phase 1: Implementation and Verification' (Protocol in workflow.md)
