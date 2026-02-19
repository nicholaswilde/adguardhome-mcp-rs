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
        |client, config, args| {
            let client = client.clone();
            let config = config.clone();
            async move { sync_instances(&client, &config, args).await }
        },
    );
}

async fn sync_instances(client: &AdGuardClient, config: &crate::config::AppConfig, args: Option<Value>) -> Result<Value> {
    let mode = args
        .as_ref()
        .and_then(|a| a.get("mode"))
        .and_then(|v| v.as_str())
        .unwrap_or(config.default_sync_mode.as_str());

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
        config.replicas.clone()
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
        // Parse URL to host and port
        let url = replica_config.url.clone();
        let _parsed_url =
            url::Url::parse(&url).map_err(|e| crate::error::Error::Config(e.to_string()))?;
        
        let replica_instance = crate::config::InstanceConfig {
            name: Some("replica".to_string()),
            url: url.clone(),
            api_key: Some(replica_config.api_key.clone()),
            ..Default::default()
        };

        let replica_client = AdGuardClient::new(replica_instance);

        match master_state.push_to_replica(&replica_client, mode).await {
            Ok(result) => {
                let mut msg = format!("Replica {}: ", url);
                if result.success {
                    msg.push_str("Successfully synced.");
                } else {
                    msg.push_str(&format!(
                        "Synced with errors. Failed modules: {}. Errors: {}",
                        result.failed_modules.join(", "),
                        result.errors.join("; ")
                    ));
                }
                results.push(msg);
            }
            Err(e) => results.push(format!("Failed to connect or push to {}: {}", url, e)),
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
