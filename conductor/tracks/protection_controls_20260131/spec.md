# Specification - Protection Control Tools

## Overview
This track adds tools to the AdGuard Home MCP server to manage global protection and specific safety features. This enables AI models to toggle network security settings instantly.

## Functional Requirements
- **Tools Implementation:**
  - **`set_protection_state`**:
    - Arguments: `enabled` (Boolean).
    - Toggles global AdGuard Home filtering.
    - Returns: The new state of global protection.
  - **`set_safe_search`**:
    - Arguments: `enabled` (Boolean).
    - Toggles enforced safe search on popular search engines.
  - **`set_safe_browsing`**:
    - Arguments: `enabled` (Boolean).
    - Toggles protection against known malicious and phishing domains.
  - **`set_parental_control`**:
    - Arguments: `enabled` (Boolean).
    - Toggles filtering of adult content.

## Technical Requirements
- **API Endpoints:**
  - Global Protection: `POST /control/protection`
  - Safe Search: `POST /control/safesearch/enable` / `POST /control/safesearch/disable`
  - Safe Browsing: `POST /control/safebrowsing/enable` / `POST /control/safebrowsing/disable`
  - Parental Control: `POST /control/parental/enable` / `POST /control/parental/disable`
- **Client Update:** Add corresponding methods to `AdGuardClient`.
- **Server Integration:** Register tools in `main.rs`.

## Acceptance Criteria
- Tools correctly update the settings in AdGuard Home.
- Changes are immediately reflected in the response and the AdGuard Home dashboard.
- Integration tests verify state changes against a real container.
