# Specification: Multi-Instance Support

## Overview
Implement the ability for the AdGuard Home MCP server to manage and communicate with multiple AdGuard Home instances. Configuration for these instances will be supported via both a `config.toml` file and specifically formatted environment variables, following the pattern established in the `proxmox-mcp-rs` project.

## Functional Requirements
### 1. Configuration Support
- **Multi-Instance Definition:** Support a list of `instances` in the configuration.
- **Instance Fields:** Each instance should support the following fields:
    - `name`: An optional friendly alias (e.g., "primary", "homelab").
    - `url`: The base URL of the AdGuard Home instance.
    - `api_key`: The API key used for authentication.
    - `username`: (Optional) For basic auth if API key is not used.
    - `password`: (Optional) For basic auth if API key is not used.
    - `no_verify_ssl`: (Optional) Boolean to skip SSL certificate verification.

### 2. Environment Variable Parsing
- Support environment variables starting with `ADGUARD__INSTANCES__<index>__<field>`.
- Examples:
    - `ADGUARD__INSTANCES__0__NAME=primary`
    - `ADGUARD__INSTANCES__0__URL=http://192.168.1.1`
    - `ADGUARD__INSTANCES__0__API_KEY=mysecretkey`

### 3. MCP Tool Integration
- All MCP tools should be updated to include an optional `instance` argument.
- The `instance` argument can be either the friendly `name` or the numerical index (as a string).
- If the `instance` argument is omitted, the tool should default to the first instance in the configuration (index 0).

## Non-Functional Requirements
- **Consistency:** Follow the architectural patterns for configuration and tool registration found in `src/config.rs` and `src/mcp.rs`.
- **Validation:** Ensure each instance configuration is validated (e.g., URL is present, authentication method provided).

## Acceptance Criteria
- Multiple AdGuard Home instances can be successfully configured via `config.toml` and environment variables.
- MCP tools correctly target the specified instance based on the `instance` argument.
- Tools default to the primary (first) instance when no argument is provided.
- An error is returned if a specified instance is not found or is misconfigured.

## Out of Scope
- Synchronization logic between instances (covered by the `instance_sync` track).
- Management of the MCP server itself via MCP tools.
