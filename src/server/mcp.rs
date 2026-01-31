use crate::adguard::AdGuardClient;
use crate::config::AppConfig;
use crate::mcp::{Message, Notification, Request, Response, ResponseError};
use crate::tools::ToolRegistry;
use anyhow::Result;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

#[derive(Clone)]
pub struct McpServer {
    client: AdGuardClient,
    registry: Arc<Mutex<ToolRegistry>>,
    config: AppConfig,
}

impl McpServer {
    pub fn new(client: AdGuardClient, registry: ToolRegistry, config: AppConfig) -> Self {
        Self {
            client,
            registry: Arc::new(Mutex::new(registry)),
            config,
        }
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let mut reader = BufReader::new(stdin()).lines();
        let mut stdout = stdout();

        while let Some(line) = reader.next_line().await? {
            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            if let Ok(Message::Request(req)) = serde_json::from_str::<Message>(input) {
                let id = req.id.clone();
                let response = self.handle_request(req).await;

                let json_resp = match response {
                    Ok(result) => Response {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(result),
                        error: None,
                    },
                    Err(e) => Response {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(ResponseError {
                            code: -32000,
                            message: e.to_string(),
                            data: None,
                        }),
                    },
                };

                let out = serde_json::to_string(&json_resp)? + "\n";
                stdout.write_all(out.as_bytes()).await?;
                stdout.flush().await?;
            }
        }
        Ok(())
    }

    pub async fn handle_request(&self, req: Request) -> Result<Value> {
        match req.method.as_str() {
            "initialize" => Ok(serde_json::json!({
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
            "list_tools" => {
                let mut tools = {
                    let registry = self.registry.lock().unwrap();
                    registry.list_tools()
                };

                if self.config.lazy_mode {
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
                }

                Ok(serde_json::json!({
                    "tools": tools
                }))
            }
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

                if tool_name == "manage_tools" && self.config.lazy_mode {
                    self.handle_manage_tools(args).await
                } else {
                    // Logic to call tool without holding mutex across await
                    let handler = {
                        let registry = self.registry.lock().unwrap();
                        // We need a way to get the handler or clone the tool logic
                        // Re-implementing logic here to avoid exposing too much from ToolRegistry?
                        // Or add `get_handler` to ToolRegistry.
                        if !registry.is_tool_enabled(tool_name) {
                            return Err(anyhow::anyhow!(
                                "Tool not found or not enabled: {}",
                                tool_name
                            ));
                        }
                        registry.get_tool(tool_name).map(|t| t.handler.clone())
                    };

                    if let Some(handler) = handler {
                        handler(&self.client, args)
                            .await
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Tool not found: {}", tool_name))
                    }
                }
            }
            _ => Err(anyhow::anyhow!("Method not found: {}", req.method)),
        }
    }

    async fn handle_manage_tools(&self, args: Option<Value>) -> Result<Value> {
        let action = args
            .as_ref()
            .and_then(|a| a.get("action"))
            .and_then(|s| s.as_str());
        match action {
            Some("list") => {
                let registry = self.registry.lock().unwrap();
                let available = registry.list_available_tools();
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&available).unwrap_or_default()
                    }]
                }))
            }
            Some("enable") => {
                let tools_to_enable = args
                    .as_ref()
                    .and_then(|a| a.get("tools"))
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();

                let mut enabled_count = 0;
                let mut should_notify = false;
                {
                    let mut registry = self.registry.lock().unwrap();
                    for name in tools_to_enable {
                        if registry.enable_tool(name) {
                            enabled_count += 1;
                            should_notify = true;
                        }
                    }
                }

                if should_notify {
                    self.send_notification("notifications/tools/list_changed", None)
                        .await?;
                }

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Enabled {} tools.", enabled_count)
                    }]
                }))
            }
            Some("disable") => {
                let tools_to_disable = args
                    .as_ref()
                    .and_then(|a| a.get("tools"))
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();

                let mut disabled_count = 0;
                let mut should_notify = false;
                {
                    let mut registry = self.registry.lock().unwrap();
                    for name in tools_to_disable {
                        if registry.disable_tool(name) {
                            disabled_count += 1;
                            should_notify = true;
                        }
                    }
                }

                if should_notify {
                    self.send_notification("notifications/tools/list_changed", None)
                        .await?;
                }

                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Disabled {} tools.", disabled_count)
                    }]
                }))
            }
            _ => Err(anyhow::anyhow!("Invalid or missing 'action' argument")),
        }
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = Notification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };
        // For stdio, we write to stdout.
        // NOTE: This assumes we are running in stdio mode.
        // If we are in HTTP mode, we should push to a queue or similar.
        // For now, I'll write to stdout, but this needs abstraction for HTTP support later.
        // In qbittorrent-mcp-rs, McpServer had a notification queue.

        let out = serde_json::to_string(&Message::Notification(notification))? + "\n";
        let mut stdout = stdout();
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    }
}
