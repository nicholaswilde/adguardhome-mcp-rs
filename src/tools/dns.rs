use super::ToolRegistry;
use crate::adguard::DnsRewrite;

pub fn register(registry: &mut ToolRegistry) {
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
}
