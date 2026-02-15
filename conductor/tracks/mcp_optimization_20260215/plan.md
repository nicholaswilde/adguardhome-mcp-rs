# Implementation Plan: MCP Best Practices & Token Optimization

## Phase 1: Foundation & System Tool Consolidation
Focus on refactoring the `ToolRegistry` to support unified tools and consolidating the `system` tools.

- [ ] Task: Refactor `src/tools/system.rs` to implement the `manage_system` unified tool.
    - [ ] Create a consolidated handler for all system-related actions.
    - [ ] Implement the `action` based dispatch logic.
    - [ ] Optimize descriptions and JSON schemas for `manage_system`.
- [ ] Task: Update unit tests in `src/tools/tests.rs` for `manage_system`.
- [ ] Task: Update `tests/docker_integration_test.rs` to use `manage_system`.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Foundation & System Tool Consolidation' (Protocol in workflow.md)

## Phase 2: DNS & Protection Tool Consolidation
Consolidate DNS and Protection related tools into unified structures.

- [ ] Task: Refactor `src/tools/dns.rs` into the `manage_dns` unified tool.
    - [ ] Combine rewrites, config, and cache management into one tool.
    - [ ] Optimize descriptions and schemas.
- [ ] Task: Refactor `src/tools/protection.rs` into the `manage_protection` unified tool.
    - [ ] Combine global/feature toggles and TLS configuration.
    - [ ] Optimize descriptions and schemas.
- [ ] Task: Update unit tests in `src/tools/tests.rs` for DNS and Protection.
- [ ] Task: Update `tests/docker_integration_test.rs` for DNS and Protection.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: DNS & Protection Tool Consolidation' (Protocol in workflow.md)

## Phase 3: Filtering & Client Tool Consolidation
Consolidate Filtering and Client management tools.

- [ ] Task: Refactor `src/tools/filtering.rs` into the `manage_filtering` unified tool.
    - [ ] Combine filters, custom rules, blocked services, and host checking.
    - [ ] Optimize descriptions and schemas.
- [ ] Task: Refactor `src/tools/clients.rs` into the `manage_clients` unified tool.
    - [ ] Combine clients, DHCP, and access control.
    - [ ] Optimize descriptions and schemas.
- [ ] Task: Update unit tests in `src/tools/tests.rs` for Filtering and Clients.
- [ ] Task: Update `tests/docker_integration_test.rs` for Filtering and Clients.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Filtering & Client Tool Consolidation' (Protocol in workflow.md)

## Phase 4: Final Cleanup & Optimization
Remove deprecated tools and perform a final token usage audit.

- [ ] Task: Remove old granular tool registrations from `src/lib.rs`.
- [ ] Task: Verify final tool count and audit token usage in `list_tools`.
- [ ] Task: Run final `task test:ci` to ensure project-wide stability.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Cleanup & Optimization' (Protocol in workflow.md)
