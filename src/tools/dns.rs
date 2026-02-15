use super::ToolRegistry;
use crate::adguard::DnsRewrite;

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        "manage_dns",
        "Manage DNS settings (rewrites, upstream servers, cache).",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "DNS action to perform",
                    "enum": ["list_rewrites", "add_rewrite", "remove_rewrite", "get_config", "set_config", "clear_cache"]
                },
                "domain": { "type": "string", "description": "Domain for rewrite" },
                "answer": { "type": "string", "description": "IP/CNAME for rewrite" },
                "upstream_dns": { "type": "array", "items": { "type": "string" } },
                "bootstrap_dns": { "type": "array", "items": { "type": "string" } },
                "fallback_dns": { "type": "array", "items": { "type": "string" } },
                "all_servers": { "type": "boolean" },
                "fastest_addr": { "type": "boolean" },
                "cache_size": { "type": "integer" },
                "cache_ttl_min": { "type": "integer" },
                "cache_ttl_max": { "type": "integer" },
                "cache_optimistic": { "type": "boolean" }
            },
            "required": ["action"]
        }),
        |client, params| {
            let client = client.clone();
            let params = params.unwrap_or_default();
            let action = params["action"].as_str().unwrap_or_default().to_string();

            async move {
                match action.as_str() {
                    "list_rewrites" => {
                        let rewrites = client.list_rewrites().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&rewrites)? }] }))
                    }
                    "add_rewrite" => {
                        let domain = params["domain"].as_str().unwrap_or_default().to_string();
                        let answer = params["answer"].as_str().unwrap_or_default().to_string();
                        client.add_rewrite(DnsRewrite { domain, answer }).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rewrite added" }] }))
                    }
                    "remove_rewrite" => {
                        let domain = params["domain"].as_str().unwrap_or_default().to_string();
                        let answer = params["answer"].as_str().unwrap_or_default().to_string();
                        client.delete_rewrite(DnsRewrite { domain, answer }).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Rewrite removed" }] }))
                    }
                    "get_config" => {
                        let config = client.get_dns_info().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&config)? }] }))
                    }
                    "set_config" => {
                        let mut config = client.get_dns_info().await?;
                        if let Some(u) = params["upstream_dns"].as_array() {
                            config.upstream_dns = u.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        }
                        if let Some(b) = params["bootstrap_dns"].as_array() {
                            config.bootstrap_dns = b.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        }
                        if let Some(f) = params["fallback_dns"].as_array() {
                            config.fallback_dns = f.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        }
                        if let Some(a) = params["all_servers"].as_bool() { config.all_servers = a; }
                        if let Some(f) = params["fastest_addr"].as_bool() { config.fastest_addr = f; }
                        if let Some(s) = params["cache_size"].as_u64() { config.cache_size = s as u32; }
                        if let Some(min) = params["cache_ttl_min"].as_u64() { config.cache_ttl_min = min as u32; }
                        if let Some(max) = params["cache_ttl_max"].as_u64() { config.cache_ttl_max = max as u32; }
                        if let Some(o) = params["cache_optimistic"].as_bool() { config.cache_optimistic = o; }

                        client.set_dns_config(config).await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "DNS config updated" }] }))
                    }
                    "clear_cache" => {
                        client.clear_dns_cache().await?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": "DNS cache cleared" }] }))
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
