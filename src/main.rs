use adguardhome_mcp_rs::adguard::AdGuardClient;
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
