# Specification - Restore System Tools (restore_system_tools_20260218)

## Overview
This track involves investigating and restoring the system tools (`backup`, `update`, `restart`) that were temporarily disabled or broken due to API changes in AdGuard Home v0.107+. The goal is to identify the correct endpoints or alternative methods to provide this functionality through the MCP server.

## Functional Requirements
- **Backup & Restore Restoration:**
    - Identify the correct API endpoints for creating and restoring backups in v0.107+.
    - Update `AdGuardClient::create_backup` and `AdGuardClient::restore_backup` with the correct logic.
- **Service Update Fix:**
    - Investigate why `POST /control/update` returns a 400 Bad Request and fix the implementation.
- **Service Restart Restoration:**
    - Find the correct method to trigger a service restart via the API, or implement a viable alternative if the direct endpoint was removed.
- **Test Re-enablement:**
    - Uncomment and update the integration tests in `tests/system_tests.rs` to verify the restored functionality.

## Non-Functional Requirements
- **Backward Compatibility:** Maintain compatibility with older AdGuard Home versions where possible, or provide informative error messages if a feature is explicitly unsupported.
- **Robustness:** Ensure that the MCP server handles API changes gracefully without crashing.

## Acceptance Criteria
- [ ] `create_backup` action in `manage_system` is functional and verified via integration tests.
- [ ] `restore_backup` action in `manage_system` is functional and verified via integration tests.
- [ ] `update_adguard_home` action in `manage_system` is functional and verified via integration tests.
- [ ] `restart_service` action in `manage_system` is functional and verified via integration tests.
- [ ] The full test suite passes consistently: `RUN_DOCKER_TESTS=true task test:integration`.

## Out of Scope
- Implementing CLI-based restart/backup methods that require shell access to the host (focus remains on the Web API).
