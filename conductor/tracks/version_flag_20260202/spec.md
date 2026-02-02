# Specification: Version Command Line Switch

## Overview
Add a `--version` (and `-V`) command line switch to the `adguardhome-mcp-rs` binary to display the current version of the application. The version should be sourced from the crate's version defined in `Cargo.toml`.

## Functional Requirements
- The application must support `--version` and `-V` flags.
- When either flag is provided, the application should print its name and version number and then exit.
- The output format must be `<app-name> <version>` (e.g., `adguardhome-mcp-rs 0.1.0`).
- The version must be sourced from the `CARGO_PKG_VERSION` environment variable at build time.
- The output should NOT include commit hashes or dirty state indicators.

## Non-Functional Requirements
- Minimal impact on startup time.
- Adhere to Rust conventions for CLI argument parsing (using `clap`).

## Acceptance Criteria
- Running `./adguardhome-mcp-rs --version` prints `adguardhome-mcp-rs x.y.z` and exits with code 0.
- Running `./adguardhome-mcp-rs -V` prints `adguardhome-mcp-rs x.y.z` and exits with code 0.
- The version number matches the version in `Cargo.toml`.

## Out of Scope
- Displaying build timestamps.
- Displaying Git commit SHAs.
- Displaying compiler information.
