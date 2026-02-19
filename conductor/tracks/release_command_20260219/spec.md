# Specification: Gemini CLI Release Command

## Overview
Implement a custom Gemini CLI command named `release` to automate the versioning, tagging, and deployment process for the project. This command will simplify the release workflow by automatically bumping the patch version, creating a git commit and tag, and pushing changes to the remote repository atomically.

## Functional Requirements
### 1. Command Definition
- **File Location:** The command definition must be stored in `.gemini/commands/release.toml`.
- **Command Name:** The command should be accessible via the Gemini CLI (e.g., as `release`).

### 2. Version Management
- **Automatic Bump:** The command must read `Cargo.toml`, extract the current version, and increment the **patch** version (e.g., `0.1.15` -> `0.1.16`).
- **File Update:** Update `Cargo.toml` and `Cargo.lock` (via `cargo check` or similar) with the new version.
- **README Update:** Update the version reference in the `README.md` warning section.

### 3. Git Operations
- **Commit:** Create a git commit including the updated `Cargo.toml`, `Cargo.lock`, and `README.md`.
    - **Message Format:** `chore: Bump version to <version>` (using `-m` to avoid opening an editor).
- **Tagging:** Create an annotated git tag for the new version.
    - **Tag Format:** `v<version>` (e.g., `v0.1.16`).
    - **Message:** Use the version string as the tag message (using `-m`).
- **Atomic Push:** Push the `main` branch and the new tag to the remote repository atomically using `git push --atomic origin main <tag>`.

### 4. Conflict and Error Handling
- **Pre-Push Check:** Before pushing, the command should attempt to handle remote conflicts.
- **Rebase Logic:** If the push fails or as a preventative measure, the command should attempt `git pull --rebase` before retrying the atomic push.

## Acceptance Criteria
- Running the `release` command successfully increments the version in `Cargo.toml`.
- A git commit is created with the correct message without opening an interactive editor.
- An annotated git tag is created following the `vX.Y.Z` format.
- The branch and tag are successfully pushed to the remote repository.
- The `README.md` version reference is kept in sync.
