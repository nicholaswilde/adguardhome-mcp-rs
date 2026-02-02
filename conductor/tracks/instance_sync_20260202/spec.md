# Specification: AdGuard Home Instance Sync

## Overview
Implement a synchronization feature that allows the AdGuard Home MCP server to act as a controller, pushing configuration from a "master" AdGuard Home instance to one or more "replica" instances. This functionality is inspired by tools like `adguardhome-sync`.

## Functional Requirements
### 1. Sync Model
- **Push Model:** The MCP server will read configuration from a source (master) instance and apply it to a list of destination (replica) instances.

### 2. Synchronization Scope
The sync process should cover the following modules:
- **Filtering Lists:** Blocklists and Allowlists.
- **Custom Rules:** User-defined filtering rules.
- **Client Management:** Individual client configurations and settings.
- **DNS Settings:** Upstream servers, bootstrap DNS, and cache configurations.
- **Blocked Services:** Global settings for platform-level service blocking.

### 3. Replica Management
- Replica instances (URLs and API keys) must be definable via:
    - `config.toml` (static list).
    - Environment variables (e.g., `ADGUARD_REPLICAS`).
    - MCP tool arguments (for ad-hoc syncs).

### 4. Trigger Mechanisms
- **Manual Trigger:** An MCP tool (e.g., `sync_instances`) to initiate sync on demand.
- **Automated Trigger:** A background task that runs at a configurable interval (defined in `config.toml`).

### 5. Synchronization Logic
- **Sync Modes:** Support both "Full Overwrite" (replica becomes identical to master) and "Additive Merge" (master config is added to replica without deleting unique replica entries).
- **Configurability:** The sync mode must be selectable via tool arguments or configuration settings.

## Non-Functional Requirements
- **Robustness:** Errors during sync with one replica should not prevent sync with others.
- **Observability:** Detailed logging of sync actions, successes, and failures.
- **Performance:** Efficient API interaction to minimize load on AdGuard Home instances.

## Acceptance Criteria
- A new MCP tool `sync_instances` is available and successfully pushes Master configuration to specified Replicas.
- Configuration for all scoped modules is correctly applied to Replicas based on the selected sync mode.
- Background sync successfully executes at the defined interval.
- Authentication works correctly using API keys for all instances.

## Out of Scope
- Synchronization of historical query logs or statistics.
- Bidirectional conflict resolution (Master always wins).
- Syncing system-level settings like network interface bindings or SSL certificates (unless specifically added later).
