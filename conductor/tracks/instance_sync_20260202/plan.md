# Implementation Plan: AdGuard Home Instance Sync

## Phase 1: Configuration and Multi-Instance Support [checkpoint: 4986302]
This phase focuses on extending the configuration system to handle multiple replica instances and sync-specific settings.

- [x] Task: Extend Configuration and Data Structures
    - [ ] Update `src/config.rs` to support a list of replica instances (URL and API Key).
    - [ ] Add support for loading replicas from environment variables (e.g., `ADGUARD_REPLICAS` as a JSON string or delimited list).
    - [ ] Add sync-related settings to `Config` (interval, default sync mode).
    - [ ] **TDD:** Write unit tests in `src/config.rs` to verify replica loading from file and env vars.
- [x] Task: Define Sync Data Models
    - [ ] Create data structures in a new module (e.g., `src/sync.rs`) to represent the aggregate configuration state (filtering, rules, clients, etc.).
    - [ ] **TDD:** Write unit tests for serialization/deserialization of these models.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Configuration and Multi-Instance Support' (Protocol in workflow.md)

## Phase 2: Implementation of Sync Logic [checkpoint: 7a00bf1]
This phase involves the logic for fetching data from the master and applying it to replicas with support for different modes.

- [x] Task: Implement Master Data Retrieval
    - [ ] Implement methods in `AdGuardClient` or a dedicated sync service to fetch all scoped configuration (Filtering, Rules, Clients, DNS, Blocked Services).
    - [ ] **TDD:** Write unit tests with `Wiremock` to ensure all data is correctly aggregated.
- [x] Task: Implement Replica Application Logic (Push)
    - [ ] Implement logic to compare and apply configuration to a replica.
    - [ ] Support "Full Overwrite" mode (delete items on replica not in master).
    - [ ] Support "Additive Merge" mode (only add missing items).
    - [ ] **TDD:** Write unit tests for both Overwrite and Merge logic using mocked responses.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Implementation of Sync Logic' (Protocol in workflow.md)

## Phase 3: MCP Interface and Background Sync [checkpoint: 57a7a18]
This phase exposes the sync functionality via the MCP server and implements the automated background task.

- [x] Task: Implement `sync_instances` MCP Tool
    - [ ] Register a new tool `sync_instances` in `src/mcp.rs`.
    - [ ] Allow optional parameters to override configured replicas and sync mode.
    - [ ] **TDD:** Write unit tests in `src/mcp.rs` to verify tool registration and argument handling.
- [x] Task: Implement Background Sync Task
    - [ ] Implement a background loop (using `tokio::time::interval`) that triggers a sync based on the configuration.
    - [ ] Ensure the background task handles errors gracefully without crashing the server.
    - [ ] **TDD:** Write unit tests to verify the interval-based triggering logic.
- [x] Task: Conductor - User Manual Verification 'Phase 3: MCP Interface and Background Sync' (Protocol in workflow.md)

## Phase 4: Integration Testing and Quality Gates
This phase ensures the entire system works end-to-end and meets quality standards.

- [x] Task: Docker Integration Tests for Sync
    - [ ] Update `tests/docker_integration_test.rs` to spin up multiple AdGuard Home containers (Master and Replica).
    - [ ] Perform a sync and verify the Replica's configuration matches the Master's according to the selected mode.
- [x] Task: Quality Gate Verification
    - [ ] Run `task test:ci` to ensure all tests pass and coverage is >80%.
    - [ ] Run `task lint` and `cargo clippy`.
    - [ ] Verify documentation is updated for the new configuration and tool.
- [x] Task: Conductor - User Manual Verification 'Phase 4: Integration Testing and Quality Gates' (Protocol in workflow.md)
