# Specification: Backup & Restore Enhancements

**Track ID:** `backup_restore_enhancements_20260218`

## Problem Statement
The current backup and restore functionality in `adguardhome-mcp-rs` is a solid foundation but lacks completeness in configuration coverage, metadata for version safety, preview capabilities before restoration, and robust error tracking.

## Goals
1.  **Full Configuration Coverage:** Expand `SyncState` to include TLS, DHCP, and global Profile Info.
2.  **Versioning and Metadata:** Embed AdGuard Home version, timestamp, and optional description into backup files.
3.  **Dry Run (Preview):** Implement a tool to diff a backup file against the current instance state before applying changes.
4.  **Tiered Restart Strategy:** Distinguish between soft reloads and full service restarts.
5.  **Robust Error Handling:** Track and report the success of each module during the restoration process.

## Functional Requirements
- `AdGuardClient` must support retrieving/setting TLS, DHCP, and Profile Info.
- `SyncState` struct in `src/sync.rs` must be updated to include new modules.
- `create_backup` tool must accept an optional `description` parameter and embed it in the JSON.
- New `restore_backup_diff` tool in `manage_system` to return a comparison summary.
- Improved error reporting in `push_to_replica` to indicate exactly which modules failed.

## Technical Constraints
- Must remain compatible with AdGuard Home v0.107+.
- Backup files should be backward compatible with existing JSON structure where possible.
- Adhere to the current TDD-based development workflow.
