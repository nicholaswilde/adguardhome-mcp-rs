use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::settings::Settings;
use anyhow::Result;
use testcontainers::GenericImage;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use tokio::io::AsyncBufReadExt;

#[tokio::test]
async fn test_adguardhome_connectivity() -> Result<()> {
    // Skip if RUN_DOCKER_TESTS is not set to true in CI
    if std::env::var("CI").is_ok()
        && std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true"
    {
        println!("Skipping Docker integration test (RUN_DOCKER_TESTS not set to true)");
        return Ok(());
    }

    println!("🐳 Starting AdGuard Home container...");

    let image = GenericImage::new("adguard/adguardhome", "latest")
        .with_exposed_port(ContainerPort::Tcp(80))
        .with_wait_for(WaitFor::message_on_stdout("AdGuard Home is available at"));

    let container = match image.start().await {
        Ok(c) => c,
        Err(e) => {
            println!(
                "⚠️ Failed to start Docker container. This is expected in environments without Docker. Skipping integration test. Error: {}",
                e
            );
            return Ok(());
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

    // Initialize the client
    let settings = Settings {
        adguard_url: base_url.clone(),
        adguard_username: None,
        adguard_password: None,
    };
    let client = AdGuardClient::new(settings);

    // Testing get_status
    println!("Testing get_status...");
    // AdGuard Home might need a moment to fully initialize its internal API even after the log message
    let mut status = None;
    for _ in 0..5 {
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