use super::*;
use crate::config::AppConfig;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_config(host: String, port: u16) -> AppConfig {
    AppConfig {
        adguard_host: host,
        adguard_port: port,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_error_handling_404() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/status"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let result = client.get_status().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_500() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/protection"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = client.set_protection(true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_status() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "v0.107.0",
            "language": "en",
            "dns_addresses": ["127.0.0.1"],
            "dns_port": 53,
            "http_port": 80,
            "protection_enabled": true,
            "dhcp_available": true,
            "running": true
        })))
        .mount(&server)
        .await;

    let status = client.get_status().await.unwrap();
    assert_eq!(status.version, "v0.107.0");
    assert!(status.protection_enabled);
}

#[tokio::test]
async fn test_configure_tls() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/tls/configure"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let tls_config = TlsConfig {
        enabled: true,
        server_name: "example.com".to_string(),
        force_https: false,
        port_https: 443,
        port_dns_over_tls: 853,
        port_dns_over_quic: 853,
        certificate_chain: "".to_string(),
        private_key: "".to_string(),
        certificate_path: "".to_string(),
        private_key_path: "".to_string(),
        valid_cert: true,
        valid_key: true,
        valid_pair: true,
    };
    client.configure_tls(tls_config).await.unwrap();
}

#[tokio::test]
async fn test_validate_tls() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/tls/validate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "server_name": "example.com",
            "force_https": false,
            "port_https": 443,
            "port_dns_over_tls": 853,
            "port_dns_over_quic": 853,
            "certificate_chain": "",
            "private_key": "",
            "certificate_path": "",
            "private_key_path": "",
            "valid_cert": true,
            "valid_key": true,
            "valid_pair": true
        })))
        .mount(&server)
        .await;

    let tls_config = TlsConfig {
        enabled: true,
        server_name: "example.com".to_string(),
        force_https: false,
        port_https: 443,
        port_dns_over_tls: 853,
        port_dns_over_quic: 853,
        certificate_chain: "".to_string(),
        private_key: "".to_string(),
        certificate_path: "".to_string(),
        private_key_path: "".to_string(),
        valid_cert: true,
        valid_key: true,
        valid_pair: true,
    };
    let result = client.validate_tls(tls_config).await.unwrap();
    assert!(result.valid_cert);
}

#[tokio::test]
async fn test_get_client_info() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/clients"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "clients": [
                {
                    "name": "Test Client",
                    "ids": ["192.168.1.100"],
                    "use_global_settings": true,
                    "filtering_enabled": true,
                    "parental_enabled": false,
                    "safebrowsing_enabled": true,
                    "safesearch_enabled": false
                }
            ]
        })))
        .mount(&server)
        .await;

    let info = client.get_client_info("Test Client").await.unwrap();
    assert_eq!(info.name, "Test Client");

    let info_by_id = client.get_client_info("192.168.1.100").await.unwrap();
    assert_eq!(info_by_id.name, "Test Client");

    let result = client.get_client_info("NonExistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_version_info_error() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/version_info"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = client.get_version_info().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_adguard_home_error() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/update"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = client.update_adguard_home().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_filter_error() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/add_url"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&server)
        .await;

    let result = client
        .add_filter("Name".to_string(), "url".to_string(), false)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reset_stats() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/stats_reset"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.reset_stats().await.unwrap();
}

#[tokio::test]
async fn test_clear_query_log() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/querylog_clear"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.clear_query_log().await.unwrap();
}

#[tokio::test]
async fn test_list_rewrites() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/rewrite/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![DnsRewrite {
            domain: "example.com".to_string(),
            answer: "1.2.3.4".to_string(),
        }]))
        .mount(&server)
        .await;

    let rewrites = client.list_rewrites().await.unwrap();
    assert_eq!(rewrites.len(), 1);
    assert_eq!(rewrites[0].domain, "example.com");
}

#[tokio::test]
async fn test_add_rewrite() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/rewrite/add"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let rewrite = DnsRewrite {
        domain: "example.com".to_string(),
        answer: "1.2.3.4".to_string(),
    };
    client.add_rewrite(rewrite).await.unwrap();
}

