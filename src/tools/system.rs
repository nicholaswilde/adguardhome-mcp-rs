use super::ToolRegistry;

pub fn register(registry: &mut ToolRegistry) {
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

    // Register clear_stats
    registry.register(
        "clear_stats",
        "Reset all statistics",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                client.reset_stats().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Statistics cleared successfully"
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
                        "[ {}] {} -> {} ({}, {}ms)\n",
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

    // Register clear_query_log
    registry.register(
        "clear_query_log",
        "Clear the DNS query log",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                client.clear_query_log().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Query log cleared successfully"
                        }
                    ]
                }))
            }
        },
    );

    // Register get_top_blocked_domains
    registry.register(
        "get_top_blocked_domains",
        "List the most frequently blocked domains",
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
                let mut text = "Top Blocked Domains:\n".to_string();
                if stats.top_blocked_domains.is_empty() {
                    text.push_str("No blocked domains found in this period.\n");
                } else {
                    for entry in stats.top_blocked_domains {
                        for (domain, count) in entry {
                            text.push_str(&format!("- {}: {}\n", domain, count));
                        }
                    }
                }
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": text }]
                }))
            }
        },
    );

    // Register get_query_log_config
    registry.register(
        "get_query_log_config",
        "Retrieve current DNS query logging settings",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let config = client.get_query_log_config().await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&config)? }]
                }))
            }
        },
    );

    // Register set_query_log_config
    registry.register(
        "set_query_log_config",
        "Update DNS query logging settings",
        serde_json::json!({
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "description": "Enable or disable query logging" },
                "interval": { "type": "integer", "description": "Retention interval in hours", "minimum": 1 },
                "anonymize_client_ip": { "type": "boolean", "description": "Anonymize client IP addresses" },
                "allowed_clients": { "type": "array", "items": { "type": "string" }, "description": "List of clients allowed to be logged" },
                "disallowed_clients": { "type": "array", "items": { "type": "string" }, "description": "List of clients forbidden from being logged" }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            async move {
                let mut config = client.get_query_log_config().await?;
                if let Some(enabled) = params["enabled"].as_bool() { config.enabled = enabled; }
                if let Some(interval) = params["interval"].as_u64() { config.interval = interval as u32; }
                if let Some(anonymize) = params["anonymize_client_ip"].as_bool() { config.anonymize_client_ip = anonymize; }
                if let Some(allowed) = params["allowed_clients"].as_array() {
                    config.allowed_clients = allowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(disallowed) = params["disallowed_clients"].as_array() {
                    config.disallowed_clients = disallowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }

                client.set_query_log_config(config).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Query log configuration updated successfully" }]
                }))
            }
        },
    );

    // Register get_version_info
    registry.register(
        "get_version_info",
        "Get AdGuard Home version information and check for updates",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let info = client.get_version_info().await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&info)? }]
                }))
            }
        },
    );

    // Register update_adguard_home
    registry.register(
        "update_adguard_home",
        "Trigger an update of AdGuard Home",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                client.update_adguard_home().await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Update triggered successfully" }]
                }))
            }
        },
    );

    // Register create_backup
    registry.register(
        "create_backup",
        "Create a full backup of the AdGuard Home configuration",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let path = client.create_backup().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Backup created successfully at: {}", path.display())
                        }
                    ]
                }))
            }
        },
    );

    // Register restore_backup
    registry.register(
        "restore_backup",
        "Restore AdGuard Home configuration from a backup file",
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the backup file (.tar.gz)"
                }
            },
            "required": ["file_path"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let file_path = params["file_path"].as_str().unwrap_or_default().to_string();
            async move {
                client.restore_backup(&file_path).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Backup restored successfully. The server may restart."
                        }
                    ]
                }))
            }
        },
    );

    // Register restart_service
    registry.register(
        "restart_service",
        "Restart the AdGuard Home service",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                client.restart_service().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Restart command sent successfully. The server will be unavailable for a moment."
                        }
                    ]
                }))
            }
        },
    );
}
