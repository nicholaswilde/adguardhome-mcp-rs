# :shield: AdGuard Home MCP Server (Rust) :robot:

[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/adguardhome-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main)](https://github.com/nicholaswilde/adguardhome-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.7) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

A Rust implementation of an AdGuard Home [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to an AdGuard Home instance and exposes tools to monitor and manage filtering via the Model Context Protocol.

## :sparkles: Features

- **Multi-Transport Support:**
  - **Stdio:** Default transport for local integrations (e.g., Claude Desktop).
  - **HTTP/SSE:** Network-accessible transport for remote clients.
- **Robust Configuration:** Supports configuration via CLI arguments, environment variables, and configuration files (TOML, YAML, JSON).
- **Authentication:**
  - Connects to AdGuard Home using username/password.
  - Secures HTTP transport with Bearer Token authentication.
- **Token Optimization:** "Lazy Mode" initially exposes a minimal toolset to save AI context tokens, loading more tools only on demand.
  - **Tools:**
  - **System:**
    - `get_status`: Get AdGuard Home status, version, and protection state.
    - `get_version_info`: Get version information and check for updates.
    - `update_adguard_home`: Trigger an update of AdGuard Home.
  - **DNS Rewrites:**

    - `list_dns_rewrites`: List all DNS rewrites.
    - `add_dns_rewrite`: Add a new DNS rewrite (domain, answer).
    - `remove_dns_rewrite`: Remove a DNS rewrite (domain, answer).
  - **Protection:**
    - `get_protection_config`: Retrieve current settings for all protection features.
    - `set_protection_config`: Update configuration for Safe Search and Parental Control.
    - `set_protection_state`: Enable or disable global AdGuard Home protection.
    - `set_safe_search`: Enable or disable enforced safe search.
    - `set_safe_browsing`: Enable or disable safe browsing protection.
    - `set_parental_control`: Enable or disable parental control.
  - **Filtering:**
    - `check_filtering`: Check how a domain is filtered.
    - `list_filter_lists`: List all configured filter lists (blocklists and allowlists).
    - `toggle_filter_list`: Enable or disable a specific filter list by Name, ID, or URL.
    - `add_filter_list`: Add a new filter list to the configuration.
    - `remove_filter_list`: Remove an existing filter list.
    - `update_filter_list`: Update the name, URL, or enabled state of a filter list.
    - `list_custom_rules`: List all user-defined DNS filtering rules.
    - `set_custom_rules`: Replace all custom filtering rules.
    - `add_custom_rule`: Add a single custom filtering rule.
    - `remove_custom_rule`: Remove a single custom filtering rule.
  - **Blocked Services:**
    - `list_blocked_services`: List all available services and their current blocked status.
    - `toggle_blocked_service`: Enable or disable blocking for a specific service.
  - **Clients:**
    - `list_clients`: List all configured and discovered network clients.
    - `get_client_info`: Get detailed configuration and usage stats for a specific client.
    - `add_client`: Add a new client configuration.
    - `update_client`: Update an existing client configuration.
    - `delete_client`: Remove a client configuration.
  - **DHCP Management:**
    - `list_dhcp_leases`: List all dynamic and static DHCP leases.
    - `add_static_lease`: Add a new static DHCP lease.
    - `remove_static_lease`: Remove an existing static DHCP lease.
  - **DNS Configuration:**
    - `get_dns_config`: Retrieve current DNS settings including upstream servers and cache configuration.
    - `set_dns_config`: Update DNS settings including upstream servers and cache configuration.
    - `clear_dns_cache`: Flush the DNS cache.
  - **Analytics & Reporting:**
    - `get_top_blocked_domains`: List the most frequently blocked domains.
    - `get_client_activity_report`: Summarize recent activity and top domains for a specific client.
  - **Access Control:**
    - `get_access_list`: Get the global access control lists (allowed/disallowed clients and blocked hosts).
    - `update_access_list`: Update the global access control lists.
  - **Monitoring:**
    - `get_stats`: Get global statistics (total queries, blocked, etc.).
    - `get_query_log`: Search and filter the DNS query log.
    - `get_query_log_config`: Retrieve current DNS query logging settings.
    - `set_query_log_config`: Update DNS query logging settings.
    - `clear_stats`: Reset all statistics.
    - `clear_query_log`: Clear the DNS query log.
  - **Management (Lazy Mode only):**
    - `manage_tools`: List and enable/disable available tools.

## :package: Installation

### Homebrew

```bash
brew install nicholaswilde/tap/adguardhome-mcp-rs
```

## :hammer_and_wrench: Build

To build the project, you need a Rust toolchain installed.

```bash
cargo build --release
```

The binary will be available at `target/release/adguardhome-mcp-rs`.

## :rocket: Usage

### :keyboard: Command Line Interface

The server can be configured via CLI arguments or environment variables.

```bash
./target/release/adguardhome-mcp-rs --adguard-host "192.168.1.10" --adguard-port 8080 --adguard-username "admin" --adguard-password "yourpassword"
```

#### Available Arguments

| Argument | Environment Variable | Description | Default |
| :--- | :--- | :--- | :--- |
| `-c, --config` | - | Path to configuration file | `config.toml` |
| `--adguard-host` | `ADGUARD_HOST` | AdGuard Home instance host | (Required) |
| `--adguard-port` | `ADGUARD_PORT` | AdGuard Home instance port | `3000` |
| `--adguard-username` | `ADGUARD_USERNAME` | AdGuard Home username | - |
| `--adguard-password` | `ADGUARD_PASSWORD` | AdGuard Home password | - |
| `--transport` | `ADGUARD_MCP_TRANSPORT` | Transport mode (`stdio` or `http`) | `stdio` |
| `--http-port` | `ADGUARD_HTTP_PORT` | Port for HTTP transport | `3000` |
| `--http-token` | `ADGUARD_HTTP_AUTH_TOKEN` | Bearer token for HTTP security | - |
| `--no-verify-ssl` | `ADGUARD_NO_VERIFY_SSL` | Disable SSL certificate verification | `true` |
| `--lazy` | `ADGUARD_LAZY_MODE` | Enable token-optimized lazy loading | `false` |
| `--log-level` | `ADGUARD_LOG_LEVEL` | Log level (`info`, `debug`, etc.) | `info` |

### :file_folder: Configuration File

The server automatically looks for `config.toml`, `config.yaml`, or `config.json` in the current directory and `~/.config/adguardhome-mcp-rs/`.

Example `config.toml`:

```toml
adguard_host = "192.168.1.10"
adguard_port = 8080
adguard_username = "admin"
adguard_password = "yourpassword"
mcp_transport = "http"
http_port = 3000
http_auth_token = "your-secure-token"
lazy_mode = true
```

### :robot: Configuration Example (Claude Desktop)

Add the following to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "adguardhome": {
      "command": "/path/to/adguardhome-mcp-rs/target/release/adguardhome-mcp-rs",
      "args": [
        "--adguard-host", "192.168.1.10",
        "--adguard-port", "8080",
        "--adguard-username", "admin",
        "--adguard-password", "yourpassword"
      ]
    }
  }
}
```

## :test_tube: Testing

The project uses [go-task](https://taskfile.dev/) for development tasks.

```bash
# Run all checks (format, lint, unit tests)
task test:ci

# Run unit tests only
task test

# Run Docker integration tests (requires Docker)
RUN_DOCKER_TESTS=true task test:integration

# Update cargo dependencies
task update

### :bar_chart: Coverage

The project uses `cargo-llvm-cov` for code coverage analysis.

```bash
# Show coverage summary in console
task coverage

# Generate detailed HTML and LCOV reports
task coverage:report

# Upload coverage to Coveralls.io (requires COVERALLS_REPO_TOKEN)
COVERALLS_REPO_TOKEN=your_token task coverage:upload
```
```

## :handshake: Contributing

Contributions are welcome! Please follow standard Rust coding conventions and ensure all tests pass (`task check`) before submitting features.

## :balance_scale: License

​[Apache License 2.0](LICENSE)

## :writing_hand: Author

​This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
