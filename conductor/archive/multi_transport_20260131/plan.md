# Implementation Plan - Multi-Transport Support (stdio/http)

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow. It mirrors the `qbittorrent-mcp-rs` implementation.

## Phase 1: Core Logic Refactoring
- [x] Task: Create `server` module structure
    - [x] Create `src/server/mod.rs` and `src/server/mcp.rs`.
    - [x] Move `JsonRpcRequest`, `JsonRpcResponse`, etc., to `src/server/mcp.rs`.
    - [x] Move `run_stdio` logic from `main.rs` to `src/server/mcp.rs`.
    - [x] Encapsulate tool logic within `McpServer` struct in `src/server/mcp.rs`.
- [x] Task: Verify Refactoring
    - [x] Run `task test:ci` to ensure stdio mode still works and no regressions were introduced.
- [x] Task: Conductor - User Manual Verification 'Core Logic Refactoring' (Protocol in workflow.md)

## Phase 2: HTTP/SSE Server Implementation
- [x] Task: Add Dependencies
    - [x] Add `axum`, `tower-http`, `futures`, `tokio-stream`, `dashmap`, `uuid` to `Cargo.toml`.
- [x] Task: Implement `src/server/http.rs`
    - [x] Implement `AppState` to hold `McpServer` and sessions.
    - [x] Implement `/sse` handler for session creation.
    - [x] Implement `/message` handler for request processing.
    - [x] Implement `auth_middleware` for token verification.
    - [x] Implement `run_http_server` entry point.
- [x] Task: Add HTTP Integration Tests (TDD)
    - [x] Update `tests/docker_integration_test.rs` to include a test case that starts the server in HTTP mode and connects via `reqwest`.
    - [x] Verify `task test:ci` passes.
- [x] Task: Conductor - User Manual Verification 'HTTP/SSE Server Implementation' (Protocol in workflow.md)

## Phase 3: Main Integration
- [x] Task: Update `Settings` and `main.rs`
    - [x] Add `server_mode`, `http_port`, `http_auth_token` to `Settings`.
    - [x] Update `main.rs` to initialize `McpServer` and call `run_http_server` or `run_stdio` based on settings.
- [x] Task: Verify Integration
    - [x] Update `tests/docker_integration_test.rs` to cover both `stdio` (default) and `http` modes via `server_mode` setting.
    - [x] Run `task test:ci` to ensure full coverage.
- [x] Task: Conductor - User Manual Verification 'Main Integration' (Protocol in workflow.md)

## Phase 4: Final Validation and Quality Gates
- [x] Task: Comprehensive Integration Testing
    - [x] Verify `get_status` works identically over `stdio` and `http`.
    - [x] Verify authentication enforcement (401 Unauthorized) in `http` mode.
    - [x] Verify `task test:ci` passes 100%.
- [x] Task: Conductor - User Manual Verification 'Final Validation' (Protocol in workflow.md)
