# Plan - Restore System Tools (restore_system_tools_20260218)

## Phase 1: Research & Discovery
Investigate the AdGuard Home v0.107+ API to find the missing or changed endpoints.

- [x] Task: Research v0.107+ API for backup/restore endpoints
    - [x] Check official OpenAPI spec if available
    - [x] Use `google_web_search` or `brave_web_search` for recent community findings
- [x] Task: Research v0.107+ API for service restart/update requirements
    - [x] Determine required payloads for `POST /control/update` to avoid 400 errors
    - [x] Look for new restart endpoints or sequences
- [x] Task: Conductor - User Manual Verification 'Research & Discovery' (Protocol in workflow.md)

## Phase 2: Implementation & Fixes [checkpoint: d3dd3ef]
Apply the findings to the codebase and update unit tests.

- [x] Task: Update `AdGuardClient` with new endpoints/logic
    - [x] Fix `create_backup` and `restore_backup`
    - [x] Fix `update_adguard_home`
    - [x] Fix/Implement `restart_service`
- [x] Task: Update Unit Tests in `src/adguard/tests.rs` and `src/tools/tests.rs`
    - [x] Mock the new endpoints and verify `AdGuardClient` calls them correctly
- [x] Task: Conductor - User Manual Verification 'Implementation & Fixes' (Protocol in workflow.md)

## Phase 3: Integration & Validation
Re-enable integration tests and perform final verification.

- [x] Task: Re-enable integration tests in `tests/system_tests.rs`
    - [x] Uncomment `create_backup`, `restore_backup`, `update_adguard_home`, and `restart_service` tests
- [x] Task: Verify functionality against real Docker container
    - [x] `RUN_DOCKER_TESTS=true task test:integration`
- [x] Task: Final Quality Audit
    - [x] `task lint`
    - [x] `task format`
- [x] Task: Conductor - User Manual Verification 'Integration & Validation' (Protocol in workflow.md)
