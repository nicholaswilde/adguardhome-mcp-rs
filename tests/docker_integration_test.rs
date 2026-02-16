use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::config::AppConfig;
use adguardhome_mcp_rs::server::http::run_http_server;
use adguardhome_mcp_rs::server::mcp::McpServer;
use adguardhome_mcp_rs::tools::ToolRegistry;
use anyhow::Result;
use std::io::Write;
use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::io::AsyncBufReadExt;

async fn start_adguard_container(
    config_path: Option<String>,
) -> Result<(testcontainers::ContainerAsync<GenericImage>, String, u16)> {
    println!("🐳 Starting AdGuard Home container...");

    let image = GenericImage::new("adguard/adguardhome", "latest")
        .with_exposed_port(ContainerPort::Tcp(80))
        .with_wait_for(WaitFor::message_on_stdout("AdGuard Home is available at"));

    let container = if let Some(path) = config_path {
        image
            .with_mount(Mount::bind_mount(
                path,
                "/opt/adguardhome/conf/AdGuardHome.yaml",
            ))
            .start()
            .await
    } else {
        image.start().await
    };

    let container = match container {
        Ok(c) => c,
        Err(e) => {
            println!(
                "⚠️ Failed to start Docker container. This is expected in environments without Docker. Skipping integration test. Error: {}",
                e
            );
            return Err(anyhow::anyhow!("Failed to start container"));
        }
    };

    // Pipe stdout logs
    let stdout = container.stdout(true);
    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("DOCKER STDOUT: {}", line);
        }
    });

    let port = container.get_host_port_ipv4(80).await?;
    let host = "localhost".to_string();

    println!(
        "✅ AdGuard Home container started at http://{}:{}",
        host, port
    );

    Ok((container, host, port))
}

async fn call_mcp_tool(
    registry: &ToolRegistry,
    client: &AdGuardClient,
    name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    registry
        .call_tool(name, client, Some(args))
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

#[tokio::test]
async fn test_mcp_http_transport() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        mcp_transport: "http".to_string(),
        http_port: 0,
        http_auth_token: Some("test-token".to_string()),
        ..Default::default()
    };

    let adguard_client = AdGuardClient::new(config.clone());
    let registry = ToolRegistry::new(&config);
    let (server, rx) = McpServer::new(adguard_client, registry, config.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let server_handle = server.clone();
    tokio::spawn(async move {
        run_http_server(
            server_handle,
            rx,
            "127.0.0.1",
            port,
            Some("test-token".to_string()),
        )
        .await
        .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let client = reqwest::Client::new();
    let mcp_url = format!("http://127.0.0.1:{}", port);

    let resp = client.get(format!("{}/sse", mcp_url)).send().await?;
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    let resp = client
        .get(format!("{}/sse", mcp_url))
        .header("Authorization", "Bearer test-token")
        .send()
        .await?;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/event-stream"
    );

    Ok(())
}

#[tokio::test]
async fn test_adguardhome_no_auth() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config);

    let mut status = None;
    for _ in 0..10 {
        match client.get_status().await {
            Ok(s) => {
                status = Some(s);
                break;
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    let status = status.expect("Failed to get status from AdGuard Home");
    assert!(!status.version.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_adguardhome_with_auth() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let mut temp_file = tempfile::NamedTempFile::new()?;
    let config_content = r#"
http:
  address: 0.0.0.0:80
users:
  - name: admin
    password: $2y$10$mow9sogGbkORyx5XpI8MLeMP5lar/V7XeAmKeeaN8L1H9aTTHGN3u
"#;
    write!(temp_file, "{}", config_content)?;
    let config_path = temp_file.path().to_str().unwrap().to_string();

    let (_container, adguard_host, adguard_port) =
        match start_adguard_container(Some(config_path)).await {
            Ok(res) => res,
            Err(_) => return Ok(()),
        };

    let config_no_auth = AppConfig {
        adguard_host: adguard_host.clone(),
        adguard_port,
        ..Default::default()
    };
    let client_no_auth = AdGuardClient::new(config_no_auth);

    let mut success = false;
    for _ in 0..10 {
        match client_no_auth.get_status().await {
            Ok(_) => {
                panic!("Expected authentication failure, but got success");
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("401") {
                    success = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
    assert!(success, "Did not receive 401 Unauthorized as expected");

    let config_auth = AppConfig {
        adguard_host,
        adguard_port,
        adguard_username: Some("admin".to_string()),
        adguard_password: Some("password".to_string()),
        ..Default::default()
    };
    let client_auth = AdGuardClient::new(config_auth);

    let mut status = None;
    for _ in 0..5 {
        match client_auth.get_status().await {
            Ok(s) => {
                status = Some(s);
                break;
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    let status = status.expect("Failed to get status with valid credentials");
    assert!(!status.version.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_dns_rewrites() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::dns::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let domain = "test.example.com".to_string();
    let answer = "1.1.1.1".to_string();

    call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "add_rewrite", "domain": domain, "answer": answer}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "list_rewrites"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&domain)
    );

    call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "remove_rewrite", "domain": domain, "answer": answer}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "list_rewrites"}),
    )
    .await?;
    assert!(
        !res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&domain)
    );

    Ok(())
}

#[tokio::test]
async fn test_monitoring_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::system::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "get_stats"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Queries:")
    );

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "get_query_log", "limit": 5}),
    )
    .await?;
    assert!(!res["content"][0]["text"].as_str().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_protection_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::protection::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "toggle_feature", "feature": "global", "enabled": false}),
    )
    .await?;
    assert!(!client.get_status().await?.protection_enabled);

    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "toggle_feature", "feature": "global", "enabled": true}),
    )
    .await?;
    assert!(client.get_status().await?.protection_enabled);

    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "toggle_feature", "feature": "safe_search", "enabled": true}),
    )
    .await?;
    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "toggle_feature", "feature": "safe_browsing", "enabled": true}),
    )
    .await?;
    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "toggle_feature", "feature": "parental_control", "enabled": true}),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_filtering_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::filtering::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let filter_name = "Integration Test Filter".to_string();
    let filter_url = "https://raw.githubusercontent.com/AdguardTeam/AdguardFilters/master/BaseFilter/sections/adservers.txt".to_string();

    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "add_filter", "name": filter_name, "url": filter_url}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_filters"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&filter_name)
    );

    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "toggle_filter", "identifier": filter_name, "enabled": false}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_filters"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"enabled\": false")
    );

    let test_rule = "||integration-test.example.com^".to_string();
    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "set_custom_rules", "rules": [test_rule]}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_custom_rules"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&test_rule)
    );

    Ok(())
}

