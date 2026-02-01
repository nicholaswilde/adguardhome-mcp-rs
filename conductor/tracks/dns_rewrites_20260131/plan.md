# Implementation Plan - DNS Rewrite Management Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion [checkpoint: 55dc71958a6b97ebb7adc31f334a2f28f63855e1]
- [x] Task: Implement DNS Rewrite methods in `AdGuardClient`
    - [x] Add `list_rewrites`, `add_rewrite`, and `delete_rewrite` methods to `src/adguard.rs`.
    - [x] Update `AdGuardClient` unit tests to cover success and failure cases for these new methods.
- [x] Task: Conductor - User Manual Verification 'API Client Expansion' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation [checkpoint: aab6ce86e03481ced645ecd5926f0dcb5fa1e8d9]
- [x] Task: Register DNS Rewrite tools in `McpServer`
    - [x] Add `list_dns_rewrites` tool definition and handler.
    - [x] Add `add_dns_rewrite` tool definition and handler.
    - [x] Add `remove_dns_rewrite` tool definition and handler.
    - [x] Verify tool registration via binary execution (`list_tools`).
- [x] Task: Conductor - User Manual Verification 'MCP Tool Implementation' (Protocol in workflow.md)

## Phase 3: Integration Testing [checkpoint: 32d714828f0ab2107ed1707281219f04e379239b]
- [x] Task: Add Docker Integration Tests for DNS Rewrites
    - [x] Update `tests/docker_integration_test.rs` to verify adding an entry, listing it, and then removing it.
    - [x] Ensure `task test:ci` passes with 100% success.
- [x] Task: Conductor - User Manual Verification 'Integration Testing' (Protocol in workflow.md)
