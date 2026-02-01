# Implementation Plan - Filtering & Blocklist Management Tools

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow.

## Phase 1: API Client Expansion (Filtering) [checkpoint: f4f56b851eb517c04d41cde6dea5a863c52c5235]
- [x] Task: Implement Filtering methods in `AdGuardClient`
    - [x] Add `list_filters`, `toggle_filter`, and `add_filter` methods to `src/adguard.rs`.
    - [x] Add unit tests for filtering API interactions.
- [x] Task: Conductor - User Manual Verification 'API Client Expansion (Filtering)' (Protocol in workflow.md)

## Phase 2: MCP Tool Implementation (Filtering) [checkpoint: a8334d0307c8e1d2bc4bd43f9e70f2cfb554c7e7]
- [x] Task: Register Filtering tools in `McpServer`
    - [x] Add `list_filter_lists` tool definition and handler.
    - [x] Add `toggle_filter_list` tool definition and handler.
    - [x] Add `add_filter_list` tool definition and handler.
    - [x] Implement logic to find filters by either Name or ID.
    - [x] Verify tool registration via `list_tools`.
- [x] Task: Conductor - User Manual Verification 'MCP Tool Implementation (Filtering)' (Protocol in workflow.md)

## Phase 3: Integration Testing (Filtering) [checkpoint: 4adb31bca9c4a5194907196092f8d66adc861a42]
- [x] Task: Add Docker Integration Tests for Filtering
    - [x] Update `tests/docker_integration_test.rs` to verify adding and toggling filters on a live instance.
    - [x] Ensure `task test:ci` passes.
- [x] Task: Conductor - User Manual Verification 'Integration Testing (Filtering)' (Protocol in workflow.md)
