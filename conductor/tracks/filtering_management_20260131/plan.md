# Implementation Plan - Filtering & Blocklist Management Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion (Filtering) [checkpoint: f4f56b851eb517c04d41cde6dea5a863c52c5235]
- [x] Task: Implement Filtering methods in `AdGuardClient`
    - [x] Add `list_filters`, `toggle_filter`, and `add_filter` methods to `src/adguard.rs`.
    - [x] Add unit tests for filtering API interactions.
- [x] Task: Conductor - User Manual Verification 'API Client Expansion (Filtering)' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation (Filtering)
- [ ] Task: Register Filtering tools in `McpServer`
    - [ ] Add `list_filter_lists`, `toggle_filter_list`, and `add_filter_list` tool definitions and handlers.
    - [ ] Implement logic to find filters by either Name or ID.
    - [ ] Verify tool registration via `list_tools`.
- [ ] Task: Conductor - User Manual Verification 'MCP Tool Implementation (Filtering)' (Protocol in workflow.md)

## Phase 3: Integration Testing (Filtering)
- [ ] Task: Add Docker Integration Tests for Filtering
    - [ ] Update `tests/docker_integration_test.rs` to verify adding and toggling filters on a live instance.
    - [ ] Ensure `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'Integration Testing (Filtering)' (Protocol in workflow.md)
