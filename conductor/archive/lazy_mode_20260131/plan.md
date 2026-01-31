# Implementation Plan - Lazy Mode

## Phase 1: Foundation & Configuration
- [x] Task: Add `LAZY_MODE` to `Settings`
    - [x] Add field to `Settings` struct.
    - [x] Update `from_env` to parse it.
    - [x] Add unit tests for configuration.
- [x] Task: Implement `ToolRegistry` and `Notification` support
    - [x] Refactor `main.rs` to move tool definition logic into a `ToolRegistry`.
    - [x] Implement `notifications/tools/list_changed` JSON-RPC message structure in `mcp.rs`.
    - [x] Add helper to send notifications to stdout.
- [x] Task: Conductor - User Manual Verification 'Foundation' (Protocol in workflow.md)

## Phase 2: Meta Tool Implementation
- [x] Task: Implement `manage_tools` logic
    - [x] Implement the `list`, `enable`, `disable` actions in `ToolRegistry`.
    - [x] Ensure `list_tools` response respects the current registry state.
    - [x] Trigger notification on state change.
- [x] Task: Integration & Logic Update
    - [x] Wire up `manage_tools` in the main dispatch loop.
    - [x] Ensure `get_status` (and future tools) are hidden by default when lazy mode is on.
- [x] Task: Conductor - User Manual Verification 'Lazy Mode' (Protocol in workflow.md)
