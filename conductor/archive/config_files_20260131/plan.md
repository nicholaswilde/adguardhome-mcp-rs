# Implementation Plan - Configuration File Support

This plan follows the TDD approach and mimics the robust configuration system found in `qbittorrent-mcp-rs`.

## Phase 1: Dependency and Type Setup
- [~] Task: Add `config` and `clap` dependencies
    - [ ] Update `Cargo.toml` to include `config` (with `toml`, `yaml`, `json` features) and `clap` (with `derive` and `env` features).
- [~] Task: Define `AppConfig` and `TransportMode` types
    - [ ] Create `src/config.rs` (or update `src/settings.rs`) to mirror the `qbittorrent-mcp-rs` structure.
    - [ ] Implement `Deserialize` for the configuration structs.
- [ ] Task: Verify Compilation
    - [ ] Run `cargo check` to ensure new dependencies and types compile.
- [ ] Task: Conductor - User Manual Verification 'Dependency and Type Setup' (Protocol in workflow.md)

## Phase 2: Configuration Loader Implementation
- [~] Task: Implement `AppConfig::load`
    - [~] Implement logic to build the config from Defaults -> File -> Environment -> CLI.
    - [~] Write unit tests for precedence (e.g., verify CLI overrides Env).
- [~] Task: Implement Search Path Logic
    - [~] Add logic to check current directory and standard user config paths.
    - [~] Write unit tests for path discovery.
- [ ] Task: Verify Loader
    - [ ] Run `task test:ci` to ensure unit tests pass.
- [ ] Task: Conductor - User Manual Verification 'Configuration Loader Implementation' (Protocol in workflow.md)

## Phase 3: Integration and Refactoring
- [~] Task: Update `main.rs` to use `AppConfig`
    - [~] Replace the manual `Settings` loading with `AppConfig::load(None, std::env::args().collect())`.
    - [~] Update `AdGuardClient` and `ToolRegistry` initialization to use the new config object.
- [~] Task: Verify with existing Integration Tests
    - [~] Run `task test:ci` to ensure transport selection and auth settings are still correctly applied from environment variables.
- [ ] Task: Conductor - User Manual Verification 'Integration and Refactoring' (Protocol in workflow.md)

## Phase 4: Final Validation
- [x] Task: Add File-based Integration Test
    - [x] Create a new integration test that writes a temporary `config.toml` and verifies the server picks it up.
    - [x] Run `task test:ci`.
- [x] Task: Conductor - User Manual Verification 'Final Validation' (Protocol in workflow.md)