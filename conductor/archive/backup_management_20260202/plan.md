# Backup Management Plan

## Phases

### Phase 1: Implementation
- [x] Define `create_backup` tool in `src/main.rs`.
- [x] Implement `create_backup` method in `AdGuardClient` (`src/adguard.rs`).
    - [x] Handle binary response.
    - [x] Save to a generic `./backups/` directory or similar.
- [x] Add tests in `src/adguard.rs`.

### Phase 2: Verification
- [x] Verify backup file creation.
- [x] Test with `cargo test`.
