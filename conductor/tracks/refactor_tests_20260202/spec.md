# Specification - Refactor tests/ (refactor_tests_20260202)

## Overview
This track involves a comprehensive refactoring of the `tests/` directory to improve organization, reduce boilerplate, and modernize the integration testing suite. The current monolithic `docker_integration_test.rs` will be broken down into domain-specific modules, and common logic will be extracted into shared helpers.

## Functional Requirements
- **Test Suite Reorganization:**
    - Split `tests/docker_integration_test.rs` into smaller, focused files.
    - **Priority:** Implement `tests/protection_tests.rs` covering global toggles, safe search, and parental controls.
- **Common Helper Extraction:**
    - Create a `tests/common/` module to house shared logic.
    - Implement a **Unified AdGuard Home Container Helper** for standardized container lifecycle management.
    - Implement **Tool Execution Macros/Helpers** to simplify MCP tool calls and assertions.
- **Modernization & Reliability:**
    - Standardize async setup with `tokio::test` and `testcontainers` to ensure reliable execution.
    - Implement a **Transport-Agnostic Test Runner** pattern to allow running tests against both `stdio` and `http` transports with minimal duplication.

## Non-Functional Requirements
- **Maintainability:** Reduce the overhead of adding new integration tests by providing robust helpers.
- **Reliability:** Minimize flakiness in Docker-based tests through better cleanup and port management.
- **Readability:** Ensure test intent is clear by hiding low-level MCP/HTTP details behind helpers.

## Acceptance Criteria
- [ ] `tests/common/` is established and utilized by all integration tests.
- [ ] `tests/protection_tests.rs` is created and contains the relevant logic extracted from the original monolithic test.
- [ ] Integration tests can be easily toggled to run against both `stdio` and `http` transports.
- [ ] The full test suite passes consistently: `RUN_DOCKER_TESTS=true task test:integration`.
- [ ] Code duplication in the `tests/` directory is significantly reduced.

## Out of Scope
- Adding new feature tests that don't already exist in the codebase.
- Refactoring of unit tests within `src/` (this track focuses on `tests/` integration suite).
