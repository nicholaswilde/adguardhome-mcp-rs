# Specification - Monitoring & Statistics Tools

## Overview
This track adds tools to the AdGuard Home MCP server for monitoring network activity and analyzing DNS queries. These tools enable AI models to retrieve aggregated statistics and search through historical query logs.

## Functional Requirements
- **Tools Implementation:**
  - **`get_stats`**:
    - Retrieves global statistics from AdGuard Home.
    - Arguments: `time_period` (Optional: "24h", "7d", "30d"; default: "24h").
    - Returns: Raw counts (Total queries, blocked, malware, etc.) and calculated percentages (e.g., % blocked).
  - **`get_query_log`**:
    - Searches the AdGuard Home DNS query log.
    - Arguments:
      - `search` (Optional: domain name filter).
      - `limit` (Optional: max entries to return; default: 50, max: 100).
      - `filter` (Optional: "all", "blocked", "allowed").
    - Returns: A list of recent query log entries.

## Technical Requirements
- **API Endpoints:**
  - Stats: `GET /control/stats`
  - Query Log: `GET /control/querylog`
- **Client Update:** Add `get_stats` and `get_query_log` methods to `AdGuardClient`.
- **Server Integration:** Register tools in `main.rs`.

## Acceptance Criteria
- `get_stats` returns valid data that matches the AdGuard Home dashboard summary.
- `get_query_log` correctly filters results based on domain search strings.
- Both tools handle API pagination/limits gracefully to avoid overloading the MCP response.
- Integration tests verify both tools against a real container.