#[tokio::test]
async fn test_delete_rewrite() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/rewrite/delete"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let rewrite = DnsRewrite {
        domain: "example.com".to_string(),
        answer: "1.2.3.4".to_string(),
    };
    client.delete_rewrite(rewrite).await.unwrap();
}

#[tokio::test]
async fn test_get_stats() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "num_dns_queries": 100,
            "num_blocked_filtering": 10,
            "num_replaced_safebrowsing": 5,
            "num_replaced_safesearch": 2,
            "num_replaced_parental": 1,
            "avg_processing_time": 0.05,
            "top_queried_domains": [{"google.com": 50}],
            "top_blocked_domains": [{"doubleclick.net": 10}],
            "top_clients": [{"192.168.1.100": 100}]
        })))
        .mount(&server)
        .await;

    let stats = client.get_stats(None).await.unwrap();
    assert_eq!(stats.num_dns_queries, 100);
    assert_eq!(stats.num_blocked_filtering, 10);
}

#[tokio::test]
async fn test_client_with_auth_execution() {
    let server = MockServer::start().await;
    let mut config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    config.adguard_username = Some("user".to_string());
    config.adguard_password = Some("pass".to_string());

    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/status"))
        .and(wiremock::matchers::header(
            "Authorization",
            "Basic dXNlcjpwYXNz",
        )) // user:pass in base64
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "v0.107.0",
            "language": "en",
            "protection_enabled": true
        })))
        .mount(&server)
        .await;

    client.get_status().await.unwrap();
}

#[tokio::test]
async fn test_get_query_log() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "client": "127.0.0.1",
                    "elapsed_ms": "0.1",
                    "reason": "NotFilteredNotFound",
                    "status": "NOERROR",
                    "time": "2021-01-01T00:00:00Z",
                    "question": {
                        "name": "google.com",
                        "type": "A"
                    }
                }
            ]
        })))
        .mount(&server)
        .await;

    let log = client.get_query_log(None, None, None).await.unwrap();
    assert_eq!(log.data.len(), 1);
    assert_eq!(log.data[0].question.name, "google.com");
}

#[tokio::test]
async fn test_get_query_log_with_params() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .and(wiremock::matchers::query_param("search", "google"))
        .and(wiremock::matchers::query_param("filter", "all"))
        .and(wiremock::matchers::query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": []
        })))
        .mount(&server)
        .await;

    let log = client
        .get_query_log(Some("google"), Some("all"), Some(10))
        .await
        .unwrap();
    assert_eq!(log.data.len(), 0);
}

#[tokio::test]
async fn test_get_stats_with_period() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/stats"))
        .and(wiremock::matchers::query_param("time_period", "1d"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "num_dns_queries": 100,
            "num_blocked_filtering": 10,
            "num_replaced_safebrowsing": 5,
            "num_replaced_safesearch": 2,
            "num_replaced_parental": 1,
            "avg_processing_time": 0.05,
            "top_queried_domains": [],
            "top_blocked_domains": [],
            "top_clients": []
        })))
        .mount(&server)
        .await;

    client.get_stats(Some("1d")).await.unwrap();
}

#[tokio::test]
async fn test_set_parental_settings_disabled() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/parental/disable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let settings = ParentalControlConfig {
        enabled: false,
        sensitivity: None,
    };
    client.set_parental_settings(settings).await.unwrap();
}

#[tokio::test]
async fn test_set_safe_search_disabled() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/safesearch/disable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_safe_search(false).await.unwrap();
}

#[tokio::test]
async fn test_set_safe_browsing_disabled() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/safebrowsing/disable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_safe_browsing(false).await.unwrap();
}

#[tokio::test]
async fn test_set_parental_control_disabled() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/parental/disable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_parental_control(false).await.unwrap();
}

#[tokio::test]
async fn test_set_protection_disabled() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/protection"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_protection(false).await.unwrap();
}

