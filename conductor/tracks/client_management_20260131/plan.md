# Implementation Plan - Client Management Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion (Clients)
- [x] Task: Implement Client methods in `AdGuardClient`
    - [x] Add `list_clients` and `get_client_info` methods to `src/adguard.rs`.
    - [x] Add unit tests for client API calls.
- [~] Task: Conductor - User Manual Verification 'API Client Expansion (Clients)' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation (Clients)
- [ ] Task: Register Client tools in `McpServer`
    - [ ] Add `list_clients` and `get_client_info` tool definitions and handlers.
    - [ ] Implement logic to resolve identifiers (IP/MAC/Name) to API targets.
    - [ ] Verify tool registration via `list_tools`.
- [ ] Task: Conductor - User Manual Verification 'MCP Tool Implementation (Clients)' (Protocol in workflow.md)

## Phase 3: Integration Testing (Clients)
- [ ] Task: Add Docker Integration Tests for Clients
    - [ ] Update `tests/docker_integration_test.rs` to verify client listing and detailed info retrieval.
    - [ ] Ensure `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'Integration Testing (Clients)' (Protocol in workflow.md)
