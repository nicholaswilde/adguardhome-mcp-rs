# Specification: MCP Best Practices & Token Optimization

## Overview
The goal of this track is to significantly reduce the token consumption of the AdGuard Home MCP server. This will be achieved by consolidating the existing granular toolset into fewer, more powerful unified tools using an `action` pattern and by optimizing tool descriptions and input schemas for conciseness and clarity.

## Functional Requirements

### 1. Tool Consolidation
Existing tools will be merged into the following unified tools:

#### `manage_system` (Consolidated from `system.rs`)
- **Actions:** `get_status`, `get_stats`, `clear_stats`, `get_query_log`, `clear_query_log`, `get_query_log_config`, `set_query_log_config`, `get_version_info`, `update_adguard_home`, `create_backup`, `restore_backup`, `restart_service`.
- **Purpose:** Centralize all system-level monitoring and maintenance tasks.

#### `manage_dns` (Consolidated from `dns.rs`)
- **Actions:** `list_rewrites`, `add_rewrite`, `remove_rewrite`, `get_config`, `set_config`, `clear_cache`.
- **Purpose:** Manage DNS-specific settings and rewrites.

#### `manage_protection` (Consolidated from `protection.rs`)
- **Actions:** `get_config`, `set_config`, `toggle_feature` (global, safe search, safe browsing, parental), `get_tls_config`, `set_tls_config`.
- **Purpose:** Centralize protection toggles and security configuration (including TLS).

#### `manage_filtering` (Consolidated from `filtering.rs`)
- **Actions:** `list_filters`, `add_filter`, `remove_filter`, `update_filter`, `toggle_filter`, `list_custom_rules`, `set_custom_rules`, `add_custom_rule`, `remove_custom_rule`, `list_blocked_services`, `toggle_blocked_service`, `check_host`.
- **Purpose:** Comprehensive management of filtering lists, custom rules, and blocked services.

#### `manage_clients` (Consolidated from `clients.rs`)
- **Actions:** `list_clients`, `get_client_info`, `add_client`, `update_client`, `delete_client`, `get_activity_report`, `get_access_list`, `update_access_list`, `list_dhcp_leases`, `add_static_lease`, `remove_static_lease`.
- **Purpose:** Centralize client management, DHCP, and access control.

### 2. Description & Schema Optimization
- Rewrite all tool and parameter descriptions to be as brief as possible while remaining fully descriptive for the AI.
- Use concise JSON schema definitions, avoiding redundant fields or verbose nesting where possible.

### 3. Deprecation & Cleanup
- The original granular tools will be removed from the `ToolRegistry` to ensure only the optimized unified tools are exposed to the AI.

## Non-Functional Requirements
- **Token Efficiency:** Measurable reduction in the number of tokens required for the `list_tools` response.
- **Maintainability:** The unified tool handlers should delegate logic to shared functions to keep the code clean.
- **Performance:** No regression in API call performance or response times.

## Acceptance Criteria
- [ ] Number of registered tools is reduced from ~40 to exactly 5 unified tools (plus `manage_tools` if in lazy mode).
- [ ] All original functionality is verified to work correctly via the new unified tools.
- [ ] **Unit Tests:** All unit tests in `src/tools/tests.rs` are refactored to validate the new unified tool structures and actions.
- [ ] **Integration Tests:** `tests/docker_integration_test.rs` is completely updated to use the new unified tools, ensuring end-to-end functionality against a real AdGuard Home instance is preserved.
- [ ] `task test:ci` passes successfully.

## Out of Scope
- Implementing new AdGuard Home API features.
- Modifying the underlying `AdGuardClient` logic.
