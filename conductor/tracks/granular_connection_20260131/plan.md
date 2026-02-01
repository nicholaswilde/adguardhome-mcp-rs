# Implementation Plan - Granular AdGuard Connection Settings

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: Configuration Refactoring [checkpoint: 651fab8fe5ad6bb7f2e4b67880b5f7844cc54042]
- [x] Task: Red Phase - Update Configuration Unit Tests
    - [x] Update tests in `src/config.rs` to expect `adguard_host` and `adguard_port` instead of `adguard_url`.
    - [x] Run `cargo test` and confirm compilation errors/failures.
- [x] Task: Green Phase - Update `AppConfig` and Mapping
    - [x] Remove `adguard_url` from `AppConfig` in `src/config.rs`.
    - [x] Add `adguard_host` and `adguard_port` (with default `3000`) to `AppConfig`.
    - [x] Update `parse_args` and `load` mapping logic.
    - [x] Run `task test:ci` and ensure unit tests pass.
- [x] Task: Conductor - User Manual Verification 'Configuration Refactoring' (Protocol in workflow.md)

## Phase 2: Client and Core Logic Refactoring
- [x] Task: Update `AdGuardClient` Implementation
    - [x] Update `src/adguard.rs` to store `adguard_host` and `adguard_port`.
    - [x] Update URL construction logic in `get_status` to use `http://{host}:{port}`.
- [x] Task: Update `main.rs` Integration
    - [x] Update `main.rs` to correctly initialize `AdGuardClient` and `ToolRegistry` with the new config fields.
- [x] Task: Verify Compilation
    - [x] Run `cargo check` to ensure all internal references are updated.
- [x] Task: Conductor - User Manual Verification 'Client and Core Logic Refactoring' (Protocol in workflow.md)

## Phase 3: Integration Test Synchronization
- [ ] Task: Update Docker Integration Tests
    - [ ] Update `tests/docker_integration_test.rs` to provide host and port separately to the client and config objects.
- [ ] Task: Final Quality Gate
    - [ ] Run `task test:ci` to ensure all tests (unit and integration) pass.
- [ ] Task: Conductor - User Manual Verification 'Integration Test Synchronization' (Protocol in workflow.md)
