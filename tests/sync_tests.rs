mod common;
use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::config::AppConfig;
use adguardhome_mcp_rs::tools::sync as sync_tools;
use adguardhome_mcp_rs::tools::{ToolRegistry, filtering};
use anyhow::Result;
use common::AdGuardContainer;

#[tokio::test]
async fn test_sync_integration() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    // 1. Start Master with auth
    let master_container =
        AdGuardContainer::new(Some("tests/common/AdGuardHome.yaml".to_string())).await?;
    let master_host = master_container.host.clone();
    let master_port = master_container.port;

    // 2. Start Replica with auth
    let replica_container =
        AdGuardContainer::new(Some("tests/common/AdGuardHome.yaml".to_string())).await?;
    let replica_host = replica_container.host.clone();
    let replica_port = replica_container.port;

    let mut master_config = AppConfig {
        adguard_host: master_host,
        adguard_port: master_port,
        adguard_username: Some("admin".to_string()),
        adguard_password: Some("password".to_string()),
        ..Default::default()
    };
    master_config.validate().unwrap();
    let master_client = AdGuardClient::new(master_config.get_instance(None).unwrap().clone());
    let mut registry = ToolRegistry::new(&master_config);
    sync_tools::register(&mut registry);
    filtering::register(&mut registry);

    // 3. Configure Master with a custom rule
    let test_rule = "||master-only-rule.com^".to_string();
    registry
        .call_tool(
            "manage_filtering",
            &master_client,
            &master_config,
            Some(serde_json::json!({"action": "set_custom_rules", "rules": [test_rule]})),
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // 4. Perform Sync via tool
    let replica_url = format!("http://{}:{}", replica_host, replica_port);
    let res = registry
        .call_tool(
            "sync_instances",
            &master_client,
            &master_config,
            Some(serde_json::json!({
                "mode": "full-overwrite",
                "replicas": [
                    {
                        "url": replica_url,
                        "api_key": "password" // Use 'password' as api_key because sync tool maps api_key to adguard_password
                    }
                ]
            })),
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("Sync response: {:?}", res);

    // 5. Verify Replica has the rule
    let mut replica_config = AppConfig {
        adguard_host: replica_host,
        adguard_port: replica_port,
        adguard_username: Some("admin".to_string()),
        adguard_password: Some("password".to_string()),
        ..Default::default()
    };
    replica_config.validate().unwrap();
    let replica_client = AdGuardClient::new(replica_config.get_instance(None).unwrap().clone());

    let mut success = false;
    for _ in 0..20 {
        match replica_client.get_user_rules().await {
            Ok(rules) => {
                if rules.contains(&test_rule) {
                    success = true;
                    break;
                }
                println!("⏳ Rules not synced yet. Current rules: {:?}", rules);
            }
            Err(e) => {
                println!("⏳ Failed to get rules from replica: {}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    assert!(success, "Replica did not receive the rule after sync");

    Ok(())
}
