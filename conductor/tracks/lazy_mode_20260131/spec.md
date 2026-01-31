# Specification - Lazy Mode (Token Optimization)

## Overview
To reduce the token consumption of the `list_tools` response—which can become very large as the number of tools grows—we will implement a "Lazy Mode". When enabled, the server will initially expose only a minimal set of management tools. The AI can then discover and "enable" specific tools or categories of tools as needed.

## Functional Requirements
1.  **Configuration:**
    -   New environment variable: `LAZY_MODE` (bool, default: `false`).
    -   If `false`, behavior remains unchanged (all tools listed).
    -   If `true`, `list_tools` initially returns only the "Meta Tools".

2.  **Meta Tools:**
    -   `manage_tools`:
        -   **Action `list`**: Returns a list of *names* and *brief descriptions* of all available tools (without full schemas).
        -   **Action `enable`**: Enables a specific tool (or list of tools) by name.
        -   **Action `disable`**: Disables a tool (removes it from the active list).

3.  **Dynamic Tool Updates:**
    -   When tools are enabled/disabled, the server must notify the client that the tool list has changed.
    -   **Mechanism:** Send a JSON-RPC notification: `notifications/tools/list_changed` (standard MCP notification).

## Technical Requirements
-   **State Management:** Need a `ToolRegistry` or similar struct to track:
    -   All registered tools (static definitions).
    -   Currently active tools (dynamic set).
-   **Notification Support:** Update the `main` loop or `mcp` module to support sending unsolicited JSON-RPC notifications to `stdout`.
-   **Concurrency:** The tool state needs to be shared between the request handler (reading stdin) and the logic that might trigger updates. (Though currently, updates are triggered *by* requests, so `RefCell` or `Mutex` might be needed if we move to a more complex architecture, but for now, the request handler owns the state).

## Security Considerations
-   None specific. Access control is handled by the underlying tools.

## User Experience
-   **Initial State (Lazy):**
    -   `list_tools` -> `[manage_tools]`
-   **Discovery:**
    -   `call_tool(manage_tools, {action: "list"})` -> `[{name: "get_status", description: "..."}]`
-   **Activation:**
    -   `call_tool(manage_tools, {action: "enable", tools: ["get_status"]})` -> "Tools enabled."
    -   *Server sends `notifications/tools/list_changed`*
    -   *Client re-fetches `list_tools`* -> `[manage_tools, get_status]`
