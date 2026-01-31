# :crab: AdGuard Home MCP RS :robot:

[![task](https://img.shields.io/badge/Task-Enabled-brightgreen?style=for-the-badge&logo=task&logoColor=white)](https://taskfile.dev/#/)

> [!WARNING]
> This project is currently in active development (v0.1.0) and is **not production-ready**. Features may change, and breaking changes may occur without notice.

A Rust implementation of an AdGuard Home [MCP (Model Context Protocol) server](https://modelcontextprotocol.io/docs/getting-started/intro). This server connects to an AdGuard Home instance and exposes tools to monitor and manage filtering via the Model Context Protocol.

## :sparkles: Features

- **Protocol:** JSON-RPC 2.0 over Stdio (MCP standard).
- **Authentication:** Supports AdGuard Home username and password authentication.
- **Tools:**
  - `get_status`: Get AdGuard Home status, version, and protection state.

## :hammer_and_wrench: Build

To build the project, you need a Rust toolchain installed.

```bash
cargo build --release
```

The binary will be available at `target/release/adguardhome-mcp-rs`.

## :rocket: Usage

You can run the server directly from the command line.

### :keyboard: Command Line Interface

The server is configured via environment variables for security.

```bash
export ADGUARD_URL="http://192.168.1.10:8080"
export ADGUARD_USERNAME="admin"
export ADGUARD_PASSWORD="yourpassword"

./target/release/adguardhome-mcp-rs
```

### :earth_africa: Environment Variables

- `ADGUARD_URL`: The base URL of your AdGuard Home instance (e.g., `http://192.168.1.10:8080`).
- `ADGUARD_USERNAME`: Your AdGuard Home username.
- `ADGUARD_PASSWORD`: Your AdGuard Home password.

### :robot: Configuration Example (Claude Desktop)

Add the following to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "adguardhome": {
      "command": "/path/to/adguardhome-mcp-rs/target/release/adguardhome-mcp-rs",
      "env": {
        "ADGUARD_URL": "http://192.168.1.10:8080",
        "ADGUARD_USERNAME": "admin",
        "ADGUARD_PASSWORD": "yourpassword"
      }
    }
  }
}
```

## :test_tube: Testing

### Using Taskfile
The project uses [go-task](https://taskfile.dev/) for common development tasks.

```bash
# Run all checks (format, lint, test)
task check

# Run unit tests
task test

# Run Docker integration tests (requires Docker)
RUN_DOCKER_TESTS=true task test
```

## :handshake: Contributing

Contributions are welcome! Please follow standard Rust coding conventions and include tests for new features.

## :balance_scale: License

​[Apache License 2.0](LICENSE)

## :writing_hand: Author

​This project was started in 2026 by [Nicholas Wilde][2].

[2]: <https://github.com/nicholaswilde/>
