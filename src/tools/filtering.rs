use super::ToolRegistry;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        "manage_filtering",
        "Manage filtering (lists, custom rules, blocked services, debugger).",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Filtering action to perform",
                    "enum": [
                        "list_filters", "add_filter", "remove_filter", "update_filter", "toggle_filter",
                        "list_custom_rules", "set_custom_rules", "add_custom_rule", "remove_custom_rule",
                        "list_blocked_services", "toggle_blocked_service", "check_host"
                    ]
                },
                "identifier": { "type": "string", "description": "Filter list Name, ID, or URL" },
                "name": { "type": "string", "description": "Filter list name" },
                "url": { "type": "string", "description": "Filter list URL" },
                "new_name": { "type": "string" },
                "new_url": { "type": "string" },
                "whitelist": { "type": "boolean", "default": false },
                "enabled": { "type": "boolean" },
                "rule": { "type": "string", "description": "Custom rule text" },
                "rules": { "type": "array", "items": { "type": "string" } },
                "service_id": { "type": "string", "description": "Service ID (e.g., 'youtube')" },
                "blocked": { "type": "boolean" },
                "domain": { "type": "string" },
                "client": { "type": "string", "description": "Optional client IP/Name" }
            },
            "required": ["action"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let action = params["action"].as_str().unwrap_or_default().to_string();

            async move {
                match action.as_str() {
                    "list_filters" => {
                        let config = client.list_filters().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&config)? }] }))
                    }
                    "add_filter" => {
                        let name = params["name"].as_str().unwrap_or_default().to_string();
                        let url = params["url"].as_str().unwrap_or_default().to_string();
                        let white = params["whitelist"].as_bool().unwrap_or(false);
                        client.add_filter(name, url, white).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Filter added" }] }))
                    }
                    "remove_filter" => {
                        let id = params["identifier"].as_str().unwrap_or_default();
                        let config = client.list_filters().await?;
                        let filter = config.filters.iter().chain(config.whitelist_filters.iter())
                            .find(|f| f.name == id || f.url == id || f.id.to_string() == id);
                        if let Some(f) = filter {
                            let is_white = config.whitelist_filters.iter().any(|wf| wf.url == f.url);
                            client.remove_filter(f.url.clone(), is_white).await?;
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Filter removed" }] }))
                        } else {
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Filter not found" }], "isError": true }))
                        }
                    }
                    "update_filter" => {
                        let id = params["identifier"].as_str().unwrap_or_default();
                        let config = client.list_filters().await?;
                        let filter = config.filters.iter().chain(config.whitelist_filters.iter())
                            .find(|f| f.name == id || f.url == id || f.id.to_string() == id);
                        if let Some(f) = filter {
                            let is_white = config.whitelist_filters.iter().any(|wf| wf.url == f.url);
                            let n_name = params["new_name"].as_str().map(|s| s.to_string()).unwrap_or(f.name.clone());
                            let n_url = params["new_url"].as_str().map(|s| s.to_string()).unwrap_or(f.url.clone());
                            let n_en = params["enabled"].as_bool().unwrap_or(f.enabled);
                            client.update_filter(f.url.clone(), n_url, n_name, is_white, n_en).await?;
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Filter updated" }] }))
                        } else {
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Filter not found" }], "isError": true }))
                        }
                    }
                    "toggle_filter" => {
                        let id = params["identifier"].as_str().unwrap_or_default();
                        let enabled = params["enabled"].as_bool().unwrap_or(true);
                        let config = client.list_filters().await?;
                        let filter = config.filters.iter().chain(config.whitelist_filters.iter())
                            .find(|f| f.name == id || f.url == id || f.id.to_string() == id);
                        if let Some(f) = filter {
                            client.toggle_filter(f.url.clone(), f.name.clone(), enabled).await?;
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Filter '{}' toggled", f.name) }] }))
                        } else {
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Filter not found" }], "isError": true }))
                        }
                    }
                    "list_custom_rules" => {
                        let rules = client.get_user_rules().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": rules.join("\n") }] }))
                    }
                    "set_custom_rules" => {
                        let rules = params["rules"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        client.set_user_rules(rules).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rules updated" }] }))
                    }
                    "add_custom_rule" => {
                        let rule = params["rule"].as_str().unwrap_or_default().to_string();
                        let mut rules = client.get_user_rules().await?;
                        if !rules.contains(&rule) {
                            rules.push(rule);
                            client.set_user_rules(rules).await?;
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rule added" }] }))
                        } else {
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rule exists" }] }))
                        }
                    }
                    "remove_custom_rule" => {
                        let rule = params["rule"].as_str().unwrap_or_default();
                        let mut rules = client.get_user_rules().await?;
                        if let Some(pos) = rules.iter().position(|r| r == rule) {
                            rules.remove(pos);
                            client.set_user_rules(rules).await?;
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rule removed" }] }))
                        } else {
                            Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rule not found" }], "isError": true }))
                        }
                    }
                    "list_blocked_services" => {
                        let all = client.list_all_services().await?;
                        let blocked = client.list_blocked_services().await?;
                        let res: Vec<_> = all.into_iter().map(|s| serde_json::json!({ "id": s.id, "name": s.name, "blocked": blocked.contains(&s.id) })).collect();
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res)? }] }))
                    }
                    "toggle_blocked_service" => {
                        let id = params["service_id"].as_str().unwrap_or_default().to_string();
                        let blocked = params["blocked"].as_bool().unwrap_or(true);
                        let mut current = client.list_blocked_services().await?;
                        let exists = current.contains(&id);
                        if blocked && !exists { current.push(id); client.set_blocked_services(current).await?; }
                        else if !blocked && exists { current.retain(|x| x != &id); client.set_blocked_services(current).await?; }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Service toggled" }] }))
                    }
                    "check_host" => {
                        let domain = params["domain"].as_str().unwrap_or_default();
                        let cl = params["client"].as_str();
                        let res = client.check_host(domain, cl).await?;
                        let mut text = format!("Status for {}: {}\n", domain, res.reason);
                        if let Some(r) = res.rule { text.push_str(&format!("Rule: {}\n", r)); }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": text }] }))
                    }
                    _ => Err(crate::error::Error::Mcp(crate::mcp::ResponseError {
                        code: -32602, message: format!("Unknown action: {}", action), data: None,
                    })),
                }
            }
        },
    );
}
