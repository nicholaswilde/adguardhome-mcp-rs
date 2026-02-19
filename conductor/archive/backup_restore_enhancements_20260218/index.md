# Track Index: Backup & Restore Enhancements

**Track ID:** `backup_restore_enhancements_20260218`
**Status:** `ACTIVE`

## Overview
This track aims to enhance the backup, restoration, and restart functionality in `adguardhome-mcp-rs` to provide a more robust and user-friendly experience, including full configuration coverage, metadata versioning, dry-run comparisons, and more reliable service management.

## Key Files
- [Specification](./spec.md)
- [Implementation Plan](./plan.md)

## Success Criteria
- [ ] `SyncState` includes TLS, DHCP, and Profile Info.
- [ ] Backups include version metadata and timestamps.
- [ ] New `restore_backup_diff` tool provides a preview of changes.
- [ ] Service restart logic handles both soft and hard restarts where applicable.
- [ ] Restoration process provides detailed error tracking and reporting.
