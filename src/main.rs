use adguardhome_mcp_rs::adguard::{AdGuardClient, DnsRewrite};
use adguardhome_mcp_rs::config::AppConfig;
use adguardhome_mcp_rs::server::http::run_http_server;
use adguardhome_mcp_rs::server::mcp::McpServer;
use adguardhome_mcp_rs::tools::ToolRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::load(None, std::env::args().collect())?;

    let adguard_client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);

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

    // Register list_dns_rewrites
    registry.register(
        "list_dns_rewrites",
        "List all DNS rewrites",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let rewrites = client.list_rewrites().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&rewrites)?
                        }
                    ]
                }))
            }
        },
    );

    // Register add_dns_rewrite
    registry.register(
        "add_dns_rewrite",
        "Add a new DNS rewrite",
        serde_json::json!({
            "type": "object",
            "properties": {
                "domain": {
                    "type": "string",
                    "description": "Domain name"
                },
                "answer": {
                    "type": "string",
                    "description": "IP address or canonical name"
                }
            },
            "required": ["domain", "answer"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let domain = params["domain"].as_str().unwrap_or_default().to_string();
            let answer = params["answer"].as_str().unwrap_or_default().to_string();
            async move {
                client.add_rewrite(DnsRewrite { domain, answer }).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "DNS rewrite added successfully"
                        }
                    ]
                }))
            }
        },
    );

    // Register remove_dns_rewrite
    registry.register(
        "remove_dns_rewrite",
        "Remove a DNS rewrite",
        serde_json::json!({
            "type": "object",
            "properties": {
                "domain": {
                    "type": "string",
                    "description": "Domain name"
                },
                "answer": {
                    "type": "string",
                    "description": "IP address or canonical name"
                }
            },
            "required": ["domain", "answer"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let domain = params["domain"].as_str().unwrap_or_default().to_string();
            let answer = params["answer"].as_str().unwrap_or_default().to_string();
            async move {
                client.delete_rewrite(DnsRewrite { domain, answer }).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "DNS rewrite removed successfully"
                        }
                    ]
                }))
            }
        },
    );

    let server = McpServer::new(adguard_client, registry, config.clone());

    match config.mcp_transport.as_str() {
        "http" => {
            run_http_server(server, "0.0.0.0", config.http_port, config.http_auth_token).await?;
        }
        _ => {
            server.run_stdio().await?;
        }
    }

    Ok(())
}
