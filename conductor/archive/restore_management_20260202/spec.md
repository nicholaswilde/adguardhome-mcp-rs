# Restore Management Specification

## User Story
As an AI agent, I want to restore a previous backup if a configuration change fails or causes issues.

## Requirements
- **Tool:** `restore_backup`
- **Input:** `file_path` (string) - The path to the backup file to restore.
- **Output:** Success message.
- **API Endpoint:** `POST /control/restore`.
    - Content-Type: `multipart/form-data` or similar.

## Technical Details
- Endpoint: `/control/restore`.
- The tool needs to read the file from the local disk (created by `create_backup`) and upload it to AdGuard Home.
