use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::settings::Settings;
use anyhow::Result;
use std::io::Write;
use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::{GenericImage, ImageExt};
use testcontainers::runners::AsyncRunner;
use tokio::io::AsyncBufReadExt;

async fn start_adguard_container(
    config_path: Option<String>,
) -> Result<(
    testcontainers::ContainerAsync<GenericImage>,
    String,
)> {
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
    let base_url = format!("http://localhost:{}", port);

    println!("✅ AdGuard Home container started at {}", base_url);

    Ok((container, base_url))
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

    let (_container, base_url) = match start_adguard_container(None).await {
        Ok(res) => res,
        Err(_) => return Ok(()), // Skip if failed to start (e.g. no docker)
    };

    // Initialize the client
    let settings = Settings {
        adguard_url: base_url.clone(),
        adguard_username: None,
        adguard_password: None,
        lazy_mode: false,
    };
    let client = AdGuardClient::new(settings);

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

    let (_container, base_url) = match start_adguard_container(Some(config_path)).await {
        Ok(res) => res,
        Err(_) => return Ok(()),
    };

    // 1. Test without credentials (expect failure)
    let settings_no_auth = Settings {
        adguard_url: base_url.clone(),
        adguard_username: None,
        adguard_password: None,
        lazy_mode: false,
    };
    let client_no_auth = AdGuardClient::new(settings_no_auth);

    println!("Testing get_status (Expected Failure)...");
    let mut success = false;
    // We expect it to fail, but we need to wait for it to be ready first.
    // However, if we can't auth, we can't check readiness via status.
    // We assume the log message "AdGuard Home is available" in start_adguard_container meant it's up.
    // But sometimes it takes a moment.
    
    // We'll try to call it. If it returns 401, we know it's reachable and enforcing auth.
    // If it returns connection error, we might need to retry.
    for _ in 0..10 {
        match client_no_auth.get_status().await {
            Ok(_) => {
                panic!("Expected authentication failure, but got success");
            }
            Err(e) => {
                // Check if it is a 401 Unauthorized
                // e is anyhow::Error, we need to downcast or check string representation
                let err_msg = e.to_string();
                if err_msg.contains("401") {
                    println!("✅ Got expected 401 Unauthorized");
                    success = true;
                    break;
                } else if err_msg.contains("connect") || err_msg.contains("receive") {
                     // Still starting up?
                     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                     continue;
                } else {
                    println!("Got unexpected error: {}", err_msg);
                     // Might be 403 or something else?
                     // AdGuard Home usually returns 401 for unauthorized.
                     // We'll retry a bit in case it's a transient startup error.
                     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
    assert!(success, "Did not receive 401 Unauthorized as expected");

    // 2. Test with credentials (expect success)
    let settings_auth = Settings {
        adguard_url: base_url.clone(),
        adguard_username: Some("admin".to_string()),
        adguard_password: Some("password".to_string()),
        lazy_mode: false,
    };
    let client_auth = AdGuardClient::new(settings_auth);

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