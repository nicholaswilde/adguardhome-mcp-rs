#![allow(dead_code)]
use adguardhome_mcp_rs::adguard::AdGuardClient;
use adguardhome_mcp_rs::config::AppConfig;
use adguardhome_mcp_rs::tools::ToolRegistry;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::io::AsyncBufReadExt;

pub struct AdGuardContainer {
    _container: testcontainers::ContainerAsync<GenericImage>,
    _config_dir: Option<tempfile::TempDir>,
    pub host: String,
    pub port: u16,
}

impl AdGuardContainer {
    pub async fn new(config_path: Option<String>) -> Result<Self> {
        println!("🐳 Starting AdGuard Home container...");

        let image = GenericImage::new("adguard/adguardhome", "latest")
            .with_exposed_port(ContainerPort::Tcp(80))
            .with_exposed_port(ContainerPort::Tcp(3000))
            .with_wait_for(WaitFor::message_on_either_std(
                "AdGuard Home is available at",
            ));

        let (container, temp_dir_handle) = if let Some(path) = config_path {
            let temp_dir = tempfile::Builder::new()
                .prefix("adguard-test-")
                .tempdir_in("./target")?;
            std::fs::copy(&path, temp_dir.path().join("AdGuardHome.yaml"))?;
            let c = image
                .with_mount(Mount::bind_mount(
                    std::fs::canonicalize(temp_dir.path())?.to_str().unwrap(),
                    "/opt/adguardhome/conf",
                ))
                .start()
                .await?;
            (c, Some(temp_dir))
        } else {
            // ALWAYS use the stable test config
            let temp_dir = tempfile::Builder::new()
                .prefix("adguard-test-")
                .tempdir_in("./target")?;
            std::fs::copy(
                "tests/common/AdGuardHome.yaml",
                temp_dir.path().join("AdGuardHome.yaml"),
            )?;
            let c = image
                .with_mount(Mount::bind_mount(
                    std::fs::canonicalize(temp_dir.path())?.to_str().unwrap(),
                    "/opt/adguardhome/conf",
                ))
                .start()
                .await?;
            (c, Some(temp_dir))
        };

        // Pipe stdout logs
        let stdout = container.stdout(true);
        let stderr = container.stderr(true);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDOUT: {}", line);
            }
        });
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDERR: {}", line);
            }
        });

        let port_80 = container.get_host_port_ipv4(80).await?;
        let host = "localhost".to_string();

        println!(
            "✅ AdGuard Home container started at http://{}:{}",
            host, port_80
        );

        // Wait for web server to be ready
        let mut config = AppConfig {
            adguard_host: host.clone(),
            adguard_port: port_80,
            adguard_username: Some("admin".to_string()),
            adguard_password: Some("password".to_string()),
            ..Default::default()
        };
        config.validate().unwrap();
        let client = AdGuardClient::new(config.get_instance(None).unwrap().clone());
        let mut ready = false;
        for _ in 0..20 {
            if client.get_status().await.is_ok() {
                ready = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        if !ready {
            return Err(anyhow::anyhow!("AdGuard Home web server timed out"));
        }

        Ok(Self {
            _container: container,
            _config_dir: temp_dir_handle,
            host,
            port: port_80,
        })
    }

    pub fn client(&self) -> AdGuardClient {
        let mut config = self.config();
        config.validate().unwrap();
        AdGuardClient::new(config.get_instance(None).unwrap().clone())
    }

    pub fn config(&self) -> AppConfig {
        AppConfig {
            adguard_host: self.host.clone(),
            adguard_port: self.port,
            adguard_username: Some("admin".to_string()),
            adguard_password: Some("password".to_string()),
            ..Default::default()
        }
    }
}

