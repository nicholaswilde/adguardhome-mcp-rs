use super::ToolRegistry;
use crate::sync::SyncState;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        "manage_system",
        "Manage AdGuard Home system (status, stats, logs, backups, updates, service control).",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "System action to perform",
                    "enum": [
                        "get_status", "get_stats", "clear_stats", "get_query_log",
                        "clear_query_log", "get_top_blocked_domains", "get_query_log_config",
                        "set_query_log_config", "get_version_info", "update_adguard_home",
                        "create_backup", "restore_backup", "restart_service"
                    ]
                },
                "time_period": { "type": "string", "enum": ["24h", "7d", "30d"], "description": "For stats" },
                "search": { "type": "string", "description": "Filter log by domain" },
                "filter": { "type": "string", "enum": ["all", "blocked", "allowed"], "description": "Filter log by status" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Max log entries" },
                "enabled": { "type": "boolean", "description": "For query log config" },
                "interval": { "type": "integer", "minimum": 1, "description": "Log retention hours" },
                "anonymize_client_ip": { "type": "boolean" },
                "allowed_clients": { "type": "array", "items": { "type": "string" } },
                "disallowed_clients": { "type": "array", "items": { "type": "string" } },
                "file_path": { "type": "string", "description": "For restore_backup" },
                "description": { "type": "string", "description": "Optional description for the backup" }
            },
            "required": ["action"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let action = params["action"].as_str().unwrap_or_default().to_string();

            async move {
                match action.as_str() {
                    "get_status" => {
                        let status = client.get_status().await?;
                        Ok(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Version: {}\nProtection: {}", status.version, status.protection_enabled)
                            }]
                        }))
                    }
                    "get_stats" => {
                        let period = params["time_period"].as_str();
                        let stats = client.get_stats(period).await?;
                        let blocked_pct = if stats.num_dns_queries > 0 {
                            (stats.num_blocked_filtering as f64 / stats.num_dns_queries as f64) * 100.0
                        } else {
                            0.0
                        };
                        Ok(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!(
                                    "Queries: {}\nBlocked: {} ({:.2}%)\nMalware: {}\nSafe Search: {}\nParental: {}\nAvg Time: {:.2}ms",
                                    stats.num_dns_queries, stats.num_blocked_filtering, blocked_pct,
                                    stats.num_replaced_safebrowsing, stats.num_replaced_safesearch,
                                    stats.num_replaced_parental, stats.avg_processing_time * 1000.0
                                )
                            }]
                        }))
                    }
                    "clear_stats" => {
                        client.reset_stats().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Stats cleared" }] }))
                    }
                    "get_query_log" => {
                        let search = params["search"].as_str();
                        let filter = params["filter"].as_str();
                        let limit = params["limit"].as_u64().map(|l| l as u32);
                        let log = client.get_query_log(search, filter, limit).await?;
                        let mut text = String::new();
                        for entry in log.data {
                            text.push_str(&format!(
                                "[{}] {} -> {} ({}, {}ms)\n",
                                entry.time, entry.question.name, entry.status, entry.reason, entry.elapsed_ms
                            ));
                        }
                        if text.is_empty() { text = "No entries found".to_string(); }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": text }] }))
                    }
                    "clear_query_log" => {
                        client.clear_query_log().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Query log cleared" }] }))
                    }
                    "get_top_blocked_domains" => {
                        let period = params["time_period"].as_str();
                        let stats = client.get_stats(period).await?;
                        let mut text = "Top Blocked Domains:\n".to_string();
                        if stats.top_blocked_domains.is_empty() {
                            text.push_str("None found.\n");
                        } else {
                            for entry in stats.top_blocked_domains {
                                for (domain, count) in entry {
                                    text.push_str(&format!("- {}: {}\n", domain, count));
                                }
                            }
                        }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": text }] }))
                    }
                    "get_query_log_config" => {
                        let config = client.get_query_log_config().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&config)? }] }))
                    }
                    "set_query_log_config" => {
                        let mut config = client.get_query_log_config().await?;
                        if let Some(e) = params["enabled"].as_bool() { config.enabled = e; }
                        if let Some(i) = params["interval"].as_u64() { config.interval = i; }
                        if let Some(a) = params["anonymize_client_ip"].as_bool() { config.anonymize_client_ip = a; }
                        if let Some(allowed) = params["allowed_clients"].as_array() {
                            config.allowed_clients = allowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        }
                        if let Some(disallowed) = params["disallowed_clients"].as_array() {
                            config.disallowed_clients = disallowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        }
                        client.set_query_log_config(config).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Config updated" }] }))
                    }
                    "get_version_info" => {
                        let info = client.get_version_info().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&info)? }] }))
                    }
                    "update_adguard_home" => {
                        client.update_adguard_home().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Update triggered" }] }))
                    }
                    "create_backup" => {
                        let description = params["description"].as_str().map(|s| s.to_string());
                        let state = SyncState::fetch_full(&client, description).await.map_err(|e| crate::error::Error::Generic(e.to_string()))?;
                        let json = serde_json::to_vec_pretty(&state)?;

                        let backup_dir = PathBuf::from("backups");
                        if !backup_dir.exists() {
                            fs::create_dir_all(&backup_dir).await?;
                        }

                        let file_name = format!("adguard_backup_{}.json", uuid::Uuid::new_v4());
                        let file_path = backup_dir.join(file_name);

                        let mut file = fs::File::create(&file_path).await?;
                        file.write_all(&json).await?;
                        file.sync_all().await?;

                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Backup: {}", file_path.display()) }] }))
                    }
                    "restore_backup" => {
                        let path = params["file_path"].as_str().unwrap_or_default();
                        let json = fs::read(path).await?;
                        let state: SyncState = serde_json::from_slice(&json)?;

                        // Version safety check
                        if let Some(ref metadata) = state.metadata {
                            let target_status = client.get_status().await?;
                            let backup_ver_str = metadata.version.trim_start_matches('v');
                            let target_ver_str = target_status.version.trim_start_matches('v');

                            if let (Ok(b_ver), Ok(t_ver)) = (semver::Version::parse(backup_ver_str), semver::Version::parse(target_ver_str)) {
                                if b_ver.major != t_ver.major {
                                    return Err(crate::error::Error::Generic(format!(
                                        "Major version mismatch: Backup is {}, Target is {}. Restoration aborted for safety.",
                                        metadata.version, target_status.version
                                    )));
                                }
                                if b_ver.minor != t_ver.minor {
                                    tracing::warn!("Minor version mismatch: Backup is {}, Target is {}. Proceeding with caution.", metadata.version, target_status.version);
                                }
                            }
                        }

                        state.push_to_replica(&client, "full-overwrite").await.map_err(|e| crate::error::Error::Generic(e.to_string()))?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Backup restored" }] }))
                    }
                    "restart_service" => {
                        client.restart_service().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Restart sent" }] }))
                    }
                    _ => Err(crate::error::Error::Mcp(crate::mcp::ResponseError {
                        code: -32602,
                        message: format!("Unknown action: {}", action),
                        data: None,
                    })),
                }
            }
        },
    );
}
