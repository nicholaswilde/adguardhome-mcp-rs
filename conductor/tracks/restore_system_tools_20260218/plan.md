# Plan - Restore System Tools (restore_system_tools_20260218)

## Phase 1: Research & Discovery
Investigate the AdGuard Home v0.107+ API to find the missing or changed endpoints.

- [ ] Task: Research v0.107+ API for backup/restore endpoints
    - [ ] Check official OpenAPI spec if available
    - [ ] Use `google_web_search` or `brave_web_search` for recent community findings
- [ ] Task: Research v0.107+ API for service restart/update requirements
    - [ ] Determine required payloads for `POST /control/update` to avoid 400 errors
    - [ ] Look for new restart endpoints or sequences
- [ ] Task: Conductor - User Manual Verification 'Research & Discovery' (Protocol in workflow.md)

## Phase 2: Implementation & Fixes
Apply the findings to the codebase and update unit tests.

- [ ] Task: Update `AdGuardClient` with new endpoints/logic
    - [ ] Fix `create_backup` and `restore_backup`
    - [ ] Fix `update_adguard_home`
    - [ ] Fix/Implement `restart_service`
- [ ] Task: Update Unit Tests in `src/adguard/tests.rs` and `src/tools/tests.rs`
    - [ ] Mock the new endpoints and verify `AdGuardClient` calls them correctly
- [ ] Task: Conductor - User Manual Verification 'Implementation & Fixes' (Protocol in workflow.md)

## Phase 3: Integration & Validation
Re-enable integration tests and perform final verification.

- [ ] Task: Re-enable integration tests in `tests/system_tests.rs`
    - [ ] Uncomment `create_backup`, `restore_backup`, `update_adguard_home`, and `restart_service` tests
- [ ] Task: Verify functionality against real Docker container
    - [ ] `RUN_DOCKER_TESTS=true task test:integration`
- [ ] Task: Final Quality Audit
    - [ ] `task lint`
    - [ ] `task format`
- [ ] Task: Conductor - User Manual Verification 'Integration & Validation' (Protocol in workflow.md)
