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

    // Register get_stats
    registry.register(
        "get_stats",
        "Get AdGuard Home statistics",
        serde_json::json!({
            "type": "object",
            "properties": {
                "time_period": {
                    "type": "string",
                    "description": "Time period (24h, 7d, 30d)",
                    "enum": ["24h", "7d", "30d"]
                }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let time_period = params["time_period"].as_str().map(|s| s.to_string());
            async move {
                let stats = client.get_stats(time_period.as_deref()).await?;
                let blocked_pct = if stats.num_dns_queries > 0 {
                    (stats.num_blocked_filtering as f64 / stats.num_dns_queries as f64) * 100.0
                } else {
                    0.0
                };
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!(
                                "Total Queries: {}\nBlocked: {} ({:.2}%)\nMalware/Phishing: {}\nSafe Search: {}\nParental Control: {}\nAvg Processing Time: {:.2}ms",
                                stats.num_dns_queries,
                                stats.num_blocked_filtering,
                                blocked_pct,
                                stats.num_replaced_safebrowsing,
                                stats.num_replaced_safesearch,
                                stats.num_replaced_parental,
                                stats.avg_processing_time * 1000.0
                            )
                        }
                    ]
                }))
            }
        },
    );

    // Register get_query_log
    registry.register(
        "get_query_log",
        "Search AdGuard Home query log",
        serde_json::json!({
            "type": "object",
            "properties": {
                "search": {
                    "type": "string",
                    "description": "Filter by domain name"
                },
                "filter": {
                    "type": "string",
                    "description": "Filter by status",
                    "enum": ["all", "blocked", "allowed"]
                },
                "limit": {
                    "type": "integer",
                    "description": "Max entries to return (default 50, max 100)",
                    "minimum": 1,
                    "maximum": 100
                }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let search = params["search"].as_str().map(|s| s.to_string());
            let filter = params["filter"].as_str().map(|s| s.to_string());
            let limit = params["limit"].as_u64().map(|l| l as u32);
            async move {
                let log = client
                    .get_query_log(search.as_deref(), filter.as_deref(), limit)
                    .await?;
                let mut text = String::new();
                for entry in log.data {
                    text.push_str(&format!(
                        "[{}] {} -> {} ({}, {}ms)\n",
                        entry.time,
                        entry.question.name,
                        entry.status,
                        entry.reason,
                        entry.elapsed_ms
                    ));
                }
                if text.is_empty() {
                    text = "No query log entries found".to_string();
                }
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": text
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
