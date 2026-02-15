use super::ToolRegistry;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        "manage_protection",
        "Manage protection settings (global, safe search, parental control, TLS).",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Protection action to perform",
                    "enum": ["get_config", "set_config", "toggle_feature", "get_tls_config", "set_tls_config"]
                },
                "feature": {
                    "type": "string",
                    "enum": ["global", "safe_search", "safe_browsing", "parental_control"],
                    "description": "Feature to toggle"
                },
                "enabled": { "type": "boolean", "description": "Toggle state" },
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
                },
                "server_name": { "type": "string" },
                "force_https": { "type": "boolean" },
                "port_https": { "type": "integer" },
                "port_dns_over_tls": { "type": "integer" },
                "port_dns_over_quic": { "type": "integer" },
                "certificate_chain": { "type": "string" },
                "private_key": { "type": "string" },
                "certificate_path": { "type": "string" },
                "private_key_path": { "type": "string" }
            },
            "required": ["action"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let action = params["action"].as_str().unwrap_or_default().to_string();

            async move {
                match action.as_str() {
                    "get_config" => {
                        let safe_search = client.get_safe_search_settings().await?;
                        let parental = client.get_parental_settings().await?;
                        let status = client.get_status().await?;
                        Ok(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::json!({
                                    "global_protection": status.protection_enabled,
                                    "safe_search": safe_search,
                                    "parental_control": parental
                                }).to_string()
                            }]
                        }))
                    }
                    "set_config" => {
                        if let Some(ss_val) = params.get("safe_search") {
                            let mut current = client.get_safe_search_settings().await?;
                            if let Some(e) = ss_val.get("enabled").and_then(|v| v.as_bool()) { current.enabled = e; }
                            if let Some(b) = ss_val.get("bing").and_then(|v| v.as_bool()) { current.bing = b; }
                            if let Some(d) = ss_val.get("duckduckgo").and_then(|v| v.as_bool()) { current.duckduckgo = d; }
                            if let Some(g) = ss_val.get("google").and_then(|v| v.as_bool()) { current.google = g; }
                            if let Some(p) = ss_val.get("pixabay").and_then(|v| v.as_bool()) { current.pixabay = p; }
                            if let Some(y) = ss_val.get("yandex").and_then(|v| v.as_bool()) { current.yandex = y; }
                            if let Some(yt) = ss_val.get("youtube").and_then(|v| v.as_bool()) { current.youtube = yt; }
                            client.set_safe_search_settings(current).await?;
                        }
                        if let Some(pc_val) = params.get("parental_control") {
                            let mut current = client.get_parental_settings().await?;
                            if let Some(e) = pc_val.get("enabled").and_then(|v| v.as_bool()) { current.enabled = e; }
                            if let Some(s) = pc_val.get("sensitivity").and_then(|v| v.as_u64()) { current.sensitivity = Some(s as u32); }
                            client.set_parental_settings(current).await?;
                        }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Config updated" }] }))
                    }
                    "toggle_feature" => {
                        let feature = params["feature"].as_str().unwrap_or_default();
                        let enabled = params["enabled"].as_bool().unwrap_or(true);
                        match feature {
                            "global" => client.set_protection(enabled).await?,
                            "safe_search" => client.set_safe_search(enabled).await?,
                            "safe_browsing" => client.set_safe_browsing(enabled).await?,
                            "parental_control" => client.set_parental_control(enabled).await?,
                            _ => return Err(crate::error::Error::Mcp(crate::mcp::ResponseError {
                                code: -32602, message: format!("Unknown feature: {}", feature), data: None,
                            })),
                        }
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Feature '{}' set to {}", feature, enabled) }] }))
                    }
                    "get_tls_config" => {
                        let config = client.get_tls_status().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&config)? }] }))
                    }
                    "set_tls_config" => {
                        let mut config = client.get_tls_status().await?;
                        if let Some(e) = params["enabled"].as_bool() { config.enabled = e; }
                        if let Some(s) = params["server_name"].as_str() { config.server_name = s.to_string(); }
                        if let Some(f) = params["force_https"].as_bool() { config.force_https = f; }
                        if let Some(p) = params["port_https"].as_u64() { config.port_https = p as u16; }
                        if let Some(p) = params["port_dns_over_tls"].as_u64() { config.port_dns_over_tls = p as u16; }
                        if let Some(p) = params["port_dns_over_quic"].as_u64() { config.port_dns_over_quic = p as u16; }
                        if let Some(c) = params["certificate_chain"].as_str() { config.certificate_chain = c.to_string(); }
                        if let Some(k) = params["private_key"].as_str() { config.private_key = k.to_string(); }
                        if let Some(cp) = params["certificate_path"].as_str() { config.certificate_path = cp.to_string(); }
                        if let Some(kp) = params["private_key_path"].as_str() { config.private_key_path = kp.to_string(); }
                        client.configure_tls(config).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "TLS config updated" }] }))
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
