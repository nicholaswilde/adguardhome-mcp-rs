use adguardhome_mcp_rs::adguard::{AdGuardClient, AdGuardClientDevice, DnsRewrite};
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

    // Register set_protection_state
    registry.register(
        "set_protection_state",
        "Enable or disable global AdGuard Home protection",
        serde_json::json!({
            "type": "object",
            "properties": {
                "enabled": {
                    "type": "boolean",
                    "description": "True to enable protection, false to disable"
                }
            },
            "required": ["enabled"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let enabled = params["enabled"].as_bool().unwrap_or(true);
            async move {
                client.set_protection(enabled).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Global protection {}", if enabled { "enabled" } else { "disabled" })
                        }
                    ]
                }))
            }
        },
    );

    // Register set_safe_search
    registry.register(
        "set_safe_search",
        "Enable or disable enforced safe search",
        serde_json::json!({
            "type": "object",
            "properties": {
                "enabled": {
                    "type": "boolean",
                    "description": "True to enable safe search, false to disable"
                }
            },
            "required": ["enabled"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let enabled = params["enabled"].as_bool().unwrap_or(true);
            async move {
                client.set_safe_search(enabled).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Safe search {}", if enabled { "enabled" } else { "disabled" })
                        }
                    ]
                }))
            }
        },
    );

    // Register set_safe_browsing
    registry.register(
        "set_safe_browsing",
        "Enable or disable safe browsing (protection against malicious domains)",
        serde_json::json!({
            "type": "object",
            "properties": {
                "enabled": {
                    "type": "boolean",
                    "description": "True to enable safe browsing, false to disable"
                }
            },
            "required": ["enabled"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let enabled = params["enabled"].as_bool().unwrap_or(true);
            async move {
                client.set_safe_browsing(enabled).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Safe browsing {}", if enabled { "enabled" } else { "disabled" })
                        }
                    ]
                }))
            }
        },
    );

    // Register set_parental_control
    registry.register(
        "set_parental_control",
        "Enable or disable parental control (filtering of adult content)",
        serde_json::json!({
            "type": "object",
            "properties": {
                "enabled": {
                    "type": "boolean",
                    "description": "True to enable parental control, false to disable"
                }
            },
            "required": ["enabled"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let enabled = params["enabled"].as_bool().unwrap_or(true);
            async move {
                client.set_parental_control(enabled).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Parental control {}", if enabled { "enabled" } else { "disabled" })
                        }
                    ]
                }))
            }
        },
    );

    // Register list_filter_lists
    registry.register(
        "list_filter_lists",
        "List all configured filter lists",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let config = client.list_filters().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&config)?
                        }
                    ]
                }))
            }
        },
    );

    // Register toggle_filter_list
    registry.register(
        "toggle_filter_list",
        "Enable or disable a filter list by Name, ID, or URL",
        serde_json::json!({
            "type": "object",
            "properties": {
                "identifier": {
                    "type": "string",
                    "description": "Name, ID, or URL of the filter list"
                },
                "enabled": {
                    "type": "boolean",
                    "description": "True to enable, false to disable"
                }
            },
            "required": ["identifier", "enabled"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let identifier = params["identifier"].as_str().unwrap_or_default().to_string();
            let enabled = params["enabled"].as_bool().unwrap_or(true);
            async move {
                let config = client.list_filters().await?;
                let filter = config
                    .filters
                    .iter()
                    .chain(config.whitelist_filters.iter())
                    .find(|f| {
                        f.name == identifier || f.url == identifier || f.id.to_string() == identifier
                    });

                if let Some(f) = filter {
                    client
                        .toggle_filter(f.url.clone(), f.name.clone(), enabled)
                        .await?;
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Filter '{}' {}", f.name, if enabled { "enabled" } else { "disabled" })
                            }
                        ]
                    }))
                } else {
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Filter '{}' not found", identifier)
                            }
                        ],
                        "isError": true
                    }))
                }
            }
        },
    );

    // Register add_filter_list
    registry.register(
        "add_filter_list",
        "Add a new filter list",
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the filter list"
                },
                "url": {
                    "type": "string",
                    "description": "URL of the filter list"
                },
                "whitelist": {
                    "type": "boolean",
                    "description": "True if this is an allowlist (whitelist), false if blocklist",
                    "default": false
                }
            },
            "required": ["name", "url"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let name = params["name"].as_str().unwrap_or_default().to_string();
            let url = params["url"].as_str().unwrap_or_default().to_string();
            let whitelist = params["whitelist"].as_bool().unwrap_or(false);
            async move {
                client.add_filter(name.clone(), url, whitelist).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Filter '{}' added successfully", name)
                        }
                    ]
                }))
            }
        },
    );

    // Register list_clients
    registry.register(
        "list_clients",
        "List all configured and discovered clients",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let clients = client.list_clients().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&clients)?
                        }
                    ]
                }))
            }
        },
    );

    // Register get_client_info
    registry.register(
        "get_client_info",
        "Get detailed information for a specific client",
        serde_json::json!({
            "type": "object",
            "properties": {
                "identifier": {
                    "type": "string",
                    "description": "IP, MAC, or Name of the client"
                }
            },
            "required": ["identifier"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let identifier = params["identifier"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            async move {
                let client_info = client.get_client_info(&identifier).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&client_info)?
                        }
                    ]
                }))
            }
        },
    );

    // Register list_custom_rules
    registry.register(
        "list_custom_rules",
        "List all user-defined DNS filtering rules",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let rules = client.get_user_rules().await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": rules.join("\n")
                        }
                    ]
                }))
            }
        },
    );

    // Register set_custom_rules
    registry.register(
        "set_custom_rules",
        "Replace all custom filtering rules",
        serde_json::json!({
            "type": "object",
            "properties": {
                "rules": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of filtering rules"
                }
            },
            "required": ["rules"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let rules = params["rules"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            async move {
                client.set_user_rules(rules).await?;
                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Custom rules updated successfully"
                        }
                    ]
                }))
            }
        },
    );

    // Register add_custom_rule
    registry.register(
        "add_custom_rule",
        "Add a single custom filtering rule",
        serde_json::json!({
            "type": "object",
            "properties": {
                "rule": {
                    "type": "string",
                    "description": "The filtering rule to add"
                }
            },
            "required": ["rule"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let rule = params["rule"].as_str().unwrap_or_default().to_string();
            async move {
                let mut rules = client.get_user_rules().await?;
                if !rules.contains(&rule) {
                    rules.push(rule);
                    client.set_user_rules(rules).await?;
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": "Rule added successfully"
                            }
                        ]
                    }))
                } else {
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": "Rule already exists"
                            }
                        ]
                    }))
                }
            }
        },
    );

    // Register remove_custom_rule
    registry.register(
        "remove_custom_rule",
        "Remove a single custom filtering rule",
        serde_json::json!({
            "type": "object",
            "properties": {
                "rule": {
                    "type": "string",
                    "description": "The filtering rule to remove"
                }
            },
            "required": ["rule"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let rule = params["rule"].as_str().unwrap_or_default().to_string();
            async move {
                let mut rules = client.get_user_rules().await?;
                if let Some(pos) = rules.iter().position(|r| r == &rule) {
                    rules.remove(pos);
                    client.set_user_rules(rules).await?;
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": "Rule removed successfully"
                            }
                        ]
                    }))
                } else {
                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": "Rule not found"
                            }
                        ],
                        "isError": true
                    }))
                }
            }
        },
    );

    // Register list_blocked_services
    registry.register(
        "list_blocked_services",
        "List all available services and their current blocked status",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let all_services = client.list_all_services().await?;
                let blocked_ids = client.list_blocked_services().await?;

                let mut results = Vec::new();
                for service in all_services {
                    let is_blocked = blocked_ids.contains(&service.id);
                    results.push(serde_json::json!({
                        "id": service.id,
                        "name": service.name,
                        "blocked": is_blocked
                    }));
                }

                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&results)?
                        }
                    ]
                }))
            }
        },
    );

    // Register toggle_blocked_service
    registry.register(
        "toggle_blocked_service",
        "Enable or disable blocking for a specific service",
        serde_json::json!({
            "type": "object",
            "properties": {
                "service_id": {
                    "type": "string",
                    "description": "The ID of the service (e.g., 'youtube', 'facebook')"
                },
                "blocked": {
                    "type": "boolean",
                    "description": "True to block the service, false to unblock"
                }
            },
            "required": ["service_id", "blocked"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let service_id = params["service_id"].as_str().unwrap_or_default().to_string();
            let blocked = params["blocked"].as_bool().unwrap_or(true);
            async move {
                let mut blocked_ids = client.list_blocked_services().await?;
                let already_blocked = blocked_ids.contains(&service_id);

                if blocked && !already_blocked {
                    blocked_ids.push(service_id.clone());
                    client.set_blocked_services(blocked_ids).await?;
                } else if !blocked && already_blocked {
                    blocked_ids.retain(|id| id != &service_id);
                    client.set_blocked_services(blocked_ids).await?;
                }

                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Service '{}' {}", service_id, if blocked { "blocked" } else { "unblocked" })
                        }
                    ]
                }))
            }
        },
    );

    // Register add_client
    registry.register(
        "add_client",
        "Add a new client configuration",
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Display name of the client" },
                "ids": { "type": "array", "items": { "type": "string" }, "description": "List of IP addresses or MAC addresses" },
                "use_global_settings": { "type": "boolean", "default": true },
                "filtering_enabled": { "type": "boolean", "default": true },
                "parental_enabled": { "type": "boolean", "default": false },
                "safebrowsing_enabled": { "type": "boolean", "default": true },
                "safesearch_enabled": { "type": "boolean", "default": false }
            },
            "required": ["name", "ids"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let device = AdGuardClientDevice {
                name: params["name"].as_str().unwrap_or_default().to_string(),
                ids: params["ids"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                use_global_settings: params["use_global_settings"].as_bool().unwrap_or(true),
                filtering_enabled: params["filtering_enabled"].as_bool().unwrap_or(true),
                parental_enabled: params["parental_enabled"].as_bool().unwrap_or(false),
                safebrowsing_enabled: params["safebrowsing_enabled"].as_bool().unwrap_or(true),
                safesearch_enabled: params["safesearch_enabled"].as_bool().unwrap_or(false),
            };
            async move {
                client.add_client(device).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Client added successfully" }]
                }))
            }
        },
    );

    // Register update_client
    registry.register(
        "update_client",
        "Update an existing client configuration",
        serde_json::json!({
            "type": "object",
            "properties": {
                "old_name": { "type": "string", "description": "The current name of the client to update" },
                "name": { "type": "string", "description": "New display name of the client" },
                "ids": { "type": "array", "items": { "type": "string" }, "description": "New list of IP addresses or MAC addresses" },
                "use_global_settings": { "type": "boolean" },
                "filtering_enabled": { "type": "boolean" },
                "parental_enabled": { "type": "boolean" },
                "safebrowsing_enabled": { "type": "boolean" },
                "safesearch_enabled": { "type": "boolean" }
            },
            "required": ["old_name"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let old_name = params["old_name"].as_str().unwrap_or_default().to_string();

            async move {
                // Get current info to fill in defaults if not provided?
                // Or just use the provided values.
                let current = client.get_client_info(&old_name).await?;
                let device = AdGuardClientDevice {
                    name: params["name"].as_str().map(|s| s.to_string()).unwrap_or(current.name),
                    ids: params["ids"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or(current.ids),
                    use_global_settings: params["use_global_settings"].as_bool().unwrap_or(current.use_global_settings),
                    filtering_enabled: params["filtering_enabled"].as_bool().unwrap_or(current.filtering_enabled),
                    parental_enabled: params["parental_enabled"].as_bool().unwrap_or(current.parental_enabled),
                    safebrowsing_enabled: params["safebrowsing_enabled"].as_bool().unwrap_or(current.safebrowsing_enabled),
                    safesearch_enabled: params["safesearch_enabled"].as_bool().unwrap_or(current.safesearch_enabled),
                };
                client.update_client(old_name, device).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Client updated successfully" }]
                }))
            }
        },
    );

    // Register delete_client
    registry.register(
        "delete_client",
        "Remove a client configuration",
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Name of the client to remove" }
            },
            "required": ["name"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let name = params["name"].as_str().unwrap_or_default().to_string();
            async move {
                client.delete_client(name).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Client removed successfully" }]
                }))
            }
        },
    );

    // Register list_dhcp_leases
    registry.register(
        "list_dhcp_leases",
        "List all dynamic and static DHCP leases",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let status = client.get_dhcp_status().await?;
                let mut text = format!(
                    "DHCP Enabled: {}\nInterface: {}\n\nDynamic Leases:\n",
                    status.enabled, status.interface_name
                );
                for lease in status.leases {
                    text.push_str(&format!(
                        "- {} ({}) [{}] Expires: {}\n",
                        lease.hostname,
                        lease.ip,
                        lease.mac,
                        lease.expires.as_deref().unwrap_or("Never")
                    ));
                }
                text.push_str("\nStatic Leases:\n");
                for lease in status.static_leases {
                    text.push_str(&format!(
                        "- {} ({}) [{}]\n",
                        lease.hostname, lease.ip, lease.mac
                    ));
                }
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": text }]
                }))
            }
        },
    );

    // Register add_static_lease
    registry.register(
        "add_static_lease",
        "Add a new static DHCP lease",
        serde_json::json!({
            "type": "object",
            "properties": {
                "mac": { "type": "string", "description": "MAC address" },
                "ip": { "type": "string", "description": "IP address" },
                "hostname": { "type": "string", "description": "Hostname" }
            },
            "required": ["mac", "ip", "hostname"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let lease = adguardhome_mcp_rs::adguard::StaticLease {
                mac: params["mac"].as_str().unwrap_or_default().to_string(),
                ip: params["ip"].as_str().unwrap_or_default().to_string(),
                hostname: params["hostname"].as_str().unwrap_or_default().to_string(),
            };
            async move {
                client.add_static_lease(lease).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Static lease added successfully" }]
                }))
            }
        },
    );

    // Register remove_static_lease
    registry.register(
        "remove_static_lease",
        "Remove a static DHCP lease",
        serde_json::json!({
            "type": "object",
            "properties": {
                "mac": { "type": "string", "description": "MAC address" },
                "ip": { "type": "string", "description": "IP address" },
                "hostname": { "type": "string", "description": "Hostname" }
            },
            "required": ["mac", "ip", "hostname"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let lease = adguardhome_mcp_rs::adguard::StaticLease {
                mac: params["mac"].as_str().unwrap_or_default().to_string(),
                ip: params["ip"].as_str().unwrap_or_default().to_string(),
                hostname: params["hostname"].as_str().unwrap_or_default().to_string(),
            };
            async move {
                client.remove_static_lease(lease).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Static lease removed successfully" }]
                }))
            }
        },
    );

    // Register get_dns_config
    registry.register(
        "get_dns_config",
        "Retrieve current DNS settings including upstream servers and cache configuration",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let config = client.get_dns_info().await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&config)? }]
                }))
            }
        },
    );

    // Register set_dns_config
    registry.register(
        "set_dns_config",
        "Update DNS settings including upstream servers and cache configuration",
        serde_json::json!({
            "type": "object",
            "properties": {
                "upstream_dns": { "type": "array", "items": { "type": "string" }, "description": "List of upstream DNS servers" },
                "bootstrap_dns": { "type": "array", "items": { "type": "string" }, "description": "List of bootstrap DNS servers" },
                "fallback_dns": { "type": "array", "items": { "type": "string" }, "description": "List of fallback DNS servers" },
                "all_servers": { "type": "boolean", "description": "If true, parallel queries to all configured upstream servers are enabled" },
                "fastest_addr": { "type": "boolean", "description": "Use fastest address detection" },
                "cache_size": { "type": "integer", "description": "DNS cache size (in bytes)" },
                "cache_ttl_min": { "type": "integer", "description": "Minimum TTL for DNS cache" },
                "cache_ttl_max": { "type": "integer", "description": "Maximum TTL for DNS cache" },
                "cache_optimistic": { "type": "boolean", "description": "Enable optimistic caching" }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            async move {
                let mut config = client.get_dns_info().await?;
                if let Some(upstream) = params["upstream_dns"].as_array() {
                    config.upstream_dns = upstream.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(bootstrap) = params["bootstrap_dns"].as_array() {
                    config.bootstrap_dns = bootstrap.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(fallback) = params["fallback_dns"].as_array() {
                    config.fallback_dns = fallback.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(all_servers) = params["all_servers"].as_bool() {
                    config.all_servers = all_servers;
                }
                if let Some(fastest_addr) = params["fastest_addr"].as_bool() {
                    config.fastest_addr = fastest_addr;
                }
                if let Some(cache_size) = params["cache_size"].as_u64() {
                    config.cache_size = cache_size as u32;
                }
                if let Some(cache_ttl_min) = params["cache_ttl_min"].as_u64() {
                    config.cache_ttl_min = cache_ttl_min as u32;
                }
                if let Some(cache_ttl_max) = params["cache_ttl_max"].as_u64() {
                    config.cache_ttl_max = cache_ttl_max as u32;
                }
                if let Some(cache_optimistic) = params["cache_optimistic"].as_bool() {
                    config.cache_optimistic = cache_optimistic;
                }

                client.set_dns_config(config).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "DNS configuration updated successfully" }]
                }))
            }
        },
    );

    // Register clear_dns_cache
    registry.register(
        "clear_dns_cache",
        "Flush the DNS cache",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                client.clear_dns_cache().await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "DNS cache cleared successfully" }]
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

    // Register get_client_activity_report
    registry.register(
        "get_client_activity_report",
        "Summarize recent activity for a specific client",
        serde_json::json!({
            "type": "object",
            "properties": {
                "identifier": {
                    "type": "string",
                    "description": "IP, MAC, or Name of the client"
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of recent queries to analyze (default 50)",
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["identifier"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let identifier = params["identifier"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let limit = params["limit"].as_u64().map(|l| l as u32).unwrap_or(50);
            async move {
                let log = client
                    .get_query_log(Some(&identifier), None, Some(limit))
                    .await?;

                let mut total = 0;
                let mut blocked = 0;
                let mut domains = std::collections::HashMap::new();

                for entry in &log.data {
                    total += 1;
                    if entry.reason != "NotFilteredNotFound"
                        && !entry.reason.is_empty()
                        && entry.status != "NOERROR"
                    {
                        // This is a rough heuristic for "blocked" or "filtered"
                        // AdGuard status/reason can be complex.
                        if entry.status != "NOERROR"
                            || entry.reason.contains("Filtered")
                            || entry.reason.contains("Block")
                        {
                            blocked += 1;
                        }
                    } else if entry.status == "NXDOMAIN" {
                        // NXDOMAIN might also be interesting
                    }

                    *domains.entry(entry.question.name.clone()).or_insert(0) += 1;
                }

                let mut top_domains: Vec<_> = domains.into_iter().collect();
                top_domains.sort_by(|a, b| b.1.cmp(&a.1));
                top_domains.truncate(5);

                let mut text = format!("Activity Report for {}\n", identifier);
                text.push_str(&format!("Recent Queries Analyzed: {}\n", total));
                text.push_str(&format!("Blocked/Filtered: {}\n\n", blocked));
                text.push_str("Top Recently Accessed Domains:\n");
                for (domain, count) in top_domains {
                    text.push_str(&format!("- {}: {}\n", domain, count));
                }

                if total == 0 {
                    text = format!("No recent activity found for client: {}", identifier);
                }

                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": text }]
                }))
            }
        },
    );

    // Register get_access_list
    registry.register(
        "get_access_list",
        "Get the global access control lists (allowed/disallowed clients and blocked hosts)",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let list = client.get_access_list().await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&list)? }]
                }))
            }
        },
    );

    // Register update_access_list
    registry.register(
        "update_access_list",
        "Update the global access control lists",
        serde_json::json!({
            "type": "object",
            "properties": {
                "allowed_clients": { "type": "array", "items": { "type": "string" }, "description": "Clients allowed to use DNS" },
                "disallowed_clients": { "type": "array", "items": { "type": "string" }, "description": "Clients forbidden from using DNS" },
                "blocked_hosts": { "type": "array", "items": { "type": "string" }, "description": "Globally blocked hostnames/IPs" }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            async move {
                let mut list = client.get_access_list().await?;
                if let Some(allowed) = params["allowed_clients"].as_array() {
                    list.allowed_clients = allowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(disallowed) = params["disallowed_clients"].as_array() {
                    list.disallowed_clients = disallowed.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(blocked) = params["blocked_hosts"].as_array() {
                    list.blocked_hosts = blocked.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }

                client.set_access_list(list).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Access control list updated successfully" }]
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
