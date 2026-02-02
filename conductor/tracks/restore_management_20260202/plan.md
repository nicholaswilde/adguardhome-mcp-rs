# Restore Management Plan

## Phases

### Phase 1: Implementation
- [ ] Define `restore_backup` tool in `src/main.rs`.
- [ ] Implement `restore_backup` method in `AdGuardClient` (`src/adguard.rs`).
    - [ ] Read file from disk.
    - [ ] Post to `/control/restore`.
- [ ] Add tests in `src/adguard.rs`.

### Phase 2: Verification
- [ ] Verify restoration process.
- [ ] Test with `cargo test`.
