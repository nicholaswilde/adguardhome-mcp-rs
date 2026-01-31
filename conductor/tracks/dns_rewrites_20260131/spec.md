# Specification - DNS Rewrite Management Tools

## Overview
This track adds management tools to the AdGuard Home MCP server for handling DNS rewrites. These tools allow AI models to list, add, and remove custom DNS rewrite entries, enabling dynamic network-wide domain redirections.

## Functional Requirements
- **Tools Implementation:**
  - **`list_dns_rewrites`:**
    - Retrieves the current list of DNS rewrites from AdGuard Home.
    - Returns a list of objects containing `domain`, `answer`, and any status info.
  - **`add_dns_rewrite`:**
    - Arguments: `domain` (String), `answer` (String).
    - Adds a new DNS rewrite entry to AdGuard Home.
  - **`remove_dns_rewrite`:**
    - Arguments: `domain` (String), `answer` (String).
    - Removes a specific DNS rewrite entry matching both domain and answer.
- **Error Handling:**
  - Standardize API error responses from AdGuard Home (e.g., if an entry already exists or cannot be found).

## Technical Requirements
- **API Endpoints:**
  - List: `GET /control/rewrite/list`
  - Add: `POST /control/rewrite/add`
  - Remove: `POST /control/rewrite/delete`
- **Client Update:** Add methods to `AdGuardClient` in `src/adguard.rs` for these endpoints.
- **Server Integration:** Register the new tools in `main.rs`.

## Acceptance Criteria
- `list_dns_rewrites` returns the actual entries configured in AdGuard Home.
- `add_dns_rewrite` successfully creates an entry visible in the AdGuard Home web UI.
- `remove_dns_rewrite` successfully deletes the specified entry.
- Integration tests in `tests/docker_integration_test.rs` verify all three tools against a real container.
