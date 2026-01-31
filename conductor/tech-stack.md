# Technology Stack

## Core Technologies
- **Rust:** The primary programming language, chosen for its memory safety, performance, and excellent support for asynchronous programming.
- **Model Context Protocol (MCP):** The framework used to expose AdGuard Home functionality as tools and resources for AI models.

## Infrastructure and Runtime
- **Tokio:** The industry-standard asynchronous runtime for Rust, providing the foundation for high-performance I/O operations.
- **Serde:** A high-performance framework for serializing and deserializing Rust data structures, essential for handling MCP JSON-RPC messages and AdGuard Home API responses.
- **go-task:** A task runner / build tool that uses a simple `Taskfile.yml` to define and run development tasks.
- **Config:** A configuration management library that layers default values, files (TOML, YAML, JSON), environment variables, and CLI arguments.
- **Clap:** A command-line argument parser for Rust.

## Networking and API Interaction
- **Reqwest:** An ergonomic and powerful HTTP client used to interact with the AdGuard Home REST API.

## Observability and Error Handling
- **Tracing:** A framework for application-level tracing and structured logging, ensuring deep visibility into the MCP server's operations.
- **Thiserror / Anyhow:** Libraries for defining and managing custom error types, providing robust and idiomatic error handling across the codebase.

## Testing
- **Testcontainers:** Used for integration testing with real AdGuard Home Docker containers.