# Implementation Plan - Multi-Transport Support (stdio/http)

This plan follows the Test-Driven Development (TDD) approach and the Phase Completion Verification protocol defined in the project workflow. It mirrors the `qbittorrent-mcp-rs` implementation.

## Phase 1: Core Logic Refactoring
- [ ] Task: Create `server` module structure
    - [ ] Create `src/server/mod.rs` and `src/server/mcp.rs`.
    - [ ] Move `JsonRpcRequest`, `JsonRpcResponse`, etc., to `src/server/mcp.rs`.
    - [ ] Move `run_stdio` logic from `main.rs` to `src/server/mcp.rs`.
    - [ ] Encapsulate tool logic within `McpServer` struct in `src/server/mcp.rs`.
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
- [ ] Task: Conductor - User Manual Verification 'HTTP/SSE Server Implementation' (Protocol in workflow.md)

## Phase 3: Main Integration
- [ ] Task: Update `Settings` and `main.rs`
    - [ ] Add `server_mode`, `http_port`, `http_auth_token` to `Settings` (Note: CLI parsing might be needed if `config` track isn't done, but basic env var support is sufficient for now).
    - [ ] Update `main.rs` to initialize `McpServer` and call `run_http_server` or `run_stdio` based on settings.
- [ ] Task: Conductor - User Manual Verification 'Main Integration' (Protocol in workflow.md)

## Phase 4: Verification
- [ ] Task: Update Integration Tests
    - [ ] Update `tests/docker_integration_test.rs` to verify `get_status` over HTTP with and without auth (using `reqwest` to hit the HTTP server).
    - [ ] Ensure `task test:ci` passes.
- [ ] Task: Conductor - User Manual Verification 'Verification' (Protocol in workflow.md)