pub async fn call_mcp_tool(
    registry: &Arc<Mutex<ToolRegistry>>,
    client: &AdGuardClient,
    config: &AppConfig,
    name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    let handler = {
        let reg = registry.lock().unwrap();
        reg.get_tool(name).map(|t| t.handler.clone())
    };

    if let Some(handler) = handler {
        handler(client, config, Some(args))
            .await
            .map_err(|e| anyhow::anyhow!(e))
    } else {
        Err(anyhow::anyhow!("Tool not found: {}", name))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Stdio,
    Http,
}

pub struct TestContext {
    pub adguard_host: String,
    pub adguard_port: u16,
    pub config: AppConfig,
    pub client: AdGuardClient,
    pub registry: Arc<Mutex<ToolRegistry>>,
    pub transport: Transport,
    pub http_url: Option<String>,
    pub auth_token: Option<String>,
}

impl TestContext {
    pub async fn new() -> Result<(Self, AdGuardContainer)> {
        let container = AdGuardContainer::new(None).await?;
        let ctx = Self::from_container(&container, Transport::Stdio).await?;
        Ok((ctx, container))
    }

    pub async fn from_container(
        container: &AdGuardContainer,
        transport: Transport,
    ) -> Result<Self> {
        let mut config = container.config();
        config.validate().unwrap();
        let adguard_client = container.client();
        let registry = Arc::new(Mutex::new(ToolRegistry::new(&config)));
        let auth_token = Some("test-token".to_string());

        let mut http_url = None;

        if transport == Transport::Http {
            use adguardhome_mcp_rs::server::http::run_http_server;
            use adguardhome_mcp_rs::server::mcp::McpServer;

            let (server, rx) =
                McpServer::with_registry(registry.clone(), config.clone());

            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            let host = "127.0.0.1".to_string();
            drop(listener);

            let server_handle = server.clone();
            let auth_token_inner = auth_token.clone();
            tokio::spawn(async move {
                run_http_server(server_handle, rx, &host, port, auth_token_inner)
                    .await
                    .unwrap();
            });

            // Wait for server to start
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            http_url = Some(format!("http://127.0.0.1:{}", port));
        }

        Ok(Self {
            adguard_host: container.host.clone(),
            adguard_port: container.port,
            config,
            client: adguard_client,
            registry,
            transport,
            http_url,
            auth_token,
        })
    }

    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match self.transport {
            Transport::Stdio => call_mcp_tool(&self.registry, &self.client, &self.config, name, args).await,
            Transport::Http => {
                let url = self.http_url.as_ref().expect("HTTP URL not set");
                let client = reqwest::Client::new();

                let resp = client
                    .get(format!("{}/sse", url))
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.auth_token.as_ref().unwrap()),
                    )
                    .send()
                    .await?;

                let mut session_id = String::new();
                let mut resp_stream = resp;

                let mut buffer = String::new();
                for _ in 0..50 {
                    if let Some(chunk) = resp_stream.chunk().await? {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        if let Some(pos) = buffer.find("session_id=") {
                            let start = pos + 11;
                            let end = buffer[start..].find('\n').unwrap_or(buffer[start..].len());
                            session_id = buffer[start..start + end].trim().to_string();
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }

                if session_id.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Failed to get session_id from SSE. Buffer: {}",
                        buffer
                    ));
                }

                let req_id = uuid::Uuid::new_v4().to_string();
                let mcp_req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "method": "call_tool",
                    "params": {
                        "name": name,
                        "arguments": args
                    }
                });

                client
                    .post(format!("{}/message?session_id={}", url, session_id))
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.auth_token.as_ref().unwrap()),
                    )
                    .json(&mcp_req)
                    .send()
                    .await?;

                let mut result = None;
                for _ in 0..100 {
                    if let Some(chunk) = resp_stream.chunk().await? {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        let events: Vec<&str> = buffer.split("\n\n").collect();
                        for event in events {
                            if event.contains("message")
                                && event.contains(&req_id)
                                && let Some(data_pos) = event.find("data: ")
                            {
                                let data_text = &event[data_pos + 6..].trim();
                                if let Ok(mcp_resp) =
                                    serde_json::from_str::<serde_json::Value>(data_text)
                                    && mcp_resp["id"] == req_id
                                {
                                    if let Some(res) = mcp_resp.get("result") {
                                        result = Some(Ok(res.clone()));
                                        break;
                                    } else if let Some(err) = mcp_resp.get("error") {
                                        result = Some(Err(anyhow::anyhow!("MCP Error: {}", err)));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if result.is_some() {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }

                result.ok_or_else(|| {
                    anyhow::anyhow!(
                        "Timeout waiting for MCP response over HTTP. Buffer: {}",
                        buffer
                    )
                })?
            }
        }
    }
}

pub async fn run_agnostic_test<F, Fut>(f: F) -> Result<()>
where
    F: Fn(TestContext) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let container = AdGuardContainer::new(None).await?;

    // Stdio
    println!("🧪 Running test with Stdio transport...");
    let ctx_stdio = TestContext::from_container(&container, Transport::Stdio).await?;
    f(ctx_stdio).await?;

    // Http
    println!("🧪 Running test with Http transport...");
    let ctx_http = TestContext::from_container(&container, Transport::Http).await?;
    f(ctx_http).await?;

    Ok(())
}
