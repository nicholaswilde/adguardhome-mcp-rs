use adguardhome_mcp_rs::server::http::run_http_server;
use adguardhome_mcp_rs::server::mcp::McpServer;
use anyhow::Result;
mod common;
use common::AdGuardContainer;

#[tokio::test]
async fn test_mcp_http_transport() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = AdGuardContainer::new(None).await?;
    let adguard_host = container.host.clone();
    let adguard_port = container.port;

    let mut config = adguardhome_mcp_rs::config::AppConfig {
        adguard_host,
        adguard_port,
        mcp_transport: "http".to_string(),
        http_port: 0,
        http_auth_token: Some("test-token".to_string()),
        ..Default::default()
    };
    config.validate().unwrap();

    let registry = adguardhome_mcp_rs::tools::ToolRegistry::new(&config);
    let (server, rx) = McpServer::new(registry, config.clone());

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
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    let container = AdGuardContainer::new(None).await?;
    let status = container.client().get_status().await?;
    assert!(!status.version.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_adguardhome_with_auth() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
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
    std::io::Write::write_all(&mut temp_file, config_content.as_bytes())?;
    let config_path = temp_file.path().to_str().unwrap().to_string();

    let container = AdGuardContainer::new(Some(config_path)).await?;
    let adguard_host = container.host.clone();
    let adguard_port = container.port;

    let mut config_no_auth = adguardhome_mcp_rs::config::AppConfig {
        adguard_host: adguard_host.clone(),
        adguard_port,
        ..Default::default()
    };
    config_no_auth.validate().unwrap();
    let client_no_auth = adguardhome_mcp_rs::adguard::AdGuardClient::new(
        config_no_auth.get_instance(None).unwrap().clone(),
    );

    let mut success = false;
    for _ in 0..5 {
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

    let mut config_auth = adguardhome_mcp_rs::config::AppConfig {
        adguard_host,
        adguard_port,
        adguard_username: Some("admin".to_string()),
        adguard_password: Some("password".to_string()),
        ..Default::default()
    };
    config_auth.validate().unwrap();
    let client_auth = adguardhome_mcp_rs::adguard::AdGuardClient::new(
        config_auth.get_instance(None).unwrap().clone(),
    );

    let status = client_auth.get_status().await?;
    assert!(!status.version.is_empty());

    Ok(())
}
