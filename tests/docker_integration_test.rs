mod common;
use adguardhome_mcp_rs::config::{AppConfig, InstanceConfig};
use adguardhome_mcp_rs::server::mcp::McpServer;
use adguardhome_mcp_rs::tools::ToolRegistry;
use anyhow::Result;
use common::AdGuardContainer;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_multi_instance_selection() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = AdGuardContainer::new(None).await?;
    let url = format!("http://{}:{}", container.host, container.port);

    // Configure two instances pointing to the same container
    let mut config = AppConfig {
        instances: vec![
            InstanceConfig {
                name: Some("primary".to_string()),
                url: url.clone(),
                username: Some("admin".to_string()),
                password: Some("password".to_string()),
                ..Default::default()
            },
            InstanceConfig {
                name: Some("secondary".to_string()),
                url: url.clone(),
                username: Some("admin".to_string()),
                password: Some("password".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    config.validate().unwrap();

    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::system::register(&mut registry);
    let (server, _rx) = McpServer::with_registry(Arc::new(Mutex::new(registry)), config);

    // 1. Call without instance arg (defaults to primary)
    let res1 = server
        .handle_request(adguardhome_mcp_rs::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: adguardhome_mcp_rs::mcp::RequestId::Number(1),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "manage_system",
                "arguments": { "action": "get_status" }
            })),
        })
        .await?;
    assert!(
        res1["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Version:")
    );

    // 2. Call with instance="secondary"
    let res2 = server
        .handle_request(adguardhome_mcp_rs::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: adguardhome_mcp_rs::mcp::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "manage_system",
                "arguments": { "action": "get_status", "instance": "secondary" }
            })),
        })
        .await?;
    assert!(
        res2["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Version:")
    );

    // 3. Call with instance="1" (index)
    let res3 = server
        .handle_request(adguardhome_mcp_rs::mcp::Request {
            jsonrpc: "2.0".to_string(),
            id: adguardhome_mcp_rs::mcp::RequestId::Number(3),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "manage_system",
                "arguments": { "action": "get_status", "instance": "1" }
            })),
        })
        .await?;
    assert!(
        res3["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Version:")
    );

    Ok(())
}

#[tokio::test]
async fn test_multi_instance_env_vars() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = AdGuardContainer::new(None).await?;
    let url = format!("http://{}:{}", container.host, container.port);

    use adguardhome_mcp_rs::test_utils::ENV_LOCK;
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe {
        std::env::set_var("ADGUARD_HOST", "localhost");
        std::env::set_var("ADGUARD_INSTANCES__0__NAME", "primary");
        std::env::set_var("ADGUARD_INSTANCES__0__URL", &url);
        std::env::set_var("ADGUARD_INSTANCES__0__USERNAME", "admin");
        std::env::set_var("ADGUARD_INSTANCES__0__PASSWORD", "password");
        std::env::set_var("ADGUARD_INSTANCES__1__NAME", "secondary");
        std::env::set_var("ADGUARD_INSTANCES__1__URL", &url);
        std::env::set_var("ADGUARD_INSTANCES__1__USERNAME", "admin");
        std::env::set_var("ADGUARD_INSTANCES__1__PASSWORD", "password");
    }

    let config = AppConfig::load(None, vec![]).unwrap();

    unsafe {
        std::env::remove_var("ADGUARD_HOST");
        std::env::remove_var("ADGUARD_INSTANCES__0__NAME");
        std::env::remove_var("ADGUARD_INSTANCES__0__URL");
        std::env::remove_var("ADGUARD_INSTANCES__0__USERNAME");
        std::env::remove_var("ADGUARD_INSTANCES__0__PASSWORD");
        std::env::remove_var("ADGUARD_INSTANCES__1__NAME");
        std::env::remove_var("ADGUARD_INSTANCES__1__URL");
        std::env::remove_var("ADGUARD_INSTANCES__1__USERNAME");
        std::env::remove_var("ADGUARD_INSTANCES__1__PASSWORD");
    }

    assert_eq!(config.instances.len(), 2);
    assert_eq!(config.instances[0].name, Some("primary".to_string()));
    assert_eq!(config.instances[1].name, Some("secondary".to_string()));

    Ok(())
}
