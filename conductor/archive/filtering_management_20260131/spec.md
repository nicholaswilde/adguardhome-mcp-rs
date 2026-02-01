# Specification - Filtering & Blocklist Management Tools

## Overview
This track adds tools to the AdGuard Home MCP server to manage filtering lists (blocklists and allowlists). AI models will be able to audit active lists, toggle them, and add new community resources to the network.

## Functional Requirements
- **Tools Implementation:**
  - **`list_filter_lists`**:
    - Retrieves the full list of configured filters from AdGuard Home.
    - Returns: Name, ID, URL, Enabled status, and usage statistics (e.g., query counts).
  - **`toggle_filter_list`**:
    - Arguments: `identifier` (String - ID or Name), `enabled` (Boolean).
    - Toggles the state of a specific list.
  - **`add_filter_list`**:
    - Arguments: `name` (String), `url` (String).
    - Adds a new filter list to the AdGuard Home configuration.
- **Error Handling:**
  - Handle cases where a list name or ID is not found.
  - Standardize API errors for invalid URLs.

## Technical Requirements
- **API Endpoints:**
  - List: `GET /control/filtering/config`
  - Toggle/Update: `POST /control/filtering/set_url` (or related update endpoint)
  - Add: `POST /control/filtering/add_url`
- **Client Update:** Add filtering management methods to `AdGuardClient`.
- **Server Integration:** Register tools in `main.rs`.

## Acceptance Criteria
- `list_filter_lists` returns all expected metadata and stats.
- Toggling a list via Name or ID successfully updates its state in AdGuard Home.
- `add_filter_list` correctly creates a new entry.
- Integration tests verify filtering management against a real container.
