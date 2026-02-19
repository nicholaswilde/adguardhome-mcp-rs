# Implementation Plan: Backup & Restore Enhancements

**Track ID:** `backup_restore_enhancements_20260218`

## Phase 1: Configuration Expansion [checkpoint: 7c6a909]
Enhance `AdGuardClient` and `SyncState` to include the missing configuration modules.

- [x] Task: Update `AdGuardClient` and `models.rs` for new modules
    - [x] Implement `get_dhcp_config`, `set_dhcp_config`, `get_profile_info`, and `set_profile_info`.
    - [x] **TDD:** Write unit tests for the new client methods in `src/adguard/tests.rs`.
- [x] Task: Update `SyncState` struct and logic in `src/sync.rs`
    - [x] Add fields for `dhcp`, `tls_status` (already has some), and `profile_info`.
    - [x] Update `fetch` and `push_to_replica` to include these modules.
    - [x] **TDD:** Write unit tests in `src/sync.rs` for fetching and pushing these new modules.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Configuration Expansion' (Protocol in workflow.md)

## Phase 2: Metadata and Versioning
Add metadata to the backup JSON and implement version safety checks.

- [x] Task: Update `SyncState` and `create_backup` logic
    - [x] Add `metadata` struct to `SyncState`: `version`, `timestamp`, `description`.
    - [x] Update `manage_system` tool's `create_backup` action to accept an optional `description`.
    - [x] **TDD:** Write unit tests in `src/tools/tests.rs` for backups with metadata.
- [x] Task: Implement version and instance safety checks
    - [x] Ensure `restore_backup` checks for the AdGuard Home version in the backup.
    - [x] **TDD:** Write unit tests for warning/preventing restoration from a drastically different version.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Metadata and Versioning' (Protocol in workflow.md)

## Phase 3: Dry Run and Comparison
Implement the ability to preview changes before a full restoration.

- [ ] Task: Implement `restore_backup_diff` tool action
    - [ ] Create logic in `SyncState` to diff a JSON snapshot against a live client.
    - [ ] Format the diff into a human-readable summary for the MCP output.
    - [ ] **TDD:** Write unit tests for diffing various configuration scenarios.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Dry Run and Comparison' (Protocol in workflow.md)

## Phase 4: Reliable Service Management
Refine the restart strategy and improve error tracking.

- [ ] Task: Enhance `restart_service` tool action
    - [ ] Update `manage_system` to support an optional `force` (hard) vs. `soft` (reload) restart parameter.
    - [ ] **TDD:** Write unit tests for both restart types.
- [ ] Task: Improve Atomic Restoration and Error Reporting
    - [ ] Update `push_to_replica` to return a list of successfully applied vs. failed modules.
    - [ ] Update `restore_backup` to output this detailed report.
    - [ ] **TDD:** Write unit tests for partial restoration failures.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Reliable Service Management' (Protocol in workflow.md)

## Phase 5: Verification and Quality
System-wide integration testing and final audit.

- [ ] Task: Integration Testing
    - [ ] Add integration tests in `tests/system_tests.rs` for full backup/restore of all modules.
    - [ ] Verify metadata and diffing functionality against a real Docker instance.
- [ ] Task: Final Quality Audit
    - [ ] `task lint`, `task format`, `task test:ci`.
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Verification and Quality' (Protocol in workflow.md)
