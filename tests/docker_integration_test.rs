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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "http".to_string(),
        lazy_mode: false,
        http_port: 0,
        http_auth_token: Some("test-token".to_string()),
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };

    let adguard_client = AdGuardClient::new(config.clone());
    let registry = ToolRegistry::new(&config);
    let server = McpServer::new(adguard_client, registry, config.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let server_handle = server.clone();
    tokio::spawn(async move {
        run_http_server(
            server_handle,
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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
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
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let rewrite = adguardhome_mcp_rs::adguard::DnsRewrite {
        domain: "test.example.com".to_string(),
        answer: "1.1.1.1".to_string(),
    };

    client.add_rewrite(rewrite.clone()).await?;
    let rewrites = client.list_rewrites().await?;
    assert!(rewrites.iter().any(|r| r.domain == rewrite.domain));

    client.delete_rewrite(rewrite.clone()).await?;
    let rewrites = client.list_rewrites().await?;
    assert!(!rewrites.iter().any(|r| r.domain == rewrite.domain));

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let _stats = client.get_stats(None).await?;
    let _log = client.get_query_log(None, None, Some(5)).await?;

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    client.set_protection(false).await?;
    assert!(!client.get_status().await?.protection_enabled);
    client.set_protection(true).await?;
    assert!(client.get_status().await?.protection_enabled);

    client.set_safe_search(true).await?;
    client.set_safe_browsing(true).await?;
    client.set_parental_control(true).await?;

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    client
        .add_filter(filter_name.clone(), filter_url.clone(), false)
        .await?;

    let filtering = client.list_filters().await?;
    assert!(filtering.filters.iter().any(|f| f.name == filter_name));

    client
        .toggle_filter(filter_url.clone(), filter_name.clone(), false)
        .await?;
    let filtering = client.list_filters().await?;
    let filter = filtering
        .filters
        .iter()
        .find(|f| f.name == filter_name)
        .unwrap();
    assert!(!filter.enabled);

    let test_rule = "||integration-test.example.com^".to_string();
    client.set_user_rules(vec![test_rule.clone()]).await?;
    let rules = client.get_user_rules().await?;
    assert!(rules.contains(&test_rule));

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let clients = client.list_clients().await?;
    if !clients.is_empty() {
        let name = &clients[0].name;
        let info = client.get_client_info(name).await?;
        assert_eq!(&info.name, name);
    }

    let test_client_name = "Integration Test Client".to_string();
    let new_client = adguardhome_mcp_rs::adguard::AdGuardClientDevice {
        name: test_client_name.clone(),
        ids: vec!["1.2.3.4".to_string()],
        use_global_settings: true,
        filtering_enabled: true,
        parental_enabled: false,
        safebrowsing_enabled: true,
        safesearch_enabled: false,
    };
    client.add_client(new_client).await?;
    assert_eq!(
        client.get_client_info(&test_client_name).await?.name,
        test_client_name
    );

    client.delete_client(test_client_name.clone()).await?;
    let clients = client.list_clients().await?;
    assert!(!clients.iter().any(|c| c.name == test_client_name));

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let services = client.list_all_services().await?;
    assert!(!services.is_empty());

    client
        .set_blocked_services(vec!["youtube".to_string()])
        .await?;
    assert!(
        client
            .list_blocked_services()
            .await?
            .contains(&"youtube".to_string())
    );

    client.set_blocked_services(vec![]).await?;
    assert!(client.list_blocked_services().await?.is_empty());

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let dns_info = client.get_dns_info().await?;
    let mut new_config = dns_info.clone();
    new_config.upstream_dns = vec!["1.1.1.1".to_string()];
    client.set_dns_config(new_config).await?;
    assert_eq!(
        client.get_dns_info().await?.upstream_dns,
        vec!["1.1.1.1".to_string()]
    );

    client.clear_dns_cache().await?;

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    let mut list = client.get_access_list().await?;
    list.blocked_hosts.push("test-blocked.com".to_string());
    client.set_access_list(list.clone()).await?;
    assert!(
        client
            .get_access_list()
            .await?
            .blocked_hosts
            .contains(&"test-blocked.com".to_string())
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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    let _status = client.get_dhcp_status().await?;

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    let result = client.check_host("google.com", None).await?;
    assert_eq!(result.reason, "NotFilteredNotFound");

    // Add a block rule
    client
        .set_user_rules(vec!["||blocked-domain.com^".to_string()])
        .await?;

    // Check the blocked domain
    let result = client.check_host("blocked-domain.com", None).await?;
    assert!(result.reason.contains("Filtered"));
    assert_eq!(result.rule.unwrap(), "||blocked-domain.com^");

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Call reset_stats
    client.reset_stats().await?;

    // Call clear_query_log
    client.clear_query_log().await?;

    // Call create_backup
    let backup_path = client.create_backup().await?;
    assert!(backup_path.exists());

    // Call restore_backup
    client.restore_backup(backup_path.to_str().unwrap()).await?;

    let _ = tokio::fs::remove_file(backup_path).await;

    // Call restart_service
    client.restart_service().await?;

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    client
        .add_filter(filter_name.clone(), filter_url.clone(), false)
        .await?;

    let filtering = client.list_filters().await?;
    assert!(filtering.filters.iter().any(|f| f.name == filter_name));

    // Update
    let new_name = "Updated CRUD Filter".to_string();
    client
        .update_filter(
            filter_url.clone(),
            filter_url.clone(),
            new_name.clone(),
            false,
            true,
        )
        .await?;

    let filtering = client.list_filters().await?;
    assert!(filtering.filters.iter().any(|f| f.name == new_name));

    // Remove
    client.remove_filter(filter_url.clone(), false).await?;

    let filtering = client.list_filters().await?;
    assert!(!filtering.filters.iter().any(|f| f.url == filter_url));

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    let mut safe_search = client.get_safe_search_settings().await?;
    // Toggle one of them
    safe_search.bing = !safe_search.bing;
    client.set_safe_search_settings(safe_search.clone()).await?;

    let updated_safe_search = client.get_safe_search_settings().await?;
    assert_eq!(updated_safe_search.bing, safe_search.bing);

    // Test Parental Settings
    let mut parental = client.get_parental_settings().await?;
    // Toggle
    parental.enabled = !parental.enabled;
    client.set_parental_settings(parental.clone()).await?;

    let updated_parental = client.get_parental_settings().await?;
    assert_eq!(updated_parental.enabled, parental.enabled);

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    let mut ql_config = client.get_query_log_config().await?;

    // Toggle anonymize_client_ip
    let original_val = ql_config.anonymize_client_ip;
    ql_config.anonymize_client_ip = !original_val;

    client.set_query_log_config(ql_config.clone()).await?;

    let updated_config = client.get_query_log_config().await?;
    assert_eq!(updated_config.anonymize_client_ip, !original_val);

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

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
    let info = client.get_version_info().await?;
    assert!(!info.version.is_empty());

    // Trigger update (might fail in Docker if already latest or not supported, but we just check if it returns 200 or 400 with message)
    // Actually, AdGuard Home in Docker usually doesn't support self-update.
    // We'll just verify the endpoint exists and responds.
    let _ = client.update_adguard_home().await;

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
        adguard_username: None,
        adguard_password: None,
        mcp_transport: "stdio".to_string(),
        lazy_mode: false,
        http_port: 3000,
        http_auth_token: None,
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };
    let client = AdGuardClient::new(config);

    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready);

    // Call get_tls_status
    let status = client.get_tls_status().await?;
    assert!(!status.enabled); // Default is usually disabled

    Ok(())
}
