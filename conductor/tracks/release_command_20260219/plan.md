# Implementation Plan: Gemini CLI Release Command

## Phase 1: Command Definition & Versioning Logic
Establish the command structure and implement the core version bumping logic.

- [x] Task: Create Command Definition
    - [x] Create the `.gemini/commands/release.toml` file with basic metadata.
- [x] Task: Implement Version Bumping
    - [x] Implement logic to read `Cargo.toml`.
    - [x] Implement regex or parsing logic to extract the current version.
    - [x] Implement logic to increment the patch version.
    - [x] Update `Cargo.toml` with the new version string.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Command Definition & Versioning Logic' (Protocol in workflow.md)

## Phase 2: Git Operations & Maintenance
Integrate git commands and ensure all project artifacts are kept in sync.

- [ ] Task: Implement Git Commit & Tag
    - [ ] Implement logic to run `cargo check` to update `Cargo.lock`.
    - [ ] Implement logic to update the version reference in `README.md`.
    - [ ] Implement git commit for the changed files with message `chore: Bump version to <version>`.
    - [ ] Implement creation of an annotated git tag `v<version>`.
- [ ] Task: Implement Atomic Push & Pull logic
    - [ ] Implement `git pull --rebase` logic to ensure local is up to date.
    - [ ] Implement `git push --atomic origin main <tag>` command.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Git Operations & Maintenance' (Protocol in workflow.md)

## Phase 3: Final Verification
Verify the end-to-end flow and refine the user experience.

- [ ] Task: End-to-End Dry Run
    - [ ] Perform a manual dry run of the command (e.g., using echo instead of actual git commands) to verify the sequence.
- [ ] Task: Documentation & Cleanup
    - [ ] Ensure the command description is clear and helpful.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Final Verification' (Protocol in workflow.md)
