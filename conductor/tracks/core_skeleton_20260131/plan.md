# Implementation Plan - Core Skeleton & Status Retrieval

This plan follows a Test-Driven Development (TDD) approach as defined in the project workflow.

## Phase 1: Project Scaffolding
- [ ] Task: Initialize Cargo project and configure dependencies
    - [ ] Run `cargo init`
    - [ ] Add `tokio`, `serde`, `serde_json`, `reqwest`, `tracing`, `thiserror` to `Cargo.toml`
- [ ] Task: Define core Error and Config types
    - [ ] Write tests for configuration loading from environment variables
    - [ ] Implement `Config` struct and custom `Error` enum
- [ ] Task: Conductor - User Manual Verification 'Project Scaffolding' (Protocol in workflow.md)

## Phase 2: MCP Server Foundation
- [ ] Task: Implement stdio transport and message loop
    - [ ] Write unit tests for JSON-RPC message parsing
    - [ ] Implement the main async loop to read from stdin and write to stdout
- [ ] Task: Implement Tool Dispatcher
    - [ ] Write tests for tool registration and lookup
    - [ ] Implement a basic dispatcher to handle `list_tools` and `call_tool`
- [ ] Task: Conductor - User Manual Verification 'MCP Server Foundation' (Protocol in workflow.md)

## Phase 3: AdGuard Home Integration
- [ ] Task: Implement AdGuard Home API Client
    - [ ] Write tests using `wiremock` or similar to mock AdGuard Home API
    - [ ] Implement client logic to fetch `/control/status`
- [ ] Task: Implement `get_status` Tool
    - [ ] Write integration test for the tool call
    - [ ] Wire the `get_status` tool to the API client
- [ ] Task: Conductor - User Manual Verification 'AdGuard Home Integration' (Protocol in workflow.md)
