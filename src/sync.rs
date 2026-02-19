use crate::adguard::AdGuardClient;
use crate::adguard::models::{
    AccessList, AdGuardClientDevice, DhcpStatus, DnsConfig, DnsRewrite, FilteringConfig,
    ParentalControlConfig, ProfileInfo, QueryLogConfig, SafeSearchConfig, TlsConfig,
};
use crate::config::AppConfig;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupMetadata {
    pub version: String,
    pub timestamp: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncState {
    pub metadata: Option<BackupMetadata>,
    pub filtering: FilteringConfig,
    pub clients: Vec<AdGuardClientDevice>,
    pub dns: DnsConfig,
    pub blocked_services: Vec<String>,
    pub rewrites: Vec<DnsRewrite>,
    pub access_list: AccessList,
    pub query_log_config: QueryLogConfig,
    pub safe_search: SafeSearchConfig,
    pub safe_browsing: bool,
    pub parental_control: ParentalControlConfig,
    pub dhcp: DhcpStatus,
    pub tls: TlsConfig,
    pub profile_info: ProfileInfo,
}

impl SyncState {
    pub async fn run_background_sync(config: AppConfig) {
        if config.replicas.is_empty() {
            tracing::info!("No replicas configured, skipping background sync.");
            return;
        }

        let mut interval = interval(Duration::from_secs(config.sync_interval_seconds));
        let master_client = AdGuardClient::new(config.clone());

        loop {
            interval.tick().await;
            tracing::info!("Starting background synchronization...");

            match Self::fetch(&master_client).await {
                Ok(state) => {
                    for replica in &config.replicas {
                        let mut replica_config = config.clone();
                        let url = replica.url.clone();
                        match url::Url::parse(&url) {
                            Ok(parsed_url) => {
                                replica_config.adguard_host =
                                    parsed_url.host_str().unwrap_or("localhost").to_string();
                                replica_config.adguard_port = parsed_url.port().unwrap_or(80);
                                replica_config.adguard_username = Some("admin".to_string());
                                replica_config.adguard_password = Some(replica.api_key.clone());

                                let replica_client = AdGuardClient::new(replica_config);
                                if let Err(e) = state
                                    .push_to_replica(&replica_client, &config.default_sync_mode)
                                    .await
                                {
                                    tracing::error!("Failed to sync to replica {}: {}", url, e);
                                } else {
                                    tracing::info!("Successfully synced to replica {}", url);
                                }
                            }
                            Err(e) => tracing::error!("Failed to parse replica URL {}: {}", url, e),
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to fetch master state for sync: {}", e),
            }
        }
    }

    pub async fn fetch(client: &AdGuardClient) -> Result<Self> {
        Self::fetch_full(client, None).await
    }

    pub async fn fetch_full(client: &AdGuardClient, description: Option<String>) -> Result<Self> {
        let filtering = client.list_filters().await?;
        let clients = client.list_clients().await?;
        let dns = client.get_dns_info().await?;
        let blocked_services = client.list_blocked_services().await?;
        let rewrites = client.list_rewrites().await?;
        let access_list = client.get_access_list().await?;
        let query_log_config = client.get_query_log_config().await?;
        let safe_search = client.get_safe_search_settings().await?;
        let status = client.get_status().await?;
        let parental_control = client.get_parental_settings().await?;
        let dhcp = client.get_dhcp_status().await?;
        let tls = client.get_tls_status().await?;
        let profile_info = client.get_profile_info().await?;

        let metadata = Some(BackupMetadata {
            version: status.version.clone(),
            timestamp: Utc::now().to_rfc3339(),
            description,
        });

        Ok(Self {
            metadata,
            filtering,
            clients,
            dns,
            blocked_services,
            rewrites,
            access_list,
            query_log_config,
            safe_search,
            safe_browsing: status.protection_enabled, // Approximation
            parental_control,
            dhcp,
            tls,
            profile_info,
        })
    }

    pub async fn push_to_replica(&self, client: &AdGuardClient, mode: &str) -> Result<()> {
        // 1. Sync User Rules
        if mode == "full-overwrite" {
            client
                .set_user_rules(self.filtering.user_rules.clone())
                .await?;
        } else {
            // Additive: Fetch existing, merge, then set
            let existing_rules = client.get_user_rules().await?;
            let mut merged_rules = existing_rules;
            for rule in &self.filtering.user_rules {
                if !merged_rules.contains(rule) {
                    merged_rules.push(rule.clone());
                }
            }
            client.set_user_rules(merged_rules).await?;
        }

        // 2. Sync Blocked Services
        if mode == "full-overwrite" {
            client
                .set_blocked_services(self.blocked_services.clone())
                .await?;
        } else {
            let existing_services = client.list_blocked_services().await?;
            let mut merged_services = existing_services;
            for service in &self.blocked_services {
                if !merged_services.contains(service) {
                    merged_services.push(service.clone());
                }
            }
            client.set_blocked_services(merged_services).await?;
        }

        // 3. Sync Rewrites
        let existing_rewrites = client.list_rewrites().await?;
        if mode == "full-overwrite" {
            // Remove rewrites not in master
            for rewrite in existing_rewrites {
                let exists_in_master = self
                    .rewrites
                    .iter()
                    .any(|r| r.domain == rewrite.domain && r.answer == rewrite.answer);
                if !exists_in_master {
                    client.delete_rewrite(rewrite).await?;
                }
            }
            // Add all master rewrites (idempotency checks usually handled by API, but we can check existence)
            for rewrite in &self.rewrites {
                client.add_rewrite(rewrite.clone()).await?;
            }
        } else {
            // Additive: Only add missing
            for rewrite in &self.rewrites {
                let exists_in_replica = existing_rewrites
                    .iter()
                    .any(|r| r.domain == rewrite.domain && r.answer == rewrite.answer);
                if !exists_in_replica {
                    client.add_rewrite(rewrite.clone()).await?;
                }
            }
        }

        // 4. Sync DNS Config
        client.set_dns_config(self.dns.clone()).await?;

        // 5. Sync Access List
        client.set_access_list(self.access_list.clone()).await?;

        // 6. Sync Query Log Config
        client
            .set_query_log_config(self.query_log_config.clone())
            .await?;

        // 7. Sync Safe Search
        client
            .set_safe_search_settings(self.safe_search.clone())
            .await?;

        // 8. Sync Parental Control
        client
            .set_parental_settings(self.parental_control.clone())
            .await?;

        // 9. Sync Protection
        client.set_protection(self.safe_browsing).await?;

        // 10. Sync DHCP Config
        client.set_dhcp_config(self.dhcp.clone()).await?;

        // 11. Sync TLS Config
        client.configure_tls(self.tls.clone()).await?;

        // 12. Sync Profile Info
        client.set_profile_info(self.profile_info.clone()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_sync_push_overwrite() {
        use crate::adguard::AdGuardClient;
        use crate::config::AppConfig;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

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
        let client = AdGuardClient::new(config);

        // Master state (Empty)
        let master_state = SyncState {
            metadata: None,
            filtering: FilteringConfig {
                enabled: true,
                interval: 1,
                filters: Vec::new(),
                whitelist_filters: Vec::new(),
                user_rules: Vec::new(),
            },
            clients: Vec::new(),
            dns: DnsConfig {
                upstream_dns: vec![],
                upstream_dns_file: "".to_string(),
                bootstrap_dns: Vec::new(),
                fallback_dns: Vec::new(),
                all_servers: false,
                fastest_addr: false,
                fastest_timeout: 0,
                cache_size: 0,
                cache_ttl_min: 0,
                cache_ttl_max: 0,
                cache_optimistic: false,
                upstream_mode: "".to_string(),
                use_private_ptr_resolvers: false,
                local_ptr_upstreams: Vec::new(),
            },
            blocked_services: Vec::new(),
            rewrites: Vec::new(),
            access_list: AccessList {
                allowed_clients: vec![],
                disallowed_clients: vec![],
                blocked_hosts: vec![],
            },
            query_log_config: QueryLogConfig {
                enabled: true,
                interval: 1,
                anonymize_client_ip: false,
                allowed_clients: vec![],
                disallowed_clients: vec![],
            },
            safe_search: SafeSearchConfig {
                enabled: true,
                bing: true,
                duckduckgo: true,
                google: true,
                pixabay: true,
                yandex: true,
                youtube: true,
            },
            safe_browsing: true,
            parental_control: ParentalControlConfig {
                enabled: true,
                sensitivity: None,
            },
            dhcp: DhcpStatus {
                enabled: false,
                interface_name: "".to_string(),
                v4: None,
                v6: None,
                leases: Vec::new(),
                static_leases: Vec::new(),
            },
            tls: TlsConfig::default(),
            profile_info: ProfileInfo {
                name: "admin".to_string(),
                language: "en".to_string(),
                theme: "dark".to_string(),
            },
        };

        // Mock Replica Current State (Has items)
        Mock::given(method("GET"))
            .and(path("/control/filtering/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "enabled": true,
                "interval": 1,
                "filters": [],
                "whitelist_filters": [],
                "user_rules": ["stale_rule"]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/control/blocked_services/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(["stale_service"])))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/control/rewrite/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                "domain": "stale.com",
                "answer": "1.1.1.1"
            }])))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/control/access/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "allowed_clients": [],
                "disallowed_clients": [],
                "blocked_hosts": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/control/querylog/config"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "enabled": true,
                "interval": 1,
                "anonymize_client_ip": false,
                "allowed_clients": [],
                "disallowed_clients": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/control/safesearch/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
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

        Mock::given(method("GET"))
            .and(path("/control/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "version": "v0.107.0",
                "language": "en",
                "protection_enabled": true
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/control/parental/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "enabled": true
            })))
            .mount(&server)
            .await;

        // Expect calls to overwrite/delete
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

        Mock::given(method("POST"))
            .and(path("/control/rewrite/delete"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/control/dns_config"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/control/access/set"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/control/querylog/config/update"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

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

        Mock::given(method("POST"))
            .and(path("/control/protection"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/control/dhcp/set_config"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/control/tls/configure"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/control/profile/update"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        master_state
            .push_to_replica(&client, "full-overwrite")
            .await
            .unwrap();
    }
}