#[tokio::test]
async fn test_check_host_with_client() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/filtering/check_host"))
        .and(wiremock::matchers::query_param("name", "example.com"))
        .and(wiremock::matchers::query_param("client", "1.2.3.4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reason": "NotFilteredNotFound"
        })))
        .mount(&server)
        .await;

    client
        .check_host("example.com", Some("1.2.3.4"))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_client_with_auth() {
    let config = AppConfig {
        adguard_host: "localhost".to_string(),
        adguard_port: 80,
        adguard_username: Some("user".to_string()),
        adguard_password: Some("pass".to_string()),
        ..Default::default()
    };
    let _client = AdGuardClient::new(config);
    // This just tests the constructor logic for auth
}

#[tokio::test]
async fn test_set_protection() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/protection"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_protection(true).await.unwrap();
}

#[tokio::test]
async fn test_set_safe_search() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/safesearch/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_safe_search(true).await.unwrap();
}

#[tokio::test]
async fn test_set_safe_browsing() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/safebrowsing/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_safe_browsing(true).await.unwrap();
}

#[tokio::test]
async fn test_set_parental_control() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/parental/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.set_parental_control(true).await.unwrap();
}

#[tokio::test]
async fn test_list_filters() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/filtering/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "interval": 1,
            "filters": [
                {
                    "url": "https://example.com/filter.txt",
                    "name": "Example Filter",
                    "id": 1,
                    "enabled": true,
                    "last_updated": "2021-01-01T00:00:00Z",
                    "rules_count": 100
                }
            ],
            "whitelist_filters": [],
            "user_rules": []
        })))
        .mount(&server)
        .await;

    let filtering = client.list_filters().await.unwrap();
    assert!(filtering.enabled);
    assert_eq!(filtering.filters.len(), 1);
    assert_eq!(filtering.filters[0].name, "Example Filter");
}

#[tokio::test]
async fn test_add_filter() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/add_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .add_filter(
            "New Filter".to_string(),
            "https://example.com/new.txt".to_string(),
            false,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_toggle_filter() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/set_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .toggle_filter(
            "https://example.com/filter.txt".to_string(),
            "Example Filter".to_string(),
            false,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_remove_filter() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/remove_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .remove_filter("https://example.com/filter.txt".to_string(), false)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_update_filter() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/set_url"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .update_filter(
            "https://example.com/old.txt".to_string(),
            "https://example.com/new.txt".to_string(),
            "New Name".to_string(),
            false,
            true,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_safe_search_settings() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/safesearch/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "bing": true,
            "duckduckgo": true,
            "google": true,
            "pixabay": true,
            "yandex": true,
            "youtube": true
        })))
        .mount(&server)
        .await;

    let settings = client.get_safe_search_settings().await.unwrap();
    assert!(settings.enabled);
    assert!(settings.google);
}

#[tokio::test]
async fn test_set_safe_search_settings() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("PUT"))
        .and(path("/control/safesearch/settings"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let settings = SafeSearchConfig {
        enabled: true,
        bing: true,
        duckduckgo: true,
        google: true,
        pixabay: true,
        yandex: true,
        youtube: true,
    };
    client.set_safe_search_settings(settings).await.unwrap();
}

#[tokio::test]
async fn test_get_parental_settings() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/parental/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "sensitivity": 0
        })))
        .mount(&server)
        .await;

    let settings = client.get_parental_settings().await.unwrap();
    assert!(settings.enabled);
    assert_eq!(settings.sensitivity, Some(0));
}

#[tokio::test]
async fn test_set_parental_settings() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/parental/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let settings = ParentalControlConfig {
        enabled: true,
        sensitivity: None,
    };
    client.set_parental_settings(settings).await.unwrap();
}

#[tokio::test]
async fn test_get_query_log_config() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/querylog/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "interval": 2160,
            "anonymize_client_ip": false,
            "allowed_clients": [],
            "disallowed_clients": []
        })))
        .mount(&server)
        .await;

    let config = client.get_query_log_config().await.unwrap();
    assert!(config.enabled);
    assert_eq!(config.interval, 2160);
}

#[tokio::test]
async fn test_set_query_log_config() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("PUT"))
        .and(path("/control/querylog/config/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let config = QueryLogConfig {
        enabled: true,
        interval: 2160,
        anonymize_client_ip: false,
        allowed_clients: vec![],
        disallowed_clients: vec![],
    };
    client.set_query_log_config(config).await.unwrap();
}

