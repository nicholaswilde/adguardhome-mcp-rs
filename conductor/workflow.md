# Project Workflow

## Guiding Principles

1. **The Plan is the Source of Truth:** All work must be tracked in `plan.md`
2. **The Tech Stack is Deliberate:** Changes to the tech stack must be documented in `tech-stack.md` *before* implementation
3. **Test-Driven Development:** Write unit tests before implementing functionality
4. **High Code Coverage:** Aim for >80% code coverage for all modules
5. **User Experience First:** Every decision should prioritize user experience
6. **Non-Interactive & CI-Aware:** Prefer non-interactive commands. Use `CI=true` for watch-mode tools (tests, linters) to ensure single execution.

## Task Workflow

All tasks follow a strict lifecycle:

### Standard Task Workflow

1. **Select Task:** Choose the next available task from `plan.md` in sequential order

2. **Mark In Progress:** Before beginning work, edit `plan.md` and change the task from `[ ]` to `[~]`

3. **Write Failing Tests (Red Phase):**
   - Create a new test file for the feature or bug fix.
   - Write one or more unit tests that clearly define the expected behavior and acceptance criteria for the task.
   - **CRITICAL:** Run the tests and confirm that they fail as expected. This is the "Red" phase of TDD. Do not proceed until you have failing tests.

4. **Implement to Pass Tests (Green Phase):**
   - Write the minimum amount of application code necessary to make the failing tests pass.
   - Run the test suite again and confirm that all tests now pass. This is the "Green" phase.

5. **Refactor (Optional but Recommended):**
   - With the safety of passing tests, refactor the implementation code and the test code to improve clarity, remove duplication, and enhance performance without changing the external behavior.
   - Rerun tests to ensure they still pass after refactoring.

6. **Verify Coverage & Compliance:**
   - Run `task test:ci` to ensure all tests pass, code is formatted, and no linting errors exist.
   - Run coverage reports (if not covered by `test:ci`).

7. **Document Deviations:** If implementation differs from tech stack:
   - **STOP** implementation
   - Update `tech-stack.md` with new design
   - Add dated note explaining the change
   - Resume implementation

8. **Mark Task Complete in Plan:**
    - Update `plan.md`, find the line for the completed task, and change its status from `[~]` to `[x]`.
    - Since commits are performed at the end of the phase, do not record a commit SHA at this step.

### Phase Completion Verification and Checkpointing Protocol

**Trigger:** This protocol is executed immediately after all tasks in a phase in `plan.md` are marked as complete.

1.  **Announce Protocol Start:** Inform the user that the phase is complete and the verification and checkpointing protocol has begun.

2.  **Ensure Test Coverage for Phase Changes:**
    -   **Step 2.1: Determine Phase Scope:** Identify all code changes since the last phase checkpoint.
    -   **Step 2.2: List Changed Files:** Review staged and unstaged changes in the working directory.
    -   **Step 2.3: Verify and Create Tests:** Ensure all new or modified code files have corresponding tests that validate the functionality described in the phase's tasks.

3.  **Execute Automated Tests with Proactive Debugging:**
    -   Announce the test command (e.g., `task test:ci`).
    -   Execute the command.
    -   If tests fail, debug and fix (maximum of two attempts before asking the user).

4.  **Propose a Detailed, Actionable Manual Verification Plan:**
    -   Analyze the completed phase's goals and provide a step-by-step verification plan for the user.

5.  **Await Explicit User Feedback:**
    -   Wait for user confirmation ("yes" or feedback) before proceeding.

6.  **Create Checkpoint Commit:**
    -   Stage all changes (including code and `plan.md`).
    -   Perform the commit with a message like `conductor(checkpoint): Checkpoint end of Phase X - <Phase Description>`.

7.  **Attach Auditable Verification Report using Git Notes:**
    -   **Step 7.1: Draft Note Content:** Create a detailed report including task summaries, test results, and the manual verification outcome.
    -   **Step 7.2: Attach Note:** Attach the report to the checkpoint commit using `git notes add`.

8.  **Get and Record Phase Checkpoint SHA:**
    -   Obtain the hash of the checkpoint commit and update `plan.md` header for the phase with `[checkpoint: <sha>]`.

9. **Commit Plan Update (if needed):**
    - If `plan.md` was updated with the SHA after the checkpoint commit, create a small follow-up commit: `conductor(plan): Record checkpoint SHA for Phase X`.

10.  **Announce Completion:** Inform the user that the phase is complete and the checkpoint has been created.

## Quality Gates

Before marking any task complete, verify:

- [ ] All tests pass
- [ ] New tools/functions are covered by Docker integration tests
- [ ] Code coverage meets requirements (>80%)
- [ ] Code follows project's code style guidelines
- [ ] All public functions/methods are documented
- [ ] Type safety is enforced
- [ ] No linting or static analysis errors (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] No security vulnerabilities introduced

## Development Commands

### Setup
```bash
task setup
```

### Daily Development
```bash
task run
task test
task lint
task format
```

### Before Committing
```bash
task check
```

## Testing Requirements

### Unit Testing
- Every module must have corresponding tests.
- Use appropriate test setup/teardown.
- Mock external dependencies (e.g., AdGuard Home API).
- Test both success and failure cases.

### Integration Testing
- All new MCP tools and core functionality MUST be added to `tests/docker_integration_test.rs`.
- Integration tests must verify functionality against a real AdGuard Home Docker container.
- Tests should cover both authenticated and unauthenticated scenarios if applicable.

## Commit Guidelines

### Message Format
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests
- `chore`: Maintenance tasks

## Definition of Done

A task is complete when:

1. All code implemented to specification
2. Unit tests written and passing
3. Code coverage meets project requirements (>80%)
4. Documentation complete (if applicable)
5. Code passes all configured linting and static analysis checks (`cargo clippy`)
6. Changes staged and ready for phase-end commit
7. Implementation notes prepared for git notes
8. New MCP tools or core functions are verified via `tests/docker_integration_test.rs`

## Deployment Workflow

### Pre-Deployment Checklist
- [ ] All tests passing
- [ ] Coverage >80%
- [ ] No clippy warnings
- [ ] Environment variables configured
- [ ] Backup created

### Deployment Steps
1. Merge feature branch to main
2. Tag release with version
3. Push to deployment service
4. Verify deployment
5. Monitor for errors