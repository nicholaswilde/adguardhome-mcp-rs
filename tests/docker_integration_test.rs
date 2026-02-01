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
    // Skip if RUN_DOCKER_TESTS is not set to true in CI
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        println!("Skipping Docker integration test (RUN_DOCKER_TESTS not set to true)");
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
        http_port: 0, // OS assigned
        http_auth_token: Some("test-token".to_string()),
        log_level: "info".to_string(),
        no_verify_ssl: true,
    };

    let adguard_client = AdGuardClient::new(config.clone());
    let registry = ToolRegistry::new(&config);
    let server = McpServer::new(adguard_client, registry, config.clone());

    // Start server on random port
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

    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let client = reqwest::Client::new();
    let mcp_url = format!("http://127.0.0.1:{}", port);

    // 1. Test unauthorized
    let resp = client.get(format!("{}/sse", mcp_url)).send().await?;
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    // 2. Test authorized SSE
    let resp = client
        .get(format!("{}/sse", mcp_url))
        .header("Authorization", "Bearer test-token")
        .send()
        .await?;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    // Check if it's SSE
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/event-stream"
    );

    Ok(())
}

#[tokio::test]
async fn test_adguardhome_no_auth() -> Result<()> {
    // Skip if RUN_DOCKER_TESTS is not set to true in CI
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        println!("Skipping Docker integration test (RUN_DOCKER_TESTS not set to true)");
        return Ok(());
    }

    let (_container, adguard_host, adguard_port) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()), // Skip if failed to start (e.g. no docker)
    };

    // Initialize the client
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

    // Testing get_status
    println!("Testing get_status (No Auth)...");
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
    println!("✅ AdGuard Home Version: {}", status.version);
    assert!(!status.version.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_adguardhome_with_auth() -> Result<()> {
    // Skip if RUN_DOCKER_TESTS is not set to true in CI
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        println!("Skipping Docker integration test (RUN_DOCKER_TESTS not set to true)");
        return Ok(());
    }

    // Create a temporary config file
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

    // 1. Test without credentials (expect failure)
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

    println!("Testing get_status (Expected Failure)...");
    let mut success = false;
    for _ in 0..10 {
        match client_no_auth.get_status().await {
            Ok(_) => {
                panic!("Expected authentication failure, but got success");
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("401") {
                    println!("✅ Got expected 401 Unauthorized");
                    success = true;
                    break;
                } else if err_msg.contains("connect") || err_msg.contains("receive") {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                } else {
                    println!("Got unexpected error: {}", err_msg);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
    assert!(success, "Did not receive 401 Unauthorized as expected");

    // 2. Test with credentials (expect success)
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

    println!("Testing get_status (With Auth)...");
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
    println!("✅ AdGuard Home Version: {}", status.version);
    assert!(!status.version.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_dns_rewrites() -> Result<()> {
    // Skip if RUN_DOCKER_TESTS is not set to true in CI
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        println!("Skipping Docker integration test (RUN_DOCKER_TESTS not set to true)");
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

    // Wait for AdGuard Home to be ready
    let mut ready = false;
    for _ in 0..15 {
        if client.get_status().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(ready, "AdGuard Home did not become ready");

    let rewrite = adguardhome_mcp_rs::adguard::DnsRewrite {
        domain: "test.example.com".to_string(),
        answer: "1.1.1.1".to_string(),
    };

    // 1. Add rewrite
    println!("Adding DNS rewrite...");
    client.add_rewrite(rewrite.clone()).await?;

    // 2. List rewrites
    println!("Listing DNS rewrites...");
    let rewrites = client.list_rewrites().await?;
    let found = rewrites
        .iter()
        .any(|r| r.domain == rewrite.domain && r.answer == rewrite.answer);
    assert!(found, "Added DNS rewrite not found in list");

    // 3. Delete rewrite
    println!("Deleting DNS rewrite...");
    client.delete_rewrite(rewrite.clone()).await?;

    // 4. List rewrites again
    println!("Verifying DNS rewrite deletion...");
    let rewrites = client.list_rewrites().await?;
    let found = rewrites
        .iter()
        .any(|r| r.domain == rewrite.domain && r.answer == rewrite.answer);
    assert!(!found, "Deleted DNS rewrite still found in list");

    Ok(())
}
