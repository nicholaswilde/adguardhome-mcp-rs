# Specification - SSL Verification Configuration

## Overview
This track adds the ability to optionally disable SSL certificate verification when connecting to AdGuard Home. This is particularly useful for users running AdGuard Home with self-signed certificates or in local environments where strict SSL validation is not desired.

## Functional Requirements
- **SSL Verification Option:**
  - Add `no_verify_ssl` (boolean) to the server configuration.
  - Default value: `true` (SSL verification is disabled by default).
- **Configuration Support:**
  - **CLI:** Add `--no-verify-ssl` flag.
  - **Environment:** Support `ADGUARD_NO_VERIFY_SSL`.
  - **Config File:** Support `no_verify_ssl` key in TOML/YAML/JSON.
- **Client Integration:**
  - Update `AdGuardClient` to configure the `reqwest::Client` with `.danger_accept_invalid_certs(no_verify_ssl)`.
  - This setting should only be active when an `https` URL is used.

## Technical Requirements
- Update `AppConfig` in `src/config.rs` to include the new field and mapping.
- Modify `AdGuardClient::new` in `src/adguard.rs` to accept and apply the SSL setting.
- Ensure the `reqwest` client is initialized correctly based on the configuration.

## Acceptance Criteria
- Running with an HTTPS URL and valid certificates works normally.
- Running with an HTTPS URL and self-signed certificates works when `no_verify_ssl` is `true`.
- The CLI flag `--no-verify-ssl` correctly overrides other configuration sources.
- Unit tests verify that the configuration is correctly parsed and passed to the client.
