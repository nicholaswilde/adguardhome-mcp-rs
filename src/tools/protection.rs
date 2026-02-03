use super::ToolRegistry;

pub fn register(registry: &mut ToolRegistry) {
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

    // Register get_protection_config
    registry.register(
        "get_protection_config",
        "Retrieve current configuration for Safe Search and Parental Control",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let safe_search = client.get_safe_search_settings().await?;
                let parental = client.get_parental_settings().await?;
                let status = client.get_status().await?;

                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::json!({
                        "global_protection_enabled": status.protection_enabled,
                        "safe_search": safe_search,
                        "parental_control": parental
                    }).to_string() }]
                }))
            }
        },
    );

    // Register set_protection_config
    registry.register(
        "set_protection_config",
        "Update configuration for Safe Search and Parental Control",
        serde_json::json!({
            "type": "object",
            "properties": {
                "safe_search": {
                    "type": "object",
                    "properties": {
                        "enabled": { "type": "boolean" },
                        "bing": { "type": "boolean" },
                        "duckduckgo": { "type": "boolean" },
                        "google": { "type": "boolean" },
                        "pixabay": { "type": "boolean" },
                        "yandex": { "type": "boolean" },
                        "youtube": { "type": "boolean" }
                    }
                },
                "parental_control": {
                    "type": "object",
                    "properties": {
                        "enabled": { "type": "boolean" },
                        "sensitivity": { "type": "integer", "minimum": 0 }
                    }
                }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            async move {
                if let Some(safe_search_val) = params.get("safe_search") {
                    let mut current = client.get_safe_search_settings().await?;
                    if let Some(enabled) = safe_search_val.get("enabled").and_then(|v| v.as_bool()) { current.enabled = enabled; }
                    if let Some(bing) = safe_search_val.get("bing").and_then(|v| v.as_bool()) { current.bing = bing; }
                    if let Some(duckduckgo) = safe_search_val.get("duckduckgo").and_then(|v| v.as_bool()) { current.duckduckgo = duckduckgo; }
                    if let Some(google) = safe_search_val.get("google").and_then(|v| v.as_bool()) { current.google = google; }
                    if let Some(pixabay) = safe_search_val.get("pixabay").and_then(|v| v.as_bool()) { current.pixabay = pixabay; }
                    if let Some(yandex) = safe_search_val.get("yandex").and_then(|v| v.as_bool()) { current.yandex = yandex; }
                    if let Some(youtube) = safe_search_val.get("youtube").and_then(|v| v.as_bool()) { current.youtube = youtube; }

                    client.set_safe_search_settings(current).await?;
                }

                if let Some(parental_val) = params.get("parental_control") {
                    let mut current = client.get_parental_settings().await?;
                    if let Some(enabled) = parental_val.get("enabled").and_then(|v| v.as_bool()) { current.enabled = enabled; }
                    if let Some(sensitivity) = parental_val.get("sensitivity").and_then(|v| v.as_u64()) { current.sensitivity = Some(sensitivity as u32); }

                    client.set_parental_settings(current).await?;
                }

                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "Protection configuration updated successfully" }]
                }))
            }
        },
    );

    // Register get_tls_config
    registry.register(
        "get_tls_config",
        "Retrieve current TLS/SSL configuration",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        |client, _params| {
            let client = client.clone();
            async move {
                let config = client.get_tls_status().await?;
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

    // Register set_tls_config
    registry.register(
        "set_tls_config",
        "Update TLS/SSL configuration",
        serde_json::json!({
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean" },
                "server_name": { "type": "string" },
                "force_https": { "type": "boolean" },
                "port_https": { "type": "integer" },
                "port_dns_over_tls": { "type": "integer" },
                "port_dns_over_quic": { "type": "integer" },
                "certificate_chain": { "type": "string" },
                "private_key": { "type": "string" },
                "certificate_path": { "type": "string" },
                "private_key_path": { "type": "string" }
            }
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            async move {
                let mut config = client.get_tls_status().await?;
                if let Some(enabled) = params["enabled"].as_bool() { config.enabled = enabled; }
                if let Some(server_name) = params["server_name"].as_str() { config.server_name = server_name.to_string(); }
                if let Some(force_https) = params["force_https"].as_bool() { config.force_https = force_https; }
                if let Some(port_https) = params["port_https"].as_u64() { config.port_https = port_https as u16; }
                if let Some(port_dns_over_tls) = params["port_dns_over_tls"].as_u64() { config.port_dns_over_tls = port_dns_over_tls as u16; }
                if let Some(port_dns_over_quic) = params["port_dns_over_quic"].as_u64() { config.port_dns_over_quic = port_dns_over_quic as u16; }
                if let Some(cert) = params["certificate_chain"].as_str() { config.certificate_chain = cert.to_string(); }
                if let Some(key) = params["private_key"].as_str() { config.private_key = key.to_string(); }
                if let Some(cert_path) = params["certificate_path"].as_str() { config.certificate_path = cert_path.to_string(); }
                if let Some(key_path) = params["private_key_path"].as_str() { config.private_key_path = key_path.to_string(); }

                client.configure_tls(config).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": "TLS configuration updated successfully" }]
                }))
            }
        },
    );
}