#[tokio::test]
async fn test_get_version_info() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "v0.107.0",
            "language": "en",
            "protection_enabled": true
        })))
        .mount(&server)
        .await;

    let info = client.get_version_info().await.unwrap();
    assert_eq!(info.version, "v0.107.0");
}

#[tokio::test]
async fn test_update_adguard_home() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/version_info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "v0.107.0",
            "announcement": "New version released!",
            "announcement_url": "https://example.com",
            "can_update": true,
            "new_version": "v0.108.0"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/control/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.update_adguard_home().await.unwrap();
}

#[tokio::test]
async fn test_list_clients() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/clients"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "clients": [
                {
                    "name": "Test Client",
                    "ids": ["192.168.1.100"],
                    "use_global_settings": true,
                    "filtering_enabled": true,
                    "parental_enabled": false,
                    "safebrowsing_enabled": true,
                    "safesearch_enabled": false
                }
            ]
        })))
        .mount(&server)
        .await;

    let clients = client.list_clients().await.unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].name, "Test Client");
}

#[tokio::test]
async fn test_get_user_rules() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/filtering/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "interval": 1,
            "filters": [],
            "whitelist_filters": [],
            "user_rules": ["rule1", "rule2"]
        })))
        .mount(&server)
        .await;

    let rules = client.get_user_rules().await.unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0], "rule1");
}

#[tokio::test]
async fn test_set_user_rules() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/set_rules"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .set_user_rules(vec!["rule1".to_string()])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_list_all_services() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/blocked_services/all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "services": [
                { "id": "youtube", "name": "YouTube" },
                { "id": "facebook", "name": "Facebook" }
            ]
        })))
        .mount(&server)
        .await;

    let services = client.list_all_services().await.unwrap();
    assert_eq!(services.len(), 2);
    assert_eq!(services[0].id, "youtube");
}

#[tokio::test]
async fn test_list_blocked_services() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/blocked_services/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec!["youtube"]))
        .mount(&server)
        .await;

    let blocked = client.list_blocked_services().await.unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0], "youtube");
}

#[tokio::test]
async fn test_set_blocked_services() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/blocked_services/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .set_blocked_services(vec!["youtube".to_string()])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_add_client() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/clients/add"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let device = AdGuardClientDevice {
        name: "New Client".to_string(),
        ids: vec!["1.2.3.4".to_string()],
        use_global_settings: true,
        filtering_enabled: true,
        parental_enabled: false,
        safebrowsing_enabled: true,
        safesearch_enabled: false,
    };
    client.add_client(device).await.unwrap();
}

#[tokio::test]
async fn test_update_client() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/clients/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let device = AdGuardClientDevice {
        name: "Updated Client".to_string(),
        ids: vec!["1.2.3.4".to_string()],
        use_global_settings: true,
        filtering_enabled: true,
        parental_enabled: false,
        safebrowsing_enabled: true,
        safesearch_enabled: false,
    };
    client
        .update_client("Old Client".to_string(), device)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_delete_client() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/clients/delete"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client
        .delete_client("Client to Delete".to_string())
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_dhcp_status() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/dhcp/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "interface_name": "eth0",
            "leases": [
                { "mac": "00:11:22:33:44:55", "ip": "192.168.1.50", "hostname": "device1", "expires": "2021-01-01T00:00:00Z" }
            ],
            "static_leases": [
                { "mac": "66:77:88:99:AA:BB", "ip": "192.168.1.10", "hostname": "server1" }
            ]
        })))
        .mount(&server)
        .await;

    let status = client.get_dhcp_status().await.unwrap();
    assert!(status.enabled);
    assert_eq!(status.interface_name, "eth0");
    assert_eq!(status.leases.len(), 1);
    assert_eq!(status.static_leases.len(), 1);
}

#[tokio::test]
async fn test_add_static_lease() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/dhcp/add_static_lease"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let lease = StaticLease {
        mac: "00:11:22:33:44:55".to_string(),
        ip: "192.168.1.10".to_string(),
        hostname: "server1".to_string(),
    };
    client.add_static_lease(lease).await.unwrap();
}

