# Implementation Plan - Multi-Transport Support (stdio/http)

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow. It mirrors the `qbittorrent-mcp-rs` implementation.

## Phase 1: Core Logic Refactoring
- [ ] Task: Create `server` module structure
    - [ ] Create `src/server/mod.rs` and `src/server/mcp.rs`.
    - [ ] Move `JsonRpcRequest`, `JsonRpcResponse`, etc., to `src/server/mcp.rs`.
    - [ ] Move `run_stdio` logic from `main.rs` to `src/server/mcp.rs`.
    - [ ] Encapsulate tool logic within `McpServer` struct in `src/server/mcp.rs`.
- [ ] Task: Verify Refactoring
    - [ ] Run `task test:ci` to ensure stdio mode still works and no regressions were introduced.
- [ ] Task: Conductor - User Manual Verification 'Core Logic Refactoring' (Protocol in workflow.md)

## Phase 2: HTTP/SSE Server Implementation
- [ ] Task: Add Dependencies
    - [ ] Add `axum`, `tower-http`, `futures`, `tokio-stream`, `dashmap`, `uuid` to `Cargo.toml`.
- [ ] Task: Implement `src/server/http.rs`
    - [ ] Implement `AppState` to hold `McpServer` and sessions.
    - [ ] Implement `/sse` handler for session creation.
    - [ ] Implement `/message` handler for request processing.
    - [ ] Implement `auth_middleware` for token verification.
    - [ ] Implement `run_http_server` entry point.
- [ ] Task: Add HTTP Integration Tests (TDD)
    - [ ] Update `tests/docker_integration_test.rs` to include a test case that starts the server in HTTP mode and connects via `reqwest`.
    - [ ] Verify `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'HTTP/SSE Server Implementation' (Protocol in workflow.md)

## Phase 3: Main Integration
- [ ] Task: Update `Settings` and `main.rs`
    - [ ] Add `server_mode`, `http_port`, `http_auth_token` to `Settings`.
    - [ ] Update `main.rs` to initialize `McpServer` and call `run_http_server` or `run_stdio` based on settings.
- [ ] Task: Verify Integration
    - [ ] Update `tests/docker_integration_test.rs` to cover both `stdio` (default) and `http` modes via `server_mode` setting.
    - [ ] Run `task test:ci` to ensure full coverage.
- [ ] Task: Conductor - User Manual Verification 'Main Integration' (Protocol in workflow.md)

## Phase 4: Final Validation and Quality Gates
- [ ] Task: Comprehensive Integration Testing
    - [ ] Verify `get_status` works identically over `stdio` and `http`.
    - [ ] Verify authentication enforcement (401 Unauthorized) in `http` mode.
    - [ ] Verify `task test:ci` passes 100%.
- [ ] Task: Conductor - User Manual Verification 'Final Validation' (Protocol in workflow.md)
