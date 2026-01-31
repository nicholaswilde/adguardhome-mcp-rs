use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::mcp;
use adguardhome_mcp_rs::settings::Settings;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let settings = Settings::from_env().unwrap_or_else(|_| Settings {
        adguard_url: "http://localhost:8080".to_string(),
        adguard_username: None,
        adguard_password: None,
    });

    let adguard_client = AdGuardClient::new(settings);

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
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "adguardhome-mcp-rs",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    })),
                    error: None,
                },
                "list_tools" => mcp::Response {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::json!({
                        "tools": [
                            {
                                "name": "get_status",
                                "description": "Get AdGuard Home status and version",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            }
                        ]
                    })),
                    error: None,
                },
                "call_tool" => {
                    let tool_name = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    match tool_name {
                        "get_status" => match adguard_client.get_status().await {
                            Ok(status) => mcp::Response {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: Some(serde_json::json!({
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": format!("AdGuard Home Version: {}\nProtection Enabled: {}", status.version, status.protection_enabled)
                                        }
                                    ]
                                })),
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
                        },
                        _ => mcp::Response {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: None,
                            error: Some(mcp::ResponseError {
                                code: -32601,
                                message: format!("Tool not found: {}", tool_name),
                                data: None,
                            }),
                        },
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
