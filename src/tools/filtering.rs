use super::ToolRegistry;

pub fn register(registry: &mut ToolRegistry) {
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

    // Register remove_filter_list
    registry.register(
        "remove_filter_list",
        "Remove an existing filter list",
        serde_json::json!({
            "type": "object",
            "properties": {
                "identifier": {
                    "type": "string",
                    "description": "Name, ID, or URL of the filter list to remove"
                },
                "whitelist": {
                    "type": "boolean",
                    "description": "True if this is an allowlist (whitelist), false if blocklist (optional, tries to auto-detect if omitted but safer to specify)",
                    "default": false
                }
            },
            "required": ["identifier"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let identifier = params["identifier"].as_str().unwrap_or_default().to_string();

            async move {
                let config = client.list_filters().await?;

                let filter_block = config.filters.iter().find(|f| {
                    f.name == identifier || f.url == identifier || f.id.to_string() == identifier
                });

                let filter_white = config.whitelist_filters.iter().find(|f| {
                    f.name == identifier || f.url == identifier || f.id.to_string() == identifier
                });

                let (target_url, is_whitelist) = if let Some(f) = filter_block {
                    (f.url.clone(), false)
                } else if let Some(f) = filter_white {
                    (f.url.clone(), true)
                } else {
                    return Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Filter '{}' not found", identifier)
                            }
                        ],
                        "isError": true
                    }));
                };

                client.remove_filter(target_url, is_whitelist).await?;

                Ok(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Filter '{}' removed successfully", identifier)
                        }
                    ]
                }))
            }
        },
    );

    // Register update_filter_list
    registry.register(
        "update_filter_list",
        "Update the name, URL, or enabled state of a filter list",
        serde_json::json!({
            "type": "object",
            "properties": {
                "identifier": {
                    "type": "string",
                    "description": "Name, ID, or URL of the filter list to update"
                },
                "new_name": {
                    "type": "string",
                    "description": "New name for the filter list"
                },
                "new_url": {
                    "type": "string",
                    "description": "New URL for the filter list"
                },
                "enabled": {
                    "type": "boolean",
                    "description": "Enable or disable the filter list"
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
            let new_name = params["new_name"].as_str().map(|s| s.to_string());
            let new_url = params["new_url"].as_str().map(|s| s.to_string());
            let enabled_opt = params["enabled"].as_bool();

            async move {
                let config = client.list_filters().await?;

                let filter_block = config.filters.iter().find(|f| {
                    f.name == identifier || f.url == identifier || f.id.to_string() == identifier
                });

                let filter_white = config.whitelist_filters.iter().find(|f| {
                    f.name == identifier || f.url == identifier || f.id.to_string() == identifier
                });

                if let Some(f) = filter_block.or(filter_white) {
                    let is_whitelist = filter_white.is_some();
                    let name_to_use = new_name.unwrap_or_else(|| f.name.clone());
                    let url_to_use = new_url.unwrap_or_else(|| f.url.clone());
                    let enabled_to_use = enabled_opt.unwrap_or(f.enabled);

                    client
                        .update_filter(
                            f.url.clone(),
                            url_to_use,
                            name_to_use.clone(),
                            is_whitelist,
                            enabled_to_use,
                        )
                        .await?;

                    Ok(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Filter '{}' updated successfully", f.name)
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

    // Register check_filtering
    registry.register(
        "check_filtering",
        "Check how a domain is filtered by AdGuard Home, identifying specific rules or lists that affect it.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "The domain name to check" },
                "client": { "type": "string", "description": "Optional IP or Name of the client checking the domain" }
            },
            "required": ["domain"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let domain = params["domain"].as_str().unwrap_or_default().to_string();
            let client_id = params["client"].as_str().map(|s| s.to_string());
            async move {
                let result = client.check_host(&domain, client_id.as_deref()).await?;
                let mut text = format!("Filtering status for {}:\n", domain);
                text.push_str(&format!("Result: {}\n", result.reason));
                if let Some(rule) = result.rule {
                    text.push_str(&format!("Matched Rule: {}\n", rule));
                }
                if let Some(filter_id) = result.filter_id {
                    text.push_str(&format!("Filter ID: {}\n", filter_id));
                }
                if let Some(rules) = result.rules.filter(|r| !r.is_empty()) {
                    text.push_str("\nAll Matched Rules:\n");
                    for r in rules {
                        text.push_str(&format!("- {} (Filter {})\n", r.text, r.filter_id));
                    }
                }
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": text }]
                }))
            }
        },
    );
}
