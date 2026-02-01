# Implementation Plan - Monitoring & Statistics Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion (Monitoring) [checkpoint: ea827c71bdcfb5eb6df4f20e59ff2c8084bb023d]
- [x] Task: Implement Stats and Query Log methods in `AdGuardClient`
    - [x] Add `get_stats` method to `src/adguard.rs`.
    - [x] Add `get_query_log` method to `src/adguard.rs`.
    - [x] Add unit tests for both methods, mocking various API responses.
- [x] Task: Conductor - User Manual Verification 'API Client Expansion (Monitoring)' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation (Monitoring)
- [ ] Task: Register Monitoring tools in `McpServer`
    - [ ] Add `get_stats` tool definition and handler.
    - [ ] Add `get_query_log` tool definition and handler.
    - [ ] Implement percentage calculation logic for `get_stats`.
    - [ ] Verify tool registration via `list_tools`.
- [ ] Task: Conductor - User Manual Verification 'MCP Tool Implementation (Monitoring)' (Protocol in workflow.md)

## Phase 3: Integration Testing (Monitoring)
- [ ] Task: Add Docker Integration Tests for Monitoring
    - [ ] Update `tests/docker_integration_test.rs` to verify data retrieval from a live AdGuard Home instance.
    - [ ] Ensure `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'Integration Testing (Monitoring)' (Protocol in workflow.md)
