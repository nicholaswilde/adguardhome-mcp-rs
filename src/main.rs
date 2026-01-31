use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::mcp;
use adguardhome_mcp_rs::settings::Settings;
use adguardhome_mcp_rs::tools::ToolRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let settings = Settings::from_env().unwrap_or_else(|_| Settings {
        adguard_url: "http://localhost:8080".to_string(),
        adguard_username: None,
        adguard_password: None,
        lazy_mode: false,
    });

    let adguard_client = AdGuardClient::new(settings.clone());
    let mut registry = ToolRegistry::new(&settings);

    // Register get_status
    registry.register(
        "get_status",
        "Get AdGuard Home status and version",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let status = client.get_status().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("AdGuard Home Version: {}\nProtection Enabled: {}", status.version, status.protection_enabled)
                        }
                    ]
                }))
            }
        },
    );

    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();

    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = stdin.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        let message = line.trim();
        if message.is_empty() {
            continue;
        }

        if let Ok(mcp::Message::Request(req)) = serde_json::from_str::<mcp::Message>(message) {
            let response = match req.method.as_str() {
                "initialize" => mcp::Response {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {
                                "listChanged": true
                            }
                        },
                        "serverInfo": {
                            "name": "adguardhome-mcp-rs",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    })),
                    error: None,
                },
                "list_tools" => {
                    let mut tools = registry.list_tools();
                    if settings.lazy_mode {
                        tools.push(serde_json::json!({
                            "name": "manage_tools",
                            "description": "Manage available tools (enable/disable) to save tokens.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "action": {
                                        "type": "string",
                                        "enum": ["list", "enable", "disable"],
                                        "description": "The action to perform."
                                    },
                                    "tools": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "List of tool names to enable or disable."
                                    }
                                },
                                "required": ["action"]
                            }
                        }));
                        // Deduplicate just in case, though not expected
                    }
                    
                    mcp::Response {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(serde_json::json!({
                            "tools": tools
                        })),
                        error: None,
                    }
                },
                "call_tool" => {
                    let tool_name = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    let args = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned();

                    if tool_name == "manage_tools" && settings.lazy_mode {
                        let action = args.as_ref().and_then(|a| a.get("action")).and_then(|s| s.as_str());
                        match action {
                            Some("list") => {
                                let available = registry.list_available_tools();
                                mcp::Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: req.id,
                                    result: Some(serde_json::json!({
                                        "content": [{
                                            "type": "text",
                                            "text": serde_json::to_string_pretty(&available).unwrap_or_default()
                                        }]
                                    })),
                                    error: None,
                                }
                            }
                            Some("enable") => {
                                let tools_to_enable = args.as_ref()
                                    .and_then(|a| a.get("tools"))
                                    .and_then(|t| t.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                                    .unwrap_or_default();
                                
                                let mut enabled_count = 0;
                                for name in tools_to_enable {
                                    if registry.enable_tool(name) {
                                        enabled_count += 1;
                                    }
                                }
                                
                                if enabled_count > 0 {
                                    // Send notification
                                    let notification = mcp::Message::Notification(mcp::Notification {
                                        jsonrpc: "2.0".to_string(),
                                        method: "notifications/tools/list_changed".to_string(),
                                        params: None,
                                    });
                                    let note_json = serde_json::to_string(&notification)? + "\n";
                                    stdout.write_all(note_json.as_bytes()).await?;
                                    stdout.flush().await?;
                                }

                                mcp::Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: req.id,
                                    result: Some(serde_json::json!({
                                        "content": [{
                                            "type": "text",
                                            "text": format!("Enabled {} tools.", enabled_count)
                                        }]
                                    })),
                                    error: None,
                                }
                            }
                            Some("disable") => {
                                let tools_to_disable = args.as_ref()
                                    .and_then(|a| a.get("tools"))
                                    .and_then(|t| t.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                                    .unwrap_or_default();
                                
                                let mut disabled_count = 0;
                                for name in tools_to_disable {
                                    if registry.disable_tool(name) {
                                        disabled_count += 1;
                                    }
                                }

                                if disabled_count > 0 {
                                    // Send notification
                                    let notification = mcp::Message::Notification(mcp::Notification {
                                        jsonrpc: "2.0".to_string(),
                                        method: "notifications/tools/list_changed".to_string(),
                                        params: None,
                                    });
                                    let note_json = serde_json::to_string(&notification)? + "\n";
                                    stdout.write_all(note_json.as_bytes()).await?;
                                    stdout.flush().await?;
                                }

                                mcp::Response {
                                    jsonrpc: "2.0".to_string(),
                                    id: req.id,
                                    result: Some(serde_json::json!({
                                        "content": [{
                                            "type": "text",
                                            "text": format!("Disabled {} tools.", disabled_count)
                                        }]
                                    })),
                                    error: None,
                                }
                            }
                            _ => mcp::Response {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: None,
                                error: Some(mcp::ResponseError {
                                    code: -32602,
                                    message: "Invalid or missing 'action' argument".to_string(),
                                    data: None,
                                }),
                            }
                        }
                    } else {
                        match registry.call_tool(tool_name, &adguard_client, args).await {
                            Ok(result) => mcp::Response {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: Some(result),
                                error: None,
                            },
                            Err(e) => mcp::Response {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: None,
                                error: Some(mcp::ResponseError {
                                    code: -32000,
                                    message: e.to_string(),
                                    data: None,
                                }),
                            },
                        }
                    }
                }
                _ => mcp::Response {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(mcp::ResponseError {
                        code: -32601,
                        message: format!("Method not found: {}", req.method),
                        data: None,
                    }),
                },
            };

            let resp_json = serde_json::to_string(&response)? + "\n";
            stdout.write_all(resp_json.as_bytes()).await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}
