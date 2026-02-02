# Implementation Plan: Multi-Instance Support

## Phase 1: Configuration Refactoring
Update the configuration system to handle multiple instances from both files and environment variables.

- [ ] Task: Update Configuration Models
    - [ ] Define `InstanceConfig` struct in `src/config.rs` with fields: `name`, `url`, `api_key`, `username`, `password`, `no_verify_ssl`.
    - [ ] Update `Config` struct to include `instances: Vec<InstanceConfig>`.
    - [ ] **TDD:** Write unit tests in `src/config.rs` for deserializing a list of instances from TOML.
- [ ] Task: Implement Environment Variable Parsing
    - [ ] Implement logic to parse `ADGUARD__INSTANCES__<N>__<FIELD>` environment variables.
    - [ ] Ensure env-defined instances are merged/appended to file-defined instances.
    - [ ] **TDD:** Write unit tests verifying that environment variables are correctly parsed and indices are respected.
- [ ] Task: Implementation Validation
    - [ ] Implement `validate()` logic for the new configuration structure.
    - [ ] Ensure at least one instance is configured and has required fields (URL and Auth).
    - [ ] **TDD:** Write unit tests for various valid and invalid multi-instance configurations.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Configuration Refactoring' (Protocol in workflow.md)

## Phase 2: Core and MCP Logic Update
Update the internal tool logic and MCP interface to target specific instances.

- [ ] Task: Update MCP Tool Definitions
    - [ ] Modify `src/mcp.rs` to add an optional `instance` argument to all tool schemas.
    - [ ] Update tool handlers to extract the `instance` parameter.
    - [ ] **TDD:** Write unit tests for a few representative tools to ensure the `instance` argument is correctly defined in the schema.
- [ ] Task: Instance Selection Logic
    - [ ] Implement a helper to select the correct instance configuration based on index or name.
    - [ ] Default to the first instance if the argument is missing.
    - [ ] **TDD:** Write unit tests for the selection logic, covering name matches, index matches, and fallbacks.
- [ ] Task: Update Client Initialization
    - [ ] Update `src/adguard.rs` (or relevant client module) to initialize the client based on the selected `InstanceConfig`.
    - [ ] **TDD:** Write unit tests for client factory/creation using different instance configurations.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core and MCP Logic Update' (Protocol in workflow.md)

## Phase 3: Verification and Quality
Ensure system-wide compliance and performance.

- [ ] Task: Integration Testing
    - [ ] Add a new test case to `tests/docker_integration_test.rs` that uses environment variables to configure two instances (targeting the same container with different "names") and verifies that both can be addressed via the MCP tool.
- [ ] Task: Quality Gate Verification
    - [ ] Run `task test:ci` to ensure all tests pass and coverage is >80%.
    - [ ] Run `task lint` and `cargo clippy`.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Verification and Quality' (Protocol in workflow.md)
