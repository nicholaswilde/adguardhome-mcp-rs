# Implementation Plan: Multi-Instance Support

## Phase 1: Configuration Refactoring [checkpoint: c7e0b01c771e60dad6c408f9be68cf66900966f5]
Update the configuration system to handle multiple instances from both files and environment variables.

- [x] Task: Update Configuration Models
    - [x] Define `InstanceConfig` struct in `src/config.rs` with fields: `name`, `url`, `api_key`, `username`, `password`, `no_verify_ssl`.
    - [x] Update `Config` struct to include `instances: Vec<InstanceConfig>`.
    - [x] **TDD:** Write unit tests in `src/config.rs` for deserializing a list of instances from TOML.
- [x] Task: Implement Environment Variable Parsing
    - [x] Implement logic to parse `ADGUARD__INSTANCES__<N>__<FIELD>` environment variables.
    - [x] Ensure env-defined instances are merged/appended to file-defined instances.
    - [x] **TDD:** Write unit tests verifying that environment variables are correctly parsed and indices are respected.
- [x] Task: Implementation Validation
    - [x] Implement `validate()` logic for the new configuration structure.
    - [x] Ensure at least one instance is configured and has required fields (URL and Auth).
    - [x] **TDD:** Write unit tests for various valid and invalid multi-instance configurations.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Configuration Refactoring' (Protocol in workflow.md)

## Phase 2: Core and MCP Logic Update [checkpoint: 6ee77f61494142d32601751ef720a7f8a65010e4]
Update the internal tool logic and MCP interface to target specific instances.

- [x] Task: Update MCP Tool Definitions
    - [x] Modify `src/mcp.rs` to add an optional `instance` argument to all tool schemas.
    - [x] Update tool handlers to extract the `instance` parameter.
    - [x] **TDD:** Write unit tests for a few representative tools to ensure the `instance` argument is correctly defined in the schema.
- [x] Task: Instance Selection Logic
    - [x] Implement a helper to select the correct instance configuration based on index or name.
    - [x] Default to the first instance if the argument is missing.
    - [x] **TDD:** Write unit tests for the selection logic, covering name matches, index matches, and fallbacks.
- [x] Task: Update Client Initialization
    - [x] Update `src/adguard.rs` (or relevant client module) to initialize the client based on the selected `InstanceConfig`.
    - [x] **TDD:** Write unit tests for client factory/creation using different instance configurations.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Core and MCP Logic Update' (Protocol in workflow.md)

## Phase 3: Verification and Quality
Ensure system-wide compliance and performance.

- [x] Task: Integration Testing
    - [x] Add a new test case to `tests/docker_integration_test.rs` that uses environment variables to configure two instances (targeting the same container with different "names") and verifies that both can be addressed via the MCP tool.
- [x] Task: Quality Gate Verification
    - [x] Run `task test:ci` to ensure all tests pass and coverage is >80%.
    - [x] Run `task lint` and `cargo clippy`.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Verification and Quality' (Protocol in workflow.md)
