# Implementation Plan - Client Management Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion (Clients) [checkpoint: 1052d378b1468ab414d0a44fb5168c8368680355]
- [x] Task: Implement Client methods in `AdGuardClient`
    - [x] Add `list_clients` and `get_client_info` methods to `src/adguard.rs`.
    - [x] Add unit tests for client API calls.
- [x] Task: Conductor - User Manual Verification 'API Client Expansion (Clients)' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation (Clients)
- [x] Task: Register Client tools in `McpServer`
    - [x] Add `list_clients` tool definition and handler.
    - [x] Add `get_client_info` tool definition and handler.
    - [x] Implement logic to resolve identifiers (IP/MAC/Name) to API targets.
    - [x] Verify tool registration via `list_tools`.
- [x] Task: Conductor - User Manual Verification 'MCP Tool Implementation (Clients)' (Protocol in workflow.md)

## Phase 3: Integration Testing (Clients)
- [ ] Task: Add Docker Integration Tests for Clients
    - [ ] Update `tests/docker_integration_test.rs` to verify client listing and detailed info retrieval.
    - [ ] Ensure `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'Integration Testing (Clients)' (Protocol in workflow.md)
