# Codebase Cleanup Specification

## User Story
As a developer, I want to remove duplicate code to ensure maintainability and correct help output.

## Requirements
- Remove duplicate registrations of:
    - `get_top_blocked_domains`
    - `get_client_activity_report`
- In `src/main.rs`.

## Technical Details
- Identify the redundant blocks in `src/main.rs` and delete them.
