use super::ToolRegistry;
use crate::adguard::{AdGuardClientDevice, StaticLease};

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        "manage_clients",
        "Manage clients (CRUD), DHCP leases, and access control lists.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Client action to perform",
                    "enum": [
                        "list_clients", "get_client_info", "add_client", "update_client",
                        "delete_client", "get_activity_report", "get_access_list",
                        "update_access_list", "list_dhcp_leases", "add_static_lease",
                        "remove_static_lease"
                    ]
                },
                "identifier": { "type": "string", "description": "IP, MAC, or Name" },
                "name": { "type": "string" },
                "old_name": { "type": "string" },
                "ids": { "type": "array", "items": { "type": "string" } },
                "use_global_settings": { "type": "boolean" },
                "filtering_enabled": { "type": "boolean" },
                "parental_enabled": { "type": "boolean" },
                "safebrowsing_enabled": { "type": "boolean" },
                "safesearch_enabled": { "type": "boolean" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 100 },
                "allowed_clients": { "type": "array", "items": { "type": "string" } },
                "disallowed_clients": { "type": "array", "items": { "type": "string" } },
                "blocked_hosts": { "type": "array", "items": { "type": "string" } },
                "mac": { "type": "string" },
                "ip": { "type": "string" },
                "hostname": { "type": "string" }
            },
            "required": ["action"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let action = params["action"].as_str().unwrap_or_default().to_string();

            async move {
                match action.as_str() {
                    "list_clients" => {
                        let res = client.list_clients().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res)? }] }))
                    }
                    "get_client_info" => {
                        let id = params["identifier"].as_str().unwrap_or_default();
                        let res = client.get_client_info(id).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res)? }] }))
                    }
                    "add_client" => {
                        let device = AdGuardClientDevice {
                            name: params["name"].as_str().unwrap_or_default().to_string(),
                            ids: params["ids"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                            use_global_settings: params["use_global_settings"].as_bool().unwrap_or(true),
                            filtering_enabled: params["filtering_enabled"].as_bool().unwrap_or(true),
                            parental_enabled: params["parental_enabled"].as_bool().unwrap_or(false),
                            safebrowsing_enabled: params["safebrowsing_enabled"].as_bool().unwrap_or(true),
                            safesearch_enabled: params["safesearch_enabled"].as_bool().unwrap_or(false),
                        };
                        client.add_client(device).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Client added" }] }))
                    }
                    "update_client" => {
                        let old = params["old_name"].as_str().unwrap_or_default();
                        let curr = client.get_client_info(old).await?;
                        let device = AdGuardClientDevice {
                            name: params["name"].as_str().map(|s| s.to_string()).unwrap_or(curr.name),
                            ids: params["ids"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or(curr.ids),
                            use_global_settings: params["use_global_settings"].as_bool().unwrap_or(curr.use_global_settings),
                            filtering_enabled: params["filtering_enabled"].as_bool().unwrap_or(curr.filtering_enabled),
                            parental_enabled: params["parental_enabled"].as_bool().unwrap_or(curr.parental_enabled),
                            safebrowsing_enabled: params["safebrowsing_enabled"].as_bool().unwrap_or(curr.safebrowsing_enabled),
                            safesearch_enabled: params["safesearch_enabled"].as_bool().unwrap_or(curr.safesearch_enabled),
                        };
                        client.update_client(old.to_string(), device).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Client updated" }] }))
                    }
                    "delete_client" => {
                        let name = params["name"].as_str().unwrap_or_default();
                        client.delete_client(name.to_string()).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Client removed" }] }))
                    }
                    "get_activity_report" => {
                        let id = params["identifier"].as_str().unwrap_or_default();
                        let lim = params["limit"].as_u64().map(|l| l as u32).unwrap_or(50);
                        let log = client.get_query_log(Some(id), None, Some(lim)).await?;
                        let mut total = 0; let mut blocked = 0;
                        let mut domains = std::collections::HashMap::new();
                        for entry in &log.data {
                            total += 1;
                            if entry.reason != "NotFilteredNotFound" && !entry.reason.is_empty() && entry.status != "NOERROR" { blocked += 1; }
                            *domains.entry(entry.question.name.clone()).or_insert(0) += 1;
                        }
                        let mut top: Vec<_> = domains.into_iter().collect();
                        top.sort_by(|a, b| b.1.cmp(&a.1)); top.truncate(5);
                        let mut text = format!("Report for {}: Analyzed={}, Blocked={}\nTop Domains:\n", id, total, blocked);
                        for (d, c) in top { text.push_str(&format!("- {}: {}\n", d, c)); }
                        if total == 0 { text = format!("No activity for {}", id); }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": text }] }))
                    }
                    "get_access_list" => {
                        let res = client.get_access_list().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res)? }] }))
                    }
                    "update_access_list" => {
                        let mut list = client.get_access_list().await?;
                        if let Some(a) = params["allowed_clients"].as_array() { list.allowed_clients = a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(); }
                        if let Some(d) = params["disallowed_clients"].as_array() { list.disallowed_clients = d.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(); }
                        if let Some(b) = params["blocked_hosts"].as_array() { list.blocked_hosts = b.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(); }
                        client.set_access_list(list).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Access list updated" }] }))
                    }
                    "list_dhcp_leases" => {
                        let res = client.get_dhcp_status().await?;
                        let mut text = format!("DHCP Enabled: {}, Interface: {}\nDynamic:\n", res.enabled, res.interface_name);
                        for l in res.leases { text.push_str(&format!("- {} ({}) expires {}\n", l.hostname, l.ip, l.expires.as_deref().unwrap_or("Never"))); }
                        text.push_str("Static:\n");
                        for l in res.static_leases { text.push_str(&format!("- {} ({}) [{}]\n", l.hostname, l.ip, l.mac)); }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": text }] }))
                    }
                    "add_static_lease" => {
                        let lease = StaticLease {
                            mac: params["mac"].as_str().unwrap_or_default().to_string(),
                            ip: params["ip"].as_str().unwrap_or_default().to_string(),
                            hostname: params["hostname"].as_str().unwrap_or_default().to_string(),
                        };
                        client.add_static_lease(lease).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Static lease added" }] }))
                    }
                    "remove_static_lease" => {
                        let lease = StaticLease {
                            mac: params["mac"].as_str().unwrap_or_default().to_string(),
                            ip: params["ip"].as_str().unwrap_or_default().to_string(),
                            hostname: params["hostname"].as_str().unwrap_or_default().to_string(),
                        };
                        client.remove_static_lease(lease).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Static lease removed" }] }))
                    }
                    _ => Err(crate::error::Error::Mcp(crate::mcp::ResponseError {
                        code: -32602, message: format!("Unknown action: {}", action), data: None,
                    })),
                }
            }
        },
    );
}
