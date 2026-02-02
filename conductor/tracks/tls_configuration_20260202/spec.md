# TLS Configuration Specification

## User Story
As an AI agent, I want to configure SSL/TLS certificates and encryption settings to secure the AdGuard Home instance.

## Requirements
- **Tool:** `get_tls_config`
- **Tool:** `set_tls_config`
- **Input:**
    - `enabled`: bool
    - `server_name`: string
    - `certificate_chain`: string (content or path?)
    - `private_key`: string
- **Output:** Status or success message.
- **API Endpoint:** `/control/tls/status`, `/control/tls/configure`.

## Technical Details
- Endpoints:
    - GET `/control/tls/status`
    - POST `/control/tls/configure`
    - POST `/control/tls/validate` (optional but good practice).
