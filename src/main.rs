use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::config::AppConfig;
use adguardhome_mcp_rs::server::http::run_http_server;
use adguardhome_mcp_rs::server::mcp::McpServer;
use adguardhome_mcp_rs::tools::ToolRegistry;
use adguardhome_mcp_rs::tools::{clients, dns, filtering, protection, system};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::load(None, std::env::args().collect())?;

    let adguard_client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);

    // Register tools from sub-modules
    system::register(&mut registry);
    dns::register(&mut registry);
    protection::register(&mut registry);
    filtering::register(&mut registry);
    clients::register(&mut registry);

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
