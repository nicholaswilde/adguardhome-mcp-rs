# Specification - Client Management Tools

## Overview
This track adds tools to the AdGuard Home MCP server to manage network clients. AI models will be able to list all known clients, audit their specific protection settings, and retrieve individual statistics.

## Functional Requirements
- **Tools Implementation:**
  - **`list_clients`**:
    - Retrieves the list of all configured and discovered clients.
    - Returns: Name, IPs, MACs, and client-specific safety settings (Safe Search, Safe Browsing, etc.).
  - **`get_client_info`**:
    - Arguments: `identifier` (String - IP, MAC, or Name).
    - Retrieves detailed configuration and usage stats for a specific client.
- **Error Handling:**
  - Standardize error responses when a client identifier cannot be resolved.

## Technical Requirements
- **API Endpoints:**
  - List: `GET /control/clients`
  - Stats: `GET /control/clients/status` (or related client status API)
- **Client Update:** Add client retrieval methods to `AdGuardClient`.
- **Server Integration:** Register tools in `main.rs`.

## Acceptance Criteria
- `list_clients` correctly identifies all network devices known to AdGuard Home.
- `get_client_info` returns accurate stats for the targeted device.
- Tools handle multiple identification formats (IP, MAC, Name) reliably.
- Integration tests verify client tools against a real container.
