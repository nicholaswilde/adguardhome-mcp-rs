#![allow(dead_code)]
use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::config::AppConfig;
use adguardhome_mcp_rs::tools::ToolRegistry;
use anyhow::Result;
use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::io::AsyncBufReadExt;

pub struct AdGuardContainer {
    _container: testcontainers::ContainerAsync<GenericImage>,
    host: String,
    port: u16,
}

impl AdGuardContainer {
    pub async fn new(config_path: Option<String>) -> Result<Self> {
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
                .await?
        } else {
            image.start().await?
        };

        // Pipe stdout logs to help debugging
        let stdout = container.stdout(true);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDOUT: {}", line);
            }
        });

        let port = container.get_host_port_ipv4(80).await?;
        let host = "127.0.0.1".to_string();

        println!(
            "✅ AdGuard Home container started at http://{}:{}",
            host, port
        );

        Ok(Self {
            _container: container,
            host,
            port,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn client(&self) -> AdGuardClient {
        AdGuardClient::new(self.config())
    }

    pub fn config(&self) -> AppConfig {
        AppConfig {
            adguard_host: self.host.clone(),
            adguard_port: self.port,
            ..Default::default()
        }
    }
}

pub async fn call_mcp_tool(
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

#[allow(dead_code)]
pub enum Transport {
    Stdio,
    Http,
}

pub struct TestContext {
    pub container: AdGuardContainer,
    pub client: AdGuardClient,
    pub registry: ToolRegistry,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        let container = AdGuardContainer::new(None).await?;
        let client = container.client();
        let registry = ToolRegistry::new(&container.config());
        Ok(Self {
            container,
            client,
            registry,
        })
    }

    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        call_mcp_tool(&self.registry, &self.client, name, args).await
    }
}
