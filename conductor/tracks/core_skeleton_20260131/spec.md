# Specification - Core Skeleton & Status Retrieval

## Overview
This track focuses on setting up the foundational Rust project structure for the AdGuard Home MCP server and implementing the first functional tool: `get_status`.

## Functional Requirements
- Initialize a Rust binary project.
- Implement the MCP JSON-RPC server transport (stdio).
- Define a tool `get_status` that queries the AdGuard Home `/control/status` endpoint.
- Return the AdGuard Home version and current protection state (enabled/disabled).

## Technical Requirements
- **Language:** Rust
- **Runtime:** `tokio`
- **MCP Framework:** Manual JSON-RPC implementation or compatible SDK.
- **HTTP Client:** `reqwest`
- **Logging:** `tracing` and `tracing-subscriber`.
- **Error Handling:** `thiserror` for internal errors.

## Security Considerations
- Ensure AdGuard Home API credentials (URL, username, password) are handled via environment variables, not hardcoded.
