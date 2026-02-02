# Backup Management Specification

## User Story
As an AI agent, I want to create a backup of the AdGuard Home configuration so that I can ensure the user's data is safe before attempting complex or risky changes.

## Requirements
- **Tool:** `create_backup`
- **Input:** None (optional filename/path if supported, but usually standard).
- **Output:** JSON indicating success and potentially the path/content of the backup.
- **API Endpoint:** `POST /control/backup` (Check AdGuard Home API docs, likely downloads a file).
    - Note: Since we are an MCP server, returning a huge binary blob might be inefficient.
    - Option 1: Save to a local path on the server and return the path.
    - Option 2: Return a success message if it's just a server-side trigger (AdGuard might not support server-side-only backup storage without download).
    - *Constraint:* `reqwest` response handling for binary files.

## Technical Details
- Endpoint: `/control/backup` returns a `.tar.gz` file.
- Implementation: Stream the response to a local file in the `data` or `backup` directory of the MCP server, or a user-specified path in config.
- Return the full path to the saved backup file.
