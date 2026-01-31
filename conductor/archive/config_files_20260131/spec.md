# Specification - Configuration File Support

## Overview
This track implements robust configuration management for the AdGuard Home MCP server, mirroring the multi-source approach used in `qbittorrent-mcp-rs`. It allows users to define settings via configuration files (TOML, YAML, JSON), environment variables, and CLI arguments.

## Functional Requirements
- **Configuration Formats:** Support for TOML, YAML, and JSON files.
- **Search Paths:**
  1. Explicit path provided via CLI (`--config` or `-c`).
  2. Local `config.{toml,yaml,json}` in the current working directory.
  3. System-standard configuration directory (e.g., `~/.config/adguardhome-mcp-rs/config.{toml,yaml,json}`).
- **Configuration Fields:**
  - `adguard_url`: The base URL of the AdGuard Home instance.
  - `adguard_username`: Admin username.
  - `adguard_password`: Admin password.
  - `mcp_transport`: Transport mode (`stdio` or `http`).
  - `lazy_mode`: Boolean to enable/disable token-optimized lazy loading.
  - `http_port`: Port for HTTP transport mode.
  - `log_level`: Severity level for logging.
- **Precedence Order (Highest to Lowest):**
  1. CLI Overrides
  2. Environment Variables (Prefix: `ADGUARD_`)
  3. Configuration File
  4. Default Values

## Technical Requirements
- **Library:** Use the `config` crate for merging sources.
- **CLI Parsing:** Use `clap` for command-line arguments and environment variable mapping.
- **Format Support:** Enable `toml`, `yaml`, and `json` features for the `config` crate in `Cargo.toml`.

## Acceptance Criteria
- Server starts correctly using only a `config.toml` file.
- Environment variables override values in the config file.
- CLI arguments override both config file and environment variables.
- Clear error messages when required fields (like `adguard_url`) are missing from all sources.
