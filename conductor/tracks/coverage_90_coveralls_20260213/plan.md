# Implementation Plan: Increase Code Coverage to >90% with Coveralls

This plan outlines the steps to reach >90% code coverage and integrate manual reporting to Coveralls.io using `cargo-llvm-cov`.

## Phase 1: Coverage Tooling and Baseline [checkpoint: 278cfa8]
Set up the necessary tools and establish the current coverage baseline.

- [x] Task: Configure coverage tooling in `Taskfile.yml`.
    - [x] Add `coverage` task to run tests with `cargo-llvm-cov` and show summary.
    - [x] Add `coverage:report` task to generate `lcov.info` and HTML reports.
    - [x] Add `coverage:upload` task to send reports to Coveralls using the `coveralls` CLI.
- [x] Task: Identify and document current coverage gaps.
    - [x] Run the initial coverage report and record the baseline (24.83%).
    - [x] Map out specific functions or modules in `client.rs` and `server/` with low coverage.
        - `server/http.rs`: 0%
        - `server/mcp.rs`: 0%
        - `tools/*.rs`: 0%
        - `main.rs`: 0%
        - `adguard/client.rs`: 72.11%
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Coverage Tooling and Baseline' (Protocol in workflow.md)

## Phase 2: Core Client and API Logic Coverage
Target the AdGuard Home API client and error handling.

- [ ] Task: Expand unit tests for `src/adguard/client.rs`.
    - [ ] Write failing tests for unhandled error responses from AdGuard Home.
    - [ ] Implement mocks in `src/adguard/tests.rs` to cover these error paths.
- [ ] Task: Increase coverage for `src/adguard/models.rs`.
    - [ ] Add tests for complex serialization/deserialization logic if present.
- [ ] Task: Verify `src/adguard/` coverage exceeds 90%.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Core Client and API Logic Coverage' (Protocol in workflow.md)

## Phase 3: Server, Tools, and Startup Coverage
Target the transport layer, MCP tools, and server initialization.

- [ ] Task: Expand tests for `src/server/http.rs` and `src/server/mcp.rs`.
    - [ ] Write failing tests for SSE session timeouts or malformed requests.
    - [ ] Implement logic to handle these scenarios if missing, or ensure existing logic is exercised.
- [ ] Task: Ensure all `src/tools/*.rs` modules reach 90% coverage.
    - [ ] Audit each tool file (DNS, Filtering, etc.) for missing edge cases.
    - [ ] Add specific unit or integration tests for missing tool handlers.
- [ ] Task: Add tests for `src/config.rs` and `src/main.rs`.
    - [ ] Test configuration loading with missing or invalid environment variables.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Server, Tools, and Startup Coverage' (Protocol in workflow.md)

## Phase 4: Final Verification and Coveralls Integration
Verify the final coverage goal and perform the first manual upload.

- [ ] Task: Run final coverage audit.
    - [ ] Verify total line coverage is >= 90% across the entire project.
- [ ] Task: Update `README.md` with coverage instructions.
    - [ ] Document how to run `task coverage` and `task coverage:upload`.
- [ ] Task: Perform manual upload to Coveralls.
    - [ ] Verify the report appears correctly on the Coveralls dashboard.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Verification and Coveralls Integration' (Protocol in workflow.md)
