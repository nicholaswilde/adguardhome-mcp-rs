# Server Operations Specification

## User Story
As an AI agent, I want to restart the AdGuard Home service to apply certain configuration changes or recover from unstable states.

## Requirements
- **Tool:** `restart_service`
- **Input:** None.
- **Output:** Success message.
- **API Endpoint:** `POST /control/restart` (or similar).
    - Note: This might cause a temporary connection loss.

## Technical Details
- Endpoint: `POST /control/restart` (Verify exact endpoint, sometimes it is implicit or requires specific payload).