#[tokio::test]
async fn test_remove_static_lease() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/dhcp/remove_static_lease"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let lease = StaticLease {
        mac: "00:11:22:33:44:55".to_string(),
        ip: "192.168.1.10".to_string(),
        hostname: "server1".to_string(),
    };
    client.remove_static_lease(lease).await.unwrap();
}

#[tokio::test]
async fn test_get_dns_info() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/dns_info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "upstream_dns": ["8.8.8.8"],
            "upstream_dns_file": "",
            "bootstrap_dns": ["1.1.1.1"],
            "fallback_dns": [],
            "all_servers": false,
            "fastest_addr": false,
            "fastest_timeout": 0,
            "cache_size": 4096,
            "cache_ttl_min": 0,
            "cache_ttl_max": 0,
            "cache_optimistic": false,
            "upstream_mode": "",
            "use_private_ptr_resolvers": true,
            "local_ptr_upstreams": []
        })))
        .mount(&server)
        .await;

    let dns_info = client.get_dns_info().await.unwrap();
    assert_eq!(dns_info.upstream_dns.len(), 1);
    assert_eq!(dns_info.upstream_dns[0], "8.8.8.8");
}

#[tokio::test]
async fn test_set_dns_config() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/dns_config"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let dns_config = DnsConfig {
        upstream_dns: vec!["8.8.8.8".to_string()],
        upstream_dns_file: "".to_string(),
        bootstrap_dns: vec!["1.1.1.1".to_string()],
        fallback_dns: vec![],
        all_servers: false,
        fastest_addr: false,
        fastest_timeout: 0,
        cache_size: 4096,
        cache_ttl_min: 0,
        cache_ttl_max: 0,
        cache_optimistic: false,
        upstream_mode: "".to_string(),
        use_private_ptr_resolvers: true,
        local_ptr_upstreams: vec![],
    };
    client.set_dns_config(dns_config).await.unwrap();
}

#[tokio::test]
async fn test_clear_dns_cache() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/cache_clear"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.clear_dns_cache().await.unwrap();
}

#[tokio::test]
async fn test_get_access_list() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/access/list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "allowed_clients": ["192.168.1.10"],
            "disallowed_clients": [],
            "blocked_hosts": ["malicious.com"]
        })))
        .mount(&server)
        .await;

    let list = client.get_access_list().await.unwrap();
    assert_eq!(list.allowed_clients.len(), 1);
    assert_eq!(list.blocked_hosts[0], "malicious.com");
}

#[tokio::test]
async fn test_set_access_list() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/access/set"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let list = AccessList {
        allowed_clients: vec!["192.168.1.10".to_string()],
        disallowed_clients: vec![],
        blocked_hosts: vec!["malicious.com".to_string()],
    };
    client.set_access_list(list).await.unwrap();
}

#[tokio::test]
async fn test_check_host() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/filtering/check_host"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "reason": "FilteredBlackList",
            "filter_id": 1,
            "rule": "||example.com^"
        })))
        .mount(&server)
        .await;

    let result = client.check_host("example.com", None).await.unwrap();
    assert_eq!(result.reason, "FilteredBlackList");
    assert_eq!(result.rule.unwrap(), "||example.com^");
}

#[tokio::test]
async fn test_restart_service() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("POST"))
        .and(path("/control/filtering/refresh"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.restart_service().await.unwrap();
}

#[tokio::test]
async fn test_get_tls_status() {
    let server = MockServer::start().await;
    let config = test_config(
        server
            .uri()
            .replace("http://", "")
            .split(':')
            .next()
            .unwrap()
            .to_string(),
        server
            .uri()
            .split(':')
            .next_back()
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = AdGuardClient::new(config);

    Mock::given(method("GET"))
        .and(path("/control/tls/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "enabled": true,
            "server_name": "example.com",
            "force_https": false,
            "port_https": 443,
            "port_dns_over_tls": 853,
            "port_dns_over_quic": 853,
            "certificate_chain": "",
            "private_key": "",
            "certificate_path": "",
            "private_key_path": "",
            "valid_cert": true,
            "valid_key": true,
            "valid_pair": true
        })))
        .mount(&server)
        .await;

    let status = client.get_tls_status().await.unwrap();
    assert!(status.enabled);
    assert_eq!(status.server_name, "example.com");
}
