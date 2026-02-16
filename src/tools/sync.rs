use crate::adguard::AdGuardClient;
use crate::error::Result;
use crate::sync::SyncState;
use crate::tools::ToolRegistry;
use serde_json::{Value, json};

pub fn register(registry: &mut ToolRegistry) {
    registry.register(
        "sync_instances",
        "Synchronize configuration from the master instance to one or more replica instances.",
        json!({
            "type": "object",
            "properties": {
                "replicas": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "url": { "type": "string", "description": "URL of the replica AdGuard Home instance." },
                            "api_key": { "type": "string", "description": "API Key for the replica." }
                        },
                        "required": ["url", "api_key"]
                    },
                    "description": "Optional list of replicas to sync to. Defaults to configured replicas."
                },
                "mode": {
                    "type": "string",
                    "enum": ["additive-merge", "full-overwrite"],
                    "description": "Sync mode. Defaults to configured default mode."
                }
            }
        }),
        |client, args| {
            let client = client.clone();
            async move { sync_instances(&client, args).await }
        },
    );
}

async fn sync_instances(client: &AdGuardClient, args: Option<Value>) -> Result<Value> {
    let mode = args
        .as_ref()
        .and_then(|a| a.get("mode"))
        .and_then(|v| v.as_str())
        .unwrap_or(client.config.default_sync_mode.as_str());

    let replicas = if let Some(r) = args
        .as_ref()
        .and_then(|a| a.get("replicas"))
        .and_then(|v| v.as_array())
    {
        r.iter()
            .filter_map(|v| {
                let url = v.get("url")?.as_str()?.to_string();
                let api_key = v.get("api_key")?.as_str()?.to_string();
                Some(crate::config::ReplicaConfig { url, api_key })
            })
            .collect::<Vec<_>>()
    } else {
        client.config.replicas.clone()
    };

    if replicas.is_empty() {
        return Ok(json!({
            "content": [{
                "type": "text",
                "text": "No replicas configured or provided for synchronization."
            }],
            "isError": true
        }));
    }

    // 1. Fetch Master State
    let master_state = SyncState::fetch(client)
        .await
        .map_err(|e| crate::error::Error::Generic(e.to_string()))?;

    let mut results = Vec::new();

    // 2. Push to Replicas
    for replica_config in replicas {
        let mut replica_app_config = client.config.clone();
        // Parse URL to host and port
        let url = replica_config.url.clone();
        let parsed_url =
            url::Url::parse(&url).map_err(|e| crate::error::Error::Config(e.to_string()))?;
        replica_app_config.adguard_host = parsed_url.host_str().unwrap_or("localhost").to_string();
        replica_app_config.adguard_port = parsed_url.port().unwrap_or(80);
        replica_app_config.adguard_username = None; // Use API Key
        replica_app_config.adguard_password = Some(replica_config.api_key.clone());

        let replica_client = AdGuardClient::new(replica_app_config);

        match master_state.push_to_replica(&replica_client, mode).await {
            Ok(_) => results.push(format!("Successfully synced to {}", url)),
            Err(e) => results.push(format!("Failed to sync to {}: {}", url, e)),
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": results.join("
    ")
        }]
    }))
}
