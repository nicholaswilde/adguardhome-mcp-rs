use crate::adguard::AdGuardClient;
use crate::config::AppConfig;
use crate::mcp::{Message, Notification, Request, Response, ResponseError};
use crate::tools::ToolRegistry;
use anyhow::Result;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

use tokio::sync::mpsc;

#[derive(Clone)]
pub struct McpServer {
    pub registry: Arc<Mutex<ToolRegistry>>,
    pub config: AppConfig,
    pub notification_tx: mpsc::Sender<Notification>,
}

impl McpServer {
    pub fn new(registry: ToolRegistry, config: AppConfig) -> (Self, mpsc::Receiver<Notification>) {
        Self::with_registry(Arc::new(Mutex::new(registry)), config)
    }

    pub fn with_registry(
        registry: Arc<Mutex<ToolRegistry>>,
        config: AppConfig,
    ) -> (Self, mpsc::Receiver<Notification>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                registry,
                config,
                notification_tx: tx,
            },
            rx,
        )
    }

    pub async fn run_stdio(&self, rx: mpsc::Receiver<Notification>) -> Result<()> {
        self.run(stdin(), stdout(), rx).await
    }

    pub async fn run<R, W>(
        &self,
        reader: R,
        mut writer: W,
        mut rx: mpsc::Receiver<Notification>,
    ) -> Result<()>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(reader).lines();

        loop {
            tokio::select! {
                line = reader.next_line() => {
                    let line = line?;
                    if let Some(line) = line {
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
                            writer.write_all(out.as_bytes()).await?;
                            writer.flush().await?;
                        }
                    } else {
                        break;
                    }
                }
                notification = rx.recv() => {
                    if let Some(n) = notification {
                        let out = serde_json::to_string(&Message::Notification(n))? + "\n";
                        writer.write_all(out.as_bytes()).await?;
                        writer.flush().await?;
                    }
                }
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
                    // 1. Extract instance
                    let instance_name = args
                        .as_ref()
                        .and_then(|a| a.get("instance"))
                        .and_then(|i| i.as_str());

                    // 2. Get instance config
                    let instance_config = match self.config.get_instance(instance_name) {
                        Ok(c) => c,
                        Err(e) => return Err(anyhow::anyhow!(e)),
                    };

                    // 3. Create client for this instance
                    let client = AdGuardClient::new(instance_config.clone());

                    let handler = {
                        let registry = self.registry.lock().unwrap();
                        if !registry.is_tool_enabled(tool_name) {
                            return Err(anyhow::anyhow!(
                                "Tool not found or not enabled: {}",
                                tool_name
                            ));
                        }
                        registry.get_tool(tool_name).map(|t| t.handler.clone())
                    };

                    if let Some(handler) = handler {
                        handler(&client, &self.config, args)
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
        let _ = self.notification_tx.send(notification).await;
        Ok(())
    }
}
