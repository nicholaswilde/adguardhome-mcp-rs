# :shield: AdGuard Home MCP Server (Rust) :robot:

[![Coveralls](https://img.shields.io/coveralls/github/nicholaswilde/adguardhome-mcp-rs/main?style=for-the-badge&logo=coveralls)](https://coveralls.io/github/nicholaswilde/adguardhome-mcp-rs?branch=main)
[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)
[![ci](https://img.shields.io/github/actions/workflow/status/nicholaswilde/adguardhome-mcp-rs/ci.yml?label=ci&style=for-the-badge&branch=main&logo=github-actions)](https://github.com/nicholaswilde/adguardhome-mcp-rs/actions/workflows/ci.yml)

> [!WARNING]
> This project is currently in active development (v0.1.15) and is **not production-ready**. Features may change, and breaking changes may occur without notice. **Use this MCP server at your own risk.**

A Rust implementation of an AdGuard Home [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to one or more AdGuard Home instances and exposes tools to monitor and manage filtering via the Model Context Protocol.

## :sparkles: Features

- **Multi-Transport Support:**
  - **Stdio:** Default transport for local integrations (e.g., Claude Desktop).
  - **HTTP/SSE:** Network-accessible transport for remote clients.
- **Multi-Instance Management:** Manage and target multiple AdGuard Home instances from a single MCP server. Tools accept an optional `instance` argument (name or index).
- **Multi-Instance Synchronization:** Synchronize configuration (filtering rules, blocked services, DNS rewrites) from a master instance to one or more replica instances automatically or on-demand.
- **Robust Configuration:** Supports configuration via CLI arguments, environment variables, and configuration files (TOML, YAML, JSON).
- **Authentication:**
  - Connects to AdGuard Home using username/password or API Key.
  - Secures HTTP transport with Bearer Token authentication.
- **Token Optimization:** Consolidated granular tools into functional groups to optimize AI context window usage.
  - **Tools:**
    - `manage_system`: System status, monitoring stats, query logs, backups, and maintenance.
    - `manage_dns`: DNS rewrites management, server configuration, and cache control.
    - `manage_protection`: Global protection state, safe search, safe browsing, and parental control.
    - `manage_filtering`: Adblock filter lists, custom user rules, and service blocking.
    - `manage_clients`: Network client management, DHCP leases, and access control.
    - `sync_instances`: Manually trigger synchronization to replica instances.
    - `manage_tools`: (Lazy Mode only) Dynamic on-demand loading of the above tools.

## :package: Installation

### Homebrew

```bash
brew install nicholaswilde/tap/adguardhome-mcp-rs
```

## :hammer_and_wrench: Build

To build the project, you need a Rust toolchain installed. For cross-compilation, [cross](https://github.com/cross-rs/cross) is used.

### Local Build

```bash
# Build in release mode
task build:local
```

The binary will be available at `target/release/adguardhome-mcp-rs`.

### Cross-Compilation

Supported architectures can be built using `task`:

```bash
# Build for AMD64 (x86_64)
task build:amd64

# Build for ARM64 (aarch64)
task build:arm64

# Build for ARMv7
task build:armv7

# Build for all supported architectures
task build
```

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
| - | `ADGUARD_INSTANCES__<N>__<FIELD>` | Configuration for multiple instances (see below) | - |
| - | `ADGUARD_REPLICAS` | JSON array of replica objects (`url`, `api_key`) | `[]` |
| - | `ADGUARD_SYNC_INTERVAL_SECONDS` | Interval for automated background sync | `3600` |
| - | `ADGUARD_DEFAULT_SYNC_MODE` | Default sync mode (`additive-merge` or `full-overwrite`) | `additive-merge` |

### :file_folder: Configuration File

The server automatically looks for `config.toml`, `config.yaml`, or `config.json` in the current directory and `~/.config/adguardhome-mcp-rs/`.

#### Multi-Instance Configuration

You can configure multiple AdGuard Home instances. The first instance in the list is the default.

```toml
# Default instance (legacy format supported for single instance)
adguard_host = "192.168.1.10"
adguard_port = 8080
adguard_username = "admin"
adguard_password = "yourpassword"

# OR use the instances list
[[instances]]
name = "primary"
url = "http://192.168.1.10:8080"
username = "admin"
password = "yourpassword"

[[instances]]
name = "homelab"
url = "http://10.0.0.5:3000"
api_key = "your-api-key"
no_verify_ssl = false

# Synchronization settings
sync_interval_seconds = 3600
default_sync_mode = "additive-merge"

[[replicas]]
url = "http://192.168.1.11:3000"
api_key = "replica-api-key-1"
```

#### Environment Variables for Multiple Instances

Use the pattern `ADGUARD_INSTANCES__<INDEX>__<FIELD>`:

- `ADGUARD_INSTANCES__0__NAME=primary`
- `ADGUARD_INSTANCES__0__URL=http://192.168.1.10:8080`
- `ADGUARD_INSTANCES__1__NAME=secondary`
- `ADGUARD_INSTANCES__1__URL=http://192.168.1.20:8080`

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

# Run MCP Inspector (requires npx)
task inspector

# Update cargo dependencies
task update
```

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

## :handshake: Contributing

Contributions are welcome! Please follow standard Rust coding conventions and ensure all tests pass (`task check`) before submitting features.

## :balance_scale: License

​[Apache License 2.0](LICENSE)

## :writing_hand: Author

​This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
