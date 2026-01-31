# Specification - Multi-Transport Support (stdio/http)

## Overview
Currently, the AdGuard Home MCP server only supports `stdio` transport. This track adds support for an alternative `http` (SSE) transport, allowing clients to connect over the network. It also introduces authentication for the HTTP transport to ensure security. The implementation will mirror the architecture used in `qbittorrent-mcp-rs`.

## Functional Requirements
- **Transport Selection:** Users can choose between `stdio` and `http` transport modes.
- **Default Behavior:** The server defaults to `stdio` if no transport is specified.
- **Configuration Precedence:**
  1. Command-line arguments (e.g., `--transport`)
  2. Environment variables (e.g., `MCP_TRANSPORT`)
  3. Configuration file (if implemented)
- **HTTP Transport:**
  - Implements the MCP HTTP/SSE transport.
  - Listens on port `3000` by default (configurable).
  - Endpoints:
    - `/sse` (GET): Establish SSE connection.
    - `/message` (POST): Send JSON-RPC requests.
- **Authentication (HTTP only):**
  - **Bearer Token:** Support for a static token in the `Authorization` header (`Bearer <token>`) or query parameter (`?token=<token>`).
- **Unified Tool Dispatching:** The core tool logic should remain identical regardless of the transport used.

## Technical Requirements
- **Server Framework:** Use `axum` for the HTTP/SSE server.
- **State Management:** Use `dashmap` for managing active SSE sessions.
- **Async Runtime:** `tokio` with `tokio-stream` and `futures` for SSE stream handling.
- **HTTP Layer:** `tower-http` for CORS and tracing.
- **Unique IDs:** `uuid` for session management.
- **Architecture:** 
  - `McpServer` struct encapsulates core logic (tools, resources).
  - `run_http_server` function handles Axum setup and routing.
  - `run_stdio` function handles standard input/output loop.
  - `main.rs` dispatches to either `run_http_server` or `run_stdio` based on config.

## Acceptance Criteria
- Running the binary without arguments starts the stdio server.
- Running with `--server-mode http` starts the HTTP server on port 3000.
- HTTP server correctly enforces authentication when configured.
- All tools (e.g., `get_status`) work identically over both transports.
- Unit and integration tests cover both transport modes.

## Out of Scope
- Simultaneous support for multiple transports (only one active transport at a time).
- Integration with external identity providers (LDAP, OAuth).