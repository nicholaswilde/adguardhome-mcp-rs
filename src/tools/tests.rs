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
        ..Default::default()
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
        .call_tool(
            "manage_dns",
            &client,
            Some(json!({"action": "list_rewrites"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/rewrite/add"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_dns",
            &client,
            Some(json!({"action": "add_rewrite", "domain": "a", "answer": "b"})),
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
            "manage_dns",
            &client,
            Some(json!({"action": "remove_rewrite", "domain": "a", "answer": "b"})),
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
        .call_tool("manage_dns", &client, Some(json!({"action": "get_config"})))
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/dns_config"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_dns",
            &client,
            Some(json!({"action": "set_config", "cache_size": 1024})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/cache_clear"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_dns",
            &client,
            Some(json!({"action": "clear_cache"})),
        )
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
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "get_status"})),
        )
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
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "get_stats"})),
        )
        .await
        .unwrap();
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "get_top_blocked_domains"})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "get_query_log"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/stats_reset"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "clear_stats"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/querylog_clear"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "clear_query_log"})),
        )
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
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "get_query_log_config"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/querylog/config"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "set_query_log_config", "enabled": false})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/version_info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"version": "v", "announcement": "", "announcement_url": "", "can_update": false, "new_version": ""})))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "get_version_info"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "update_adguard_home"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/backup"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1]))
        .mount(&server)
        .await;
    let resp = registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "create_backup"})),
        )
        .await
        .unwrap();
    let text = resp["content"][0]["text"].as_str().unwrap();
    if let Some(p) = text.split("Backup: ").nth(1) {
        let _ = std::fs::remove_file(p.trim());
    }

    Mock::given(method("POST"))
        .and(path("/control/restart"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_system",
            &client,
            Some(json!({"action": "restart_service"})),
        )
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
            "manage_system",
            &client,
            Some(json!({"action": "restore_backup", "file_path": "test.tar.gz"})),
        )
        .await
        .unwrap();
    let _ = std::fs::remove_file("test.tar.gz");
}

#[tokio::test]
async fn test_sync_instances_tool() {
    let (server, client, mut registry) = setup().await;
    super::sync::register(&mut registry);

    // Mock Master calls
    Mock::given(method("GET"))
        .and(path("/control/filtering/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "enabled": true, "interval": 1, "filters": [], "whitelist_filters": [], "user_rules": []
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/control/clients"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"clients": []})))
        .mount(&server)
        .await;
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
    Mock::given(method("GET"))
        .and(path("/control/blocked_services/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/control/rewrite/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    // Mock Replica calls (set_rules, set_blocked_services, add_rewrite)
    Mock::given(method("POST"))
        .and(path("/control/filtering/set_rules"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/control/blocked_services/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    // Additive mode check existing rewrites
    Mock::given(method("GET"))
        .and(path("/control/rewrite/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    // Call tool with ad-hoc replica (targeting the same mock server)
    let replica_url = format!("http://{}", server.uri().replace("http://", ""));
    let res = registry
        .call_tool(
            "sync_instances",
            &client,
            Some(json!({
                "replicas": [{"url": replica_url, "api_key": "test"}],
                "mode": "additive-merge"
            })),
        )
        .await
        .unwrap();

    assert!(
        res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Successfully synced")
    );
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
            "manage_protection",
            &client,
            Some(json!({"action": "toggle_feature", "feature": "global", "enabled": true})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/safesearch/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_protection",
            &client,
            Some(json!({"action": "toggle_feature", "feature": "safe_search", "enabled": true})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/safebrowsing/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_protection",
            &client,
            Some(json!({"action": "toggle_feature", "feature": "safe_browsing", "enabled": true})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/parental/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_protection",
            &client,
            Some(
                json!({"action": "toggle_feature", "feature": "parental_control", "enabled": true}),
            ),
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
        .call_tool(
            "manage_protection",
            &client,
            Some(json!({"action": "get_config"})),
        )
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
            "manage_protection",
            &client,
            Some(json!({
                "action": "set_config",
                "safe_search": {"enabled": true},
                "parental_control": {"enabled": true}
            })),
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
        .call_tool(
            "manage_protection",
            &client,
            Some(json!({"action": "get_tls_config"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/tls/configure"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_protection",
            &client,
            Some(json!({"action": "set_tls_config", "enabled": true})),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_filtering_tools() {
    let (server, client, mut registry) = setup().await;
    super::filtering::register(&mut registry);

    Mock::given(method("GET"))
        .and(path("/control/filtering/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"enabled": true, "interval": 1, "filters": [{"url": "a", "name": "a", "id": 1, "enabled": true, "rules_count": 1}], "whitelist_filters": [], "user_rules": ["rule1"]})))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "list_filters"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/filtering/add_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "add_filter", "name": "a", "url": "b"})),
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
            "manage_filtering",
            &client,
            Some(json!({"action": "toggle_filter", "identifier": "a", "enabled": true})),
        )
        .await
        .unwrap();
    // Test filter not found
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "toggle_filter", "identifier": "not-found", "enabled": true})),
        )
        .await
        .unwrap();

    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "update_filter", "identifier": "a", "new_name": "c"})),
        )
        .await
        .unwrap();
    // Test filter not found
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "update_filter", "identifier": "not-found", "new_name": "c"})),
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
            "manage_filtering",
            &client,
            Some(json!({"action": "remove_filter", "identifier": "a"})),
        )
        .await
        .unwrap();
    // Test filter not found
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "remove_filter", "identifier": "not-found"})),
        )
        .await
        .unwrap();

    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "list_custom_rules"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/filtering/set_rules"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "set_custom_rules", "rules": ["a"]})),
        )
        .await
        .unwrap();
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "add_custom_rule", "rule": "new"})),
        )
        .await
        .unwrap();
    // Test rule already exists
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "add_custom_rule", "rule": "rule1"})),
        )
        .await
        .unwrap();

    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "remove_custom_rule", "rule": "rule1"})),
        )
        .await
        .unwrap();
    // Test rule not found
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "remove_custom_rule", "rule": "not-found"})),
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
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "list_blocked_services"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/blocked_services/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "toggle_blocked_service", "service_id": "youtube", "blocked": true})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/filtering/check_host"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"reason": "a"})))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_filtering",
            &client,
            Some(json!({"action": "check_host", "domain": "a"})),
        )
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
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "list_clients"})),
        )
        .await
        .unwrap();
    registry
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "get_client_info", "identifier": "Test Client"})),
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
            "manage_clients",
            &client,
            Some(json!({"action": "add_client", "name": "a", "ids": ["b"]})),
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
            "manage_clients",
            &client,
            Some(json!({"action": "update_client", "old_name": "Test Client", "name": "New"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/clients/delete"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "delete_client", "name": "a"})),
        )
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "get_activity_report", "identifier": "a"})),
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
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "get_access_list"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/access/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "update_access_list", "allowed_clients": ["a"]})),
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
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "list_dhcp_leases"})),
        )
        .await
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/control/dhcp/add_static_lease"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    registry
        .call_tool(
            "manage_clients",
            &client,
            Some(json!({"action": "add_static_lease", "mac": "a", "ip": "b", "hostname": "c"})),
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
            "manage_clients",
            &client,
            Some(json!({"action": "remove_static_lease", "mac": "a", "ip": "b", "hostname": "c"})),
        )
        .await
        .unwrap();
}
