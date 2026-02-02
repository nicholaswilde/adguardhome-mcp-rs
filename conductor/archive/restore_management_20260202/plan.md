# Restore Management Plan

## Phases

### Phase 1: Implementation
- [x] Define `restore_backup` tool in `src/main.rs`.
- [x] Implement `restore_backup` method in `AdGuardClient` (`src/adguard.rs`).
    - [x] Read file from disk.
    - [x] Post to `/control/restore`.
- [x] Add tests in `src/adguard.rs`.

### Phase 2: Verification
- [x] Verify restoration process.
- [x] Test with `cargo test`.
