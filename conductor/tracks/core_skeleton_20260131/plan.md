# Implementation Plan - Core Skeleton & Status Retrieval

This plan follows a Test-Driven Development (TDD) approach as defined in the project workflow.

## Phase 1: Project Scaffolding
- [x] Task: Initialize Cargo project and configure dependencies
    - [x] Run `cargo init`
    - [x] Add `tokio`, `serde`, `serde_json`, `reqwest`, `tracing`, `thiserror` to `Cargo.toml`
- [x] Task: Define core Error and Config types
    - [x] Write tests for configuration loading from environment variables
    - [x] Implement `Config` struct and custom `Error` enum
- [x] Task: Conductor - User Manual Verification 'Project Scaffolding' (Protocol in workflow.md)

## Phase 2: MCP Server Foundation
- [x] Task: Implement stdio transport and message loop
    - [x] Write unit tests for JSON-RPC message parsing
    - [x] Implement the main async loop to read from stdin and write to stdout
- [x] Task: Implement Tool Dispatcher
    - [x] Write tests for tool registration and lookup
    - [x] Implement a basic dispatcher to handle `list_tools` and `call_tool`
- [ ] Task: Conductor - User Manual Verification 'MCP Server Foundation' (Protocol in workflow.md)

## Phase 3: AdGuard Home Integration
- [x] Task: Implement AdGuard Home API Client
    - [x] Write tests using `wiremock` or similar to mock AdGuard Home API
    - [x] Implement client logic to fetch `/control/status`
- [x] Task: Implement `get_status` Tool
    - [x] Write integration test for the tool call
    - [x] Wire the `get_status` tool to the API client
- [ ] Task: Conductor - User Manual Verification 'AdGuard Home Integration' (Protocol in workflow.md)

## Phase 4: Docker Integration Testing
- [x] Task: Implement Docker Integration Test
    - [x] Add testcontainers dependency
    - [x] Create integration test using adguard/adguardhome image
    - [x] Verify connectivity and get_status tool against a real instance
- [ ] Task: Conductor - User Manual Verification 'Docker Integration Testing' (Protocol in workflow.md)
