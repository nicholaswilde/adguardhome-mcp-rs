pub mod adguard;
pub mod config;
pub mod error;
pub mod mcp;
pub mod server;
pub mod sync;
pub mod tools;

pub async fn run(args: Vec<String>) -> anyhow::Result<()> {
    use crate::config::AppConfig;
    use crate::server::http::run_http_server;
    use crate::server::mcp::McpServer;
    use crate::tools::ToolRegistry;
    use crate::tools::{clients, dns, filtering, protection, sync as sync_tools, system};

    // Load configuration
    let config = AppConfig::load(None, args)?;

    let mut registry = ToolRegistry::new(&config);

    // Register tools from sub-modules
    system::register(&mut registry);
    dns::register(&mut registry);
    protection::register(&mut registry);
    filtering::register(&mut registry);
    clients::register(&mut registry);
    sync_tools::register(&mut registry);

    let (server, rx) = McpServer::new(registry, config.clone());

    // Start background sync task
    let sync_config = config.clone();
    tokio::spawn(async move {
        crate::sync::SyncState::run_background_sync(sync_config).await;
    });

    match config.mcp_transport.as_str() {
        "http" => {
            run_http_server(
                server,
                rx,
                "0.0.0.0",
                config.http_port,
                config.http_auth_token,
            )
            .await?;
        }
        _ => {
            server.run_stdio(rx).await?;
        }
    }

    Ok(())
}

pub mod test_utils {
    use std::sync::Mutex;
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}
