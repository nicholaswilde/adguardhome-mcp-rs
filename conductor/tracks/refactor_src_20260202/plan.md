# Plan - Refactor src/ (refactor_src_20260202)

## Phase 1: AdGuard API Client Refactor [checkpoint: 92d8715c8bae7670595ba92aeaf49eb7e7f13ff6]
Focuses on converting the monolithic `adguard.rs` into a structured module.

- [x] Task: Create `src/adguard/` directory and initialize `mod.rs`
- [x] Task: Extract data structures and DTOs from `adguard.rs` to `src/adguard/models.rs`
- [x] Task: Move the core `AdGuard` client implementation to `src/adguard/client.rs`
- [x] Task: Update `lib.rs` and internal imports to point to the new `adguard` module structure
- [x] Task: Verify Phase 1 with existing unit tests (Update imports in tests if necessary)
- [x] Task: Conductor - User Manual Verification 'AdGuard API Client Refactor' (Protocol in workflow.md)

## Phase 2: Tools & MCP Logic Refactor [checkpoint: f5b4cd0303b68598e573e6e6a3c5b03e82f7e3e0]
Breaks down the large `tools.rs` into domain-specific files.

- [x] Task: Create `src/tools/` directory and initialize `mod.rs`
- [x] Task: Split `tools.rs` into sub-modules:
    - [x] `src/tools/filtering.rs` (Filtering, Custom Rules, Blocked Services)
    - [x] `src/tools/dns.rs` (DNS Configuration, Rewrites)
    - [x] `src/tools/protection.rs` (Protection State, Safe Search)
    - [x] `src/tools/clients.rs` (Client Management)
    - [x] `src/tools/system.rs` (Stats, Logs, Updates)
- [x] Task: Audit `src/mcp.rs` and `src/server/mcp.rs` to ensure a clear distinction between core MCP logic and the transport-specific implementations
- [x] Task: Update all internal tool registrations to use the new module paths
- [x] Task: Verify Phase 2 with unit tests and ensure "Lazy Mode" functionality is preserved
- [x] Task: Conductor - User Manual Verification 'Tools & MCP Logic Refactor' (Protocol in workflow.md)

## Phase 3: Final Consolidation & Quality Gate
Final cleanup and project-wide verification.

- [x] Task: Run project-wide quality checks:
    - [x] `cargo check`
    - [x] `task lint` (Clippy)
    - [x] `task format`
- [x] Task: Execute full test suite including Docker integration tests:
    - [x] `RUN_DOCKER_TESTS=true task test:integration`
- [x] Task: Manually verify both `stdio` and `http` transports respond correctly
- [~] Task: Conductor - User Manual Verification 'Final Consolidation & Quality Gate' (Protocol in workflow.md)
