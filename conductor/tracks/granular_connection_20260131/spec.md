# Specification - Granular AdGuard Connection Settings

## Overview
This track refactors the connection settings for AdGuard Home. Instead of a single `adguard_url` string, we will split the connection details into `adguard_host` and `adguard_port`. This improves configuration clarity and aligns with modular configuration standards.

## Functional Requirements
- **Configuration Update:**
  - Remove `adguard_url` from `AppConfig`.
  - Add `adguard_host` (String, required).
  - Add `adguard_port` (u16, defaults to `3000`).
- **Connection Logic:**
  - The server will construct the full API URL using the format: `http://{adguard_host}:{adguard_port}`.
  - The protocol is assumed to be `http` for this phase.
- **Source Support:**
  - **CLI:** Support `--adguard-host` and `--adguard-port`.
  - **Environment:** Support `ADGUARD_HOST` and `ADGUARD_PORT`.
  - **Config File:** Support `adguard_host` and `adguard_port` keys.
- **Backward Compatibility:** No backward compatibility for `adguard_url` is required; this is a breaking change.

## Technical Requirements
- Update `src/config.rs` to reflect the new struct fields and `clap`/`config` mapping.
- Update `src/adguard.rs` (`AdGuardClient`) to store and use host/port instead of a pre-formatted URL.
- Update `tests/docker_integration_test.rs` and other tests to provide host and port separately.

## Acceptance Criteria
- Running the binary with `--adguard-host localhost --adguard-port 3000` correctly initializes the client.
- Environment variables `ADGUARD_HOST` and `ADGUARD_PORT` are correctly merged.
- The `config.toml` correctly supports the new keys.
- `task test:ci` passes with all tests updated to the new configuration format.
