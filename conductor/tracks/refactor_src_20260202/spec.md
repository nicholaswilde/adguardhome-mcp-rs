# Specification - Refactor src/ (refactor_src_20260202)

## Overview
This track involves a comprehensive refactoring of the `src/` directory to improve modularity, separation of concerns, and testability. The goal is to evolve the current flat file structure into a more organized, hierarchical module system while maintaining the existing logic and functionality.

## Functional Requirements
- **Module Reorganization:**
    - Convert `src/adguard.rs` into a directory `src/adguard/` with a `mod.rs` and child modules (e.g., `client.rs`, `models.rs`).
    - Convert `src/tools.rs` into a directory `src/tools/` with a `mod.rs` and domain-specific child modules (e.g., `filtering.rs`, `dns.rs`, `protection.rs`).
- **Separation of Concerns:**
    - Ensure the AdGuard API client (the "how") is strictly decoupled from the MCP tool definitions (the "what").
    - Clarify the relationship between `src/mcp.rs` and `src/server/mcp.rs`.
- **Maintain Functionality:**
    - The refactor must not introduce changes to the external behavior of the MCP server or its tools.
    - All existing features (Lazy Mode, HTTP/Stdio transports, etc.) must remain fully functional.

## Non-Functional Requirements
- **Testability:** The new structure should facilitate better unit testing by allowing easier mocking of the AdGuard API client.
- **Modularity:** Large files should be broken down into logically related components.
- **Idiomatic Rust:** Adhere to standard Rust project structures and module naming conventions.

## Acceptance Criteria
- [ ] `src/` directory is reorganized into logical sub-modules.
- [ ] No monolithic files (>500 lines) remain where splitting is logical.
- [ ] All existing unit and integration tests pass without modification to the test logic itself (only import paths should change).
- [ ] `cargo clippy` and `cargo fmt` pass without warnings.
- [ ] The server starts and responds correctly on both `stdio` and `http` transports.

## Out of Scope
- Implementation of new MCP tools or AdGuard API features.
- Performance optimization (unless a direct result of the refactor).
- Database migrations or changes to external configuration formats.