#[tokio::test]
async fn test_client_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::clients::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "list_clients"}),
    )
    .await?;
    assert!(res["content"][0]["text"].as_str().unwrap().contains("["));

    let test_client_name = "Integration Test Client".to_string();
    call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({
            "action": "add_client",
            "name": test_client_name,
            "ids": ["1.2.3.4"]
        }),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "get_client_info", "identifier": test_client_name}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&test_client_name)
    );

    call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "delete_client", "name": test_client_name}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "list_clients"}),
    )
    .await?;
    assert!(
        !res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&test_client_name)
    );

    Ok(())
}

#[tokio::test]
async fn test_blocked_services() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::filtering::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_blocked_services"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("youtube")
    );

    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "toggle_blocked_service", "service_id": "youtube", "blocked": true}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_blocked_services"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("\"blocked\": true")
    );

    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "toggle_blocked_service", "service_id": "youtube", "blocked": false}),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_dns_config_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::dns::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "get_config"}),
    )
    .await?;
    let _dns_info: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;

    call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "set_config", "upstream_dns": ["1.1.1.1"]}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "get_config"}),
    )
    .await?;
    let updated_info: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
    assert_eq!(updated_info["upstream_dns"][0].as_str().unwrap(), "1.1.1.1");

    call_mcp_tool(
        &registry,
        &client,
        "manage_dns",
        serde_json::json!({"action": "clear_cache"}),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_access_control_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::clients::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "get_access_list"}),
    )
    .await?;
    let mut list: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;

    let blocked_hosts = list["blocked_hosts"].as_array_mut().unwrap();
    blocked_hosts.push(serde_json::json!("test-blocked.com"));

    call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "update_access_list", "blocked_hosts": blocked_hosts}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "get_access_list"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("test-blocked.com")
    );

    Ok(())
}

#[tokio::test]
async fn test_dhcp_tools() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::clients::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // DHCP might not be enabled in the default container, but we can check if the endpoint responds
    call_mcp_tool(
        &registry,
        &client,
        "manage_clients",
        serde_json::json!({"action": "list_dhcp_leases"}),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_check_filtering() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::filtering::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Check an allowed domain
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "check_host", "domain": "google.com"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("NotFilteredNotFound")
    );

    // Add a block rule
    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "set_custom_rules", "rules": ["||blocked-domain.com^"]}),
    )
    .await?;

    // Check the blocked domain
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "check_host", "domain": "blocked-domain.com"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Filtered")
    );

    Ok(())
}

