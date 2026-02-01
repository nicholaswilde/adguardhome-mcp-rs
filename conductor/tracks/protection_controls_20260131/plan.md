# Implementation Plan - Protection Control Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion (Protection) [checkpoint: 66c52f11b967b70085f4ee53282d7a02b2ce8171]
- [x] Task: Implement Protection methods in `AdGuardClient`
    - [x] Add `set_protection`, `set_safe_search`, `set_safe_browsing`, and `set_parental_control` methods to `src/adguard.rs`.
    - [x] Add unit tests for all new methods.
- [x] Task: Conductor - User Manual Verification 'API Client Expansion (Protection)' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation (Protection)
- [x] Task: Register Protection tools in `McpServer`
    - [x] Add `set_protection_state`, `set_safe_search`, `set_safe_browsing`, and `set_parental_control` tool definitions and handlers.
    - [x] Verify tool registration via `list_tools`.
- [x] Task: Conductor - User Manual Verification 'MCP Tool Implementation (Protection)' (Protocol in workflow.md)

## Phase 3: Integration Testing (Protection)
- [ ] Task: Add Docker Integration Tests for Protection
    - [ ] Update `tests/docker_integration_test.rs` to verify that toggling settings via MCP actually changes the state in the AdGuard Home instance.
    - [ ] Ensure `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'Integration Testing (Protection)' (Protocol in workflow.md)
