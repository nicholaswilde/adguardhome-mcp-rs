# Plan - Refactor src/ (refactor_src_20260202)

## Phase 1: AdGuard API Client Refactor
Focuses on converting the monolithic `adguard.rs` into a structured module.

- [ ] Task: Create `src/adguard/` directory and initialize `mod.rs`
- [ ] Task: Extract data structures and DTOs from `adguard.rs` to `src/adguard/models.rs`
- [ ] Task: Move the core `AdGuard` client implementation to `src/adguard/client.rs`
- [ ] Task: Update `lib.rs` and internal imports to point to the new `adguard` module structure
- [ ] Task: Verify Phase 1 with existing unit tests (Update imports in tests if necessary)
- [ ] Task: Conductor - User Manual Verification 'AdGuard API Client Refactor' (Protocol in workflow.md)

## Phase 2: Tools & MCP Logic Refactor
Breaks down the large `tools.rs` into domain-specific files.

- [ ] Task: Create `src/tools/` directory and initialize `mod.rs`
- [ ] Task: Split `tools.rs` into sub-modules:
    - [ ] `src/tools/filtering.rs` (Filtering, Custom Rules, Blocked Services)
    - [ ] `src/tools/dns.rs` (DNS Configuration, Rewrites)
    - [ ] `src/tools/protection.rs` (Protection State, Safe Search)
    - [ ] `src/tools/clients.rs` (Client Management)
    - [ ] `src/tools/system.rs` (Stats, Logs, Updates)
- [ ] Task: Audit `src/mcp.rs` and `src/server/mcp.rs` to ensure a clear distinction between core MCP logic and the transport-specific implementations
- [ ] Task: Update all internal tool registrations to use the new module paths
- [ ] Task: Verify Phase 2 with unit tests and ensure "Lazy Mode" functionality is preserved
- [ ] Task: Conductor - User Manual Verification 'Tools & MCP Logic Refactor' (Protocol in workflow.md)

## Phase 3: Final Consolidation & Quality Gate
Final cleanup and project-wide verification.

- [ ] Task: Run project-wide quality checks:
    - [ ] `cargo check`
    - [ ] `task lint` (Clippy)
    - [ ] `task format`
- [ ] Task: Execute full test suite including Docker integration tests:
    - [ ] `RUN_DOCKER_TESTS=true task test:integration`
- [ ] Task: Manually verify both `stdio` and `http` transports respond correctly
- [ ] Task: Conductor - User Manual Verification 'Final Consolidation & Quality Gate' (Protocol in workflow.md)