#[tokio::test]
async fn test_maintenance_tools_integration() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::system::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Call clear_stats
    call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "clear_stats"}),
    )
    .await?;

    // Call clear_query_log
    call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "clear_query_log"}),
    )
    .await?;

    // Call create_backup
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "create_backup"}),
    )
    .await?;
    let backup_path_str = res["content"][0]["text"]
        .as_str()
        .unwrap()
        .split("Backup: ")
        .nth(1)
        .unwrap();
    let backup_path = std::path::Path::new(backup_path_str);
    assert!(backup_path.exists());

    // Call restore_backup
    call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "restore_backup", "file_path": backup_path_str}),
    )
    .await?;

    let _ = tokio::fs::remove_file(backup_path).await;

    // Call restart_service
    call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "restart_service"}),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_filter_list_crud_integration() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::filtering::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let filter_name = "CRUD Test Filter".to_string();
    let filter_url = "https://raw.githubusercontent.com/AdguardTeam/AdguardFilters/master/BaseFilter/sections/adservers.txt".to_string();

    // Add
    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "add_filter", "name": filter_name, "url": filter_url}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_filters"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&filter_name)
    );

    // Update
    let new_name = "Updated CRUD Filter".to_string();
    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "update_filter", "identifier": filter_url, "new_name": new_name}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_filters"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&new_name)
    );

    // Remove
    call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "remove_filter", "identifier": filter_url}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_filtering",
        serde_json::json!({"action": "list_filters"}),
    )
    .await?;
    assert!(
        !res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains(&filter_url)
    );

    Ok(())
}

#[tokio::test]
async fn test_advanced_protection_integration() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::protection::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Test Safe Search Settings
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "get_config"}),
    )
    .await?;
    let config_val: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
    let mut safe_search = config_val["safe_search"].clone();

    // Toggle one of them
    let original_bing = safe_search["bing"].as_bool().unwrap();
    safe_search["bing"] = serde_json::json!(!original_bing);
    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "set_config", "safe_search": safe_search}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "get_config"}),
    )
    .await?;
    let updated_config: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
    assert_eq!(
        updated_config["safe_search"]["bing"].as_bool().unwrap(),
        !original_bing
    );

    // Test Parental Settings
    let mut parental = config_val["parental_control"].clone();
    let original_parental = parental["enabled"].as_bool().unwrap();
    parental["enabled"] = serde_json::json!(!original_parental);
    call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "set_config", "parental_control": parental}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "get_config"}),
    )
    .await?;
    let updated_config: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
    assert_eq!(
        updated_config["parental_control"]["enabled"]
            .as_bool()
            .unwrap(),
        !original_parental
    );

    Ok(())
}

#[tokio::test]
async fn test_query_log_config_integration() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::system::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Get current config
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "get_query_log_config"}),
    )
    .await?;
    let ql_config: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;

    // Toggle anonymize_client_ip
    let original_val = ql_config["anonymize_client_ip"].as_bool().unwrap();
    call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "set_query_log_config", "anonymize_client_ip": !original_val}),
    )
    .await?;

    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "get_query_log_config"}),
    )
    .await?;
    let updated_config: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
    assert_eq!(
        updated_config["anonymize_client_ip"].as_bool().unwrap(),
        !original_val
    );

    Ok(())
}

#[tokio::test]
async fn test_system_info_integration() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::system::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Get version info
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "get_version_info"}),
    )
    .await?;
    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("version")
    );

    // Trigger update
    call_mcp_tool(
        &registry,
        &client,
        "manage_system",
        serde_json::json!({"action": "update_adguard_home"}),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_tls_config_integration() -> Result<()> {
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    let config = AppConfig {
        adguard_host,
        adguard_port,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    adguardhome_mcp_rs::tools::protection::register(&mut registry);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Call get_tls_config
    let res = call_mcp_tool(
        &registry,
        &client,
        "manage_protection",
        serde_json::json!({"action": "get_tls_config"}),
    )
    .await?;
    let status: serde_json::Value =
        serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
    assert!(!status["enabled"].as_bool().unwrap()); // Default is usually disabled

    Ok(())
}
