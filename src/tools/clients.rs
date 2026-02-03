use super::ToolRegistry;
use crate::adguard::{AdGuardClientDevice, StaticLease};

pub fn register(registry: &mut ToolRegistry) {
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
                        blocked += 1;
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
            let lease = StaticLease {
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
            let lease = StaticLease {
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
}
