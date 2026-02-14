use super::ToolRegistry;
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
async fn test_dns_tools() {
    let (server, client, mut registry) = setup().await;
    super::dns::register(&mut registry);

    Mock::given(method("GET"))
        .and(path("/control/rewrite/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    registry
        .call_tool("list_dns_rewrites", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/rewrite/add"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "add_dns_rewrite",
            &client,
            Some(json!({"domain": "a", "answer": "b"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/rewrite/delete"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "remove_dns_rewrite",
            &client,
            Some(json!({"domain": "a", "answer": "b"})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/dns_info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "upstream_dns": [], "upstream_dns_file": "", "bootstrap_dns": [], "fallback_dns": [],
            "all_servers": false, "fastest_addr": false, "fastest_timeout": 0, "cache_size": 0,
            "cache_ttl_min": 0, "cache_ttl_max": 0, "cache_optimistic": false, "upstream_mode": "",
            "use_private_ptr_resolvers": false, "local_ptr_upstreams": []
        })))
        .mount(&server)
        .await;
    registry
        .call_tool("get_dns_config", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/dns_config"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("set_dns_config", &client, Some(json!({"cache_size": 1024})))
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/cache_clear"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("clear_dns_cache", &client, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_system_tools() {
    let (server, client, mut registry) = setup().await;
    super::system::register(&mut registry);

    Mock::given(method("GET"))
        .and(path("/control/status"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"version": "v", "language": "en", "protection_enabled": true}),
            ),
        )
        .mount(&server)
        .await;
    registry
        .call_tool("get_status", &client, None)
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "num_dns_queries": 0, "num_blocked_filtering": 0, "num_replaced_safebrowsing": 0,
            "num_replaced_safesearch": 0, "num_replaced_parental": 0, "avg_processing_time": 0.0,
            "top_queried_domains": [], "top_blocked_domains": [], "top_clients": []
        })))
        .mount(&server)
        .await;
    registry
        .call_tool("get_stats", &client, None)
        .await
        .unwrap();
    registry
        .call_tool("get_top_blocked_domains", &client, None)
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    registry
        .call_tool("get_query_log", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/stats_reset"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("clear_stats", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/querylog_clear"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("clear_query_log", &client, None)
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/querylog/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "enabled": true, "interval": 1, "anonymize_client_ip": false, "allowed_clients": [], "disallowed_clients": []
        })))
        .mount(&server)
        .await;
    registry
        .call_tool("get_query_log_config", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/querylog/config"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "set_query_log_config",
            &client,
            Some(json!({"enabled": false})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/version_info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"version": "v", "announcement": "", "announcement_url": "", "can_update": false, "new_version": ""})))
        .mount(&server)
        .await;
    registry
        .call_tool("get_version_info", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("update_adguard_home", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/backup"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1]))
        .mount(&server)
        .await;
    let resp = registry
        .call_tool("create_backup", &client, None)
        .await
        .unwrap();
    let text = resp["content"][0]["text"].as_str().unwrap();
    if let Some(p) = text.split("at: ").nth(1) {
        let _ = std::fs::remove_file(p.trim());
    }

    Mock::given(method("POST"))
        .and(path("/control/restart"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("restart_service", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/restore"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let _ = std::fs::write("test.tar.gz", vec![1]);
    registry
        .call_tool(
            "restore_backup",
            &client,
            Some(json!({"file_path": "test.tar.gz"})),
        )
        .await
        .unwrap();
    let _ = std::fs::remove_file("test.tar.gz");
}

#[tokio::test]
async fn test_protection_tools() {
    let (server, client, mut registry) = setup().await;
    super::protection::register(&mut registry);

    Mock::given(method("POST"))
        .and(path("/control/protection"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "set_protection_state",
            &client,
            Some(json!({"enabled": true})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/safesearch/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("set_safe_search", &client, Some(json!({"enabled": true})))
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/safebrowsing/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("set_safe_browsing", &client, Some(json!({"enabled": true})))
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/parental/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "set_parental_control",
            &client,
            Some(json!({"enabled": true})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/safesearch/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"enabled": true, "bing": true, "duckduckgo": true, "google": true, "pixabay": true, "yandex": true, "youtube": true})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/control/parental/status"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"enabled": true, "sensitivity": 0})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/control/status"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"version": "v", "language": "en", "protection_enabled": true}),
            ),
        )
        .mount(&server)
        .await;
    registry
        .call_tool("get_protection_config", &client, None)
        .await
        .unwrap();

    Mock::given(method("PUT"))
        .and(path("/control/safesearch/settings"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/control/parental/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "set_protection_config",
            &client,
            Some(json!({"safe_search": {"enabled": true}, "parental_control": {"enabled": true}})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/tls/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "enabled": true, "server_name": "a", "force_https": false, "port_https": 443, "port_dns_over_tls": 853, "port_dns_over_quic": 853,
            "certificate_chain": "", "private_key": "", "certificate_path": "", "private_key_path": "", "valid_cert": true, "valid_key": true, "valid_pair": true
        })))
        .mount(&server)
        .await;
    registry
        .call_tool("get_tls_config", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/tls/configure"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("set_tls_config", &client, Some(json!({"enabled": true})))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_filtering_tools() {
    let (server, client, mut registry) = setup().await;
    super::filtering::register(&mut registry);

    Mock::given(method("GET"))
        .and(path("/control/filtering/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"enabled": true, "interval": 1, "filters": [{"url": "a", "name": "a", "id": 1, "enabled": true, "rules_count": 1}], "whitelist_filters": [], "user_rules": ["rule1"]})))
        .mount(&server)
        .await;
    registry
        .call_tool("list_filter_lists", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/filtering/add_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "add_filter_list",
            &client,
            Some(json!({"name": "a", "url": "b"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/filtering/set_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "toggle_filter_list",
            &client,
            Some(json!({"identifier": "a", "enabled": true})),
        )
        .await
        .unwrap();
    // Test filter not found
    registry
        .call_tool(
            "toggle_filter_list",
            &client,
            Some(json!({"identifier": "not-found", "enabled": true})),
        )
        .await
        .unwrap();

    registry
        .call_tool(
            "update_filter_list",
            &client,
            Some(json!({"identifier": "a", "new_name": "c"})),
        )
        .await
        .unwrap();
    // Test filter not found
    registry
        .call_tool(
            "update_filter_list",
            &client,
            Some(json!({"identifier": "not-found", "new_name": "c"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/filtering/remove_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "remove_filter_list",
            &client,
            Some(json!({"identifier": "a"})),
        )
        .await
        .unwrap();
    // Test filter not found
    registry
        .call_tool(
            "remove_filter_list",
            &client,
            Some(json!({"identifier": "not-found"})),
        )
        .await
        .unwrap();

    registry
        .call_tool("list_custom_rules", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/filtering/set_rules"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("set_custom_rules", &client, Some(json!({"rules": ["a"]})))
        .await
        .unwrap();
    registry
        .call_tool("add_custom_rule", &client, Some(json!({"rule": "new"})))
        .await
        .unwrap();
    // Test rule already exists
    registry
        .call_tool("add_custom_rule", &client, Some(json!({"rule": "rule1"})))
        .await
        .unwrap();

    registry
        .call_tool(
            "remove_custom_rule",
            &client,
            Some(json!({"rule": "rule1"})),
        )
        .await
        .unwrap();
    // Test rule not found
    registry
        .call_tool(
            "remove_custom_rule",
            &client,
            Some(json!({"rule": "not-found"})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/blocked_services/all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"services": []})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/control/blocked_services/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    registry
        .call_tool("list_blocked_services", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/blocked_services/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "toggle_blocked_service",
            &client,
            Some(json!({"service_id": "youtube", "blocked": true})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/filtering/check_host"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"reason": "a"})))
        .mount(&server)
        .await;
    registry
        .call_tool("check_filtering", &client, Some(json!({"domain": "a"})))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_clients_tools() {
    let (server, client, mut registry) = setup().await;
    super::clients::register(&mut registry);

    let client_json = json!({
        "name": "Test Client", "ids": ["1.2.3.4"], "use_global_settings": true, "filtering_enabled": true,
        "parental_enabled": false, "safebrowsing_enabled": true, "safesearch_enabled": false
    });

    Mock::given(method("GET"))
        .and(path("/control/clients"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"clients": [client_json]})))
        .mount(&server)
        .await;
    registry
        .call_tool("list_clients", &client, None)
        .await
        .unwrap();
    registry
        .call_tool(
            "get_client_info",
            &client,
            Some(json!({"identifier": "Test Client"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/clients/add"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "add_client",
            &client,
            Some(json!({"name": "a", "ids": ["b"]})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/clients/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "update_client",
            &client,
            Some(json!({"old_name": "Test Client", "name": "New"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/clients/delete"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool("delete_client", &client, Some(json!({"name": "a"})))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "get_client_activity_report",
            &client,
            Some(json!({"identifier": "a"})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/access/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"allowed_clients": [], "disallowed_clients": [], "blocked_hosts": []}),
        ))
        .mount(&server)
        .await;
    registry
        .call_tool("get_access_list", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/access/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "update_access_list",
            &client,
            Some(json!({"allowed_clients": ["a"]})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/dhcp/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"enabled": false, "interface_name": "", "leases": [], "static_leases": []}),
        ))
        .mount(&server)
        .await;
    registry
        .call_tool("list_dhcp_leases", &client, None)
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/dhcp/add_static_lease"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "add_static_lease",
            &client,
            Some(json!({"mac": "a", "ip": "b", "hostname": "c"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/dhcp/remove_static_lease"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "remove_static_lease",
            &client,
            Some(json!({"mac": "a", "ip": "b", "hostname": "c"})),
        )
        .await
        .unwrap();
}
