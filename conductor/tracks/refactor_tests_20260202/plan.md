# Plan - Refactor tests/ (refactor_tests_20260202)

## Phase 1: Common Infrastructure & Helpers [checkpoint: 382e49b]
Establish the foundational helpers to reduce boilerplate in all integration tests.

- [x] Task: Create `tests/common/` directory and initialize `mod.rs`
- [x] Task: Implement `Unified AdGuard Home Container Helper`
    - [ ] Define a struct/function to manage `testcontainers` lifecycle
    - [ ] Standardize container configuration (env vars, ports, health checks)
- [x] Task: Implement `Tool Execution Helpers`
    - [ ] Create macros or functions to wrap `stdio` and `http` MCP calls
    - [ ] Standardize assertions on `CallToolResult`
- [x] Task: Implement `Transport-Agnostic Test Runner`
    - [ ] Create a pattern to execute a test closure against multiple transport configurations
- [x] Task: Conductor - User Manual Verification 'Common Infrastructure & Helpers' (Protocol in workflow.md)

## Phase 2: Domain Split & Migration (Protection)
Migrate the priority domain to the new structure as a proof of concept for the refactor.

- [x] Task: Create `tests/protection_tests.rs`
- [x] Task: Migrate protection-related tests from `docker_integration_test.rs`
    - [x] Global protection toggles
    - [x] Safe Search settings
    - [x] Parental Control settings
- [x] Task: Refactor migrated tests to use Phase 1 helpers
- [x] Task: Verify `protection_tests.rs` passes against both transports
- [~] Task: Conductor - User Manual Verification 'Domain Split & Migration (Protection)' (Protocol in workflow.md)

## Phase 3: Suite Consolidation & Modernization
Migrate remaining tests and finalize the suite structure.

- [ ] Task: Migrate remaining domains (Filtering, DNS, Clients, System) to focused test files
- [ ] Task: Update `docker_integration_test.rs` or remove if fully migrated
- [ ] Task: Final Quality Audit
    - [ ] `task lint`
    - [ ] `task format`
- [ ] Task: Execute full suite verification
    - [ ] `RUN_DOCKER_TESTS=true task test:integration`
- [ ] Task: Conductor - User Manual Verification 'Suite Consolidation & Modernization' (Protocol in workflow.md)
