# Specification: Increase Code Coverage to >90% with Coveralls

## Overview
This track aims to increase the `adguardhome-mcp-rs` project's code coverage to above 90% and integrate reporting to Coveralls.io. It leverages the successful pattern used in the `qbittorrent-mcp-rs` project to target under-tested areas and provide a standardized upload mechanism.

## Functional Requirements
- **Comprehensive Coverage Expansion:**
    - Increase test coverage for `src/adguard/client.rs` (API client and error handling).
    - Expand testing for `src/server/http.rs` and `src/server/mcp.rs` (transport and tool dispatching).
    - Ensure all modules in `src/tools/` (DNS, Filtering, Protection, System, etc.) reach the target.
    - Cover configuration and startup logic in `src/config.rs` and `src/main.rs`.
- **Tooling Integration:**
    - Configure `cargo-llvm-cov` for source-based code coverage.
    - Generate `lcov.info` reports.
- **Coveralls.io Integration:**
    - Implement a `task coverage:upload` command to send reports to Coveralls.io.
    - Use the `coveralls` CLI for the upload process.
    - Require `COVERALLS_REPO_TOKEN` for authentication.

## Non-Functional Requirements
- **Coverage Target:** >90% total line coverage.
- **Standalone Workflow:** Coverage tasks will remain independent from the standard `task test` to prevent overhead during daily development.

## Acceptance Criteria
- [ ] Total project line coverage is at or above 90% as reported by `cargo llvm-cov`.
- [ ] All priority modules (`client.rs`, `server/`, `tools/`) meet or exceed the target.
- [ ] `Taskfile.yml` includes `coverage`, `coverage:report`, and `coverage:upload` tasks.
- [ ] `README.md` is updated with instructions for running coverage and uploading to Coveralls.
- [ ] A successful manual upload to Coveralls.io is verified.

## Out of Scope
- Automated CI upload in this specific track (though the tools will be ready).
- Hard failure of local tests on coverage drops (soft enforcement only).
