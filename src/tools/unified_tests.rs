#[cfg(test)]
mod tests {
    use super::super::ToolRegistry;
    use crate::adguard::AdGuardClient;
    use crate::config::AppConfig;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup() -> (MockServer, AdGuardClient, ToolRegistry) {
        let server = MockServer::start().await;
        let config = AppConfig {
            adguard_host: server
                .uri()
                .replace("http://", "")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            adguard_port: server
                .uri()
                .split(':')
                .next_back()
                .unwrap()
                .parse()
                .unwrap(),
            adguard_username: None,
            adguard_password: None,
            mcp_transport: "stdio".to_string(),
            lazy_mode: false,
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
            no_verify_ssl: true,
        };
        let client = AdGuardClient::new(config.clone());
        let registry = ToolRegistry::new(&config);
        (server, client, registry)
    }

    #[tokio::test]
    async fn test_manage_system_unified() {
        let (server, client, mut registry) = setup().await;
        super::super::system::register(&mut registry);

        // Test get_status action
        Mock::given(method("GET"))
            .and(path("/control/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"version": "v1.2.3", "language": "en", "protection_enabled": true}),
            ))
            .mount(&server)
            .await;

        let result = registry
            .call_tool(
                "manage_system",
                &client,
                Some(json!({"action": "get_status"})),
            )
            .await;

        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(
            val["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("v1.2.3")
        );
    }
}
