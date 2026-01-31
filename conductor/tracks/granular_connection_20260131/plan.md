# Implementation Plan - Granular AdGuard Connection Settings

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: Configuration Refactoring
- [ ] Task: Red Phase - Update Configuration Unit Tests
    - [ ] Update tests in `src/config.rs` to expect `adguard_host` and `adguard_port` instead of `adguard_url`.
    - [ ] Run `cargo test` and confirm compilation errors/failures.
- [ ] Task: Green Phase - Update `AppConfig` and Mapping
    - [ ] Remove `adguard_url` from `AppConfig` in `src/config.rs`.
    - [ ] Add `adguard_host` and `adguard_port` (with default `3000`) to `AppConfig`.
    - [ ] Update `parse_args` and `load` mapping logic.
    - [ ] Run `task test:ci` and ensure unit tests pass.
- [ ] Task: Conductor - User Manual Verification 'Configuration Refactoring' (Protocol in workflow.md)

## Phase 2: Client and Core Logic Refactoring
- [ ] Task: Update `AdGuardClient` Implementation
    - [ ] Update `src/adguard.rs` to store `adguard_host` and `adguard_port`.
    - [ ] Update URL construction logic in `get_status` to use `http://{host}:{port}`.
- [ ] Task: Update `main.rs` Integration
    - [ ] Update `main.rs` to correctly initialize `AdGuardClient` and `ToolRegistry` with the new config fields.
- [ ] Task: Verify Compilation
    - [ ] Run `cargo check` to ensure all internal references are updated.
- [ ] Task: Conductor - User Manual Verification 'Client and Core Logic Refactoring' (Protocol in workflow.md)

## Phase 3: Integration Test Synchronization
- [ ] Task: Update Docker Integration Tests
    - [ ] Update `tests/docker_integration_test.rs` to provide host and port separately to the client and config objects.
- [ ] Task: Final Quality Gate
    - [ ] Run `task test:ci` to ensure all tests (unit and integration) pass.
- [ ] Task: Conductor - User Manual Verification 'Integration Test Synchronization' (Protocol in workflow.md)
