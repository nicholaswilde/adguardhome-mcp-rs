use crate::adguard::AdGuardClient;
use crate::adguard::models::{
    AccessList, AdGuardClientDevice, DhcpStatus, DnsConfig, DnsRewrite, FilteringConfig,
    ParentalControlConfig, ProfileInfo, QueryLogConfig, SafeSearchConfig, TlsConfig,
};
use crate::config::{AppConfig, InstanceConfig};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub applied_modules: Vec<String>,
    pub failed_modules: Vec<String>,
    pub errors: Vec<String>,
}

impl SyncState {
    pub async fn run_background_sync(config: AppConfig) {
        if config.replicas.is_empty() {
            tracing::info!("No replicas configured, skipping background sync.");
            return;
        }

        let mut interval = interval(Duration::from_secs(config.sync_interval_seconds));
        let master_instance = config.get_instance(None).expect("No instances configured").clone();
        let master_client = AdGuardClient::new(master_instance);

        loop {
            interval.tick().await;
            tracing::info!("Starting background synchronization...");

            match Self::fetch(&master_client).await {
                Ok(state) => {
                    for replica in &config.replicas {
                        let url = replica.url.clone();
                        match url::Url::parse(&url) {
                            Ok(_parsed_url) => {
                                let replica_instance = InstanceConfig {
                                    name: Some("replica".to_string()),
                                    url: url.clone(),
                                    api_key: Some(replica.api_key.clone()),
                                    ..Default::default()
                                };

                                let replica_client = AdGuardClient::new(replica_instance);
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

    pub async fn push_to_replica(&self, client: &AdGuardClient, mode: &str) -> Result<SyncResult> {
        let mut applied = Vec::new();
        let mut failed = Vec::new();
        let mut errors = Vec::new();

        macro_rules! try_sync {
            ($module:expr, $op:expr) => {
                match $op.await {
                    Ok(_) => applied.push($module.to_string()),
                    Err(e) => {
                        failed.push($module.to_string());
                        errors.push(format!("{}: {}", $module, e));
                    }
                }
            };
        }

        // 1. Sync User Rules
        try_sync!("User Rules", async {
            if mode == "full-overwrite" {
                client
                    .set_user_rules(self.filtering.user_rules.clone())
                    .await
            } else {
                let existing_rules = client.get_user_rules().await?;
                let mut merged_rules = existing_rules;
                for rule in &self.filtering.user_rules {
                    if !merged_rules.contains(rule) {
                        merged_rules.push(rule.clone());
                    }
                }
                client.set_user_rules(merged_rules).await
            }
        });

        // 2. Sync Blocked Services
        try_sync!("Blocked Services", async {
            if mode == "full-overwrite" {
                client
                    .set_blocked_services(self.blocked_services.clone())
                    .await
            } else {
                let existing_services = client.list_blocked_services().await?;
                let mut merged_services = existing_services;
                for service in &self.blocked_services {
                    if !merged_services.contains(service) {
                        merged_services.push(service.clone());
                    }
                }
                client.set_blocked_services(merged_services).await
            }
        });

        // 3. Sync Rewrites
        try_sync!("DNS Rewrites", async {
            let existing_rewrites = client.list_rewrites().await?;
            if mode == "full-overwrite" {
                for rewrite in existing_rewrites {
                    let exists_in_master = self
                        .rewrites
                        .iter()
                        .any(|r| r.domain == rewrite.domain && r.answer == rewrite.answer);
                    if !exists_in_master {
                        client.delete_rewrite(rewrite).await?;
                    }
                }
                for rewrite in &self.rewrites {
                    client.add_rewrite(rewrite.clone()).await?;
                }
            } else {
                for rewrite in &self.rewrites {
                    let exists_in_replica = existing_rewrites
                        .iter()
                        .any(|r| r.domain == rewrite.domain && r.answer == rewrite.answer);
                    if !exists_in_replica {
                        client.add_rewrite(rewrite.clone()).await?;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        // 4. Sync DNS Config
        try_sync!("DNS Config", client.set_dns_config(self.dns.clone()));

        // 5. Sync Access List
        try_sync!(
            "Access List",
            client.set_access_list(self.access_list.clone())
        );

        // 6. Sync Query Log Config
        try_sync!(
            "Query Log Config",
            client.set_query_log_config(self.query_log_config.clone())
        );

        // 7. Sync Safe Search
        try_sync!(
            "Safe Search",
            client.set_safe_search_settings(self.safe_search.clone())
        );

        // 8. Sync Parental Control
        try_sync!(
            "Parental Control",
            client.set_parental_settings(self.parental_control.clone())
        );

        // 9. Sync Protection
        try_sync!(
            "Global Protection",
            client.set_protection(self.safe_browsing)
        );

        // 10. Sync DHCP Config
        try_sync!("DHCP Config", client.set_dhcp_config(self.dhcp.clone()));

        // 11. Sync TLS Config
        try_sync!("TLS Config", client.configure_tls(self.tls.clone()));

        // 12. Sync Profile Info
        try_sync!(
            "Profile Info",
            client.set_profile_info(self.profile_info.clone())
        );

        let success = failed.is_empty();
        Ok(SyncResult {
            success,
            applied_modules: applied,
            failed_modules: failed,
            errors,
        })
    }

    pub fn diff(&self, other: &Self) -> String {
        let mut changes = Vec::new();

        if self.filtering.enabled != other.filtering.enabled {
            changes.push(format!(
                "Filtering: Enabled: {} -> {}",
                other.filtering.enabled, self.filtering.enabled
            ));
        }

        if self.filtering.user_rules != other.filtering.user_rules {
            changes.push("User Rules: Changed".to_string());
        }

        if self.dns.upstream_dns != other.dns.upstream_dns {
            changes.push("DNS: Upstream DNS changed".to_string());
        }

        if self.blocked_services != other.blocked_services {
            changes.push("Blocked Services: Changed".to_string());
        }

        if self.safe_browsing != other.safe_browsing {
            changes.push(format!(
                "Safe Browsing: {} -> {}",
                other.safe_browsing, self.safe_browsing
            ));
        }

        if self.parental_control.enabled != other.parental_control.enabled {
            changes.push(format!(
                "Parental Control: {} -> {}",
                other.parental_control.enabled, self.parental_control.enabled
            ));
        }

        if self.dhcp.enabled != other.dhcp.enabled {
            changes.push(format!(
                "DHCP: {} -> {}",
                other.dhcp.enabled, self.dhcp.enabled
            ));
        }

        if self.tls.enabled != other.tls.enabled {
            changes.push(format!(
                "TLS: {} -> {}",
                other.tls.enabled, self.tls.enabled
            ));
        }

        if changes.is_empty() {
            "No significant changes detected.".to_string()
        } else {
            changes.join("\n")
        }
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
        let mut config = AppConfig {
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
        config.validate().unwrap();
        let client = AdGuardClient::new(config.get_instance(None).unwrap().clone());

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

    #[tokio::test]
    async fn test_sync_state_diff() {
        use crate::adguard::models::*;
        let state1 = SyncState {
            metadata: None,
            filtering: FilteringConfig {
                enabled: true,
                interval: 1,
                filters: vec![],
                whitelist_filters: vec![],
                user_rules: vec!["rule1".to_string()],
            },
            clients: vec![],
            dns: DnsConfig {
                upstream_dns: vec!["1.1.1.1".to_string()],
                upstream_dns_file: "".to_string(),
                bootstrap_dns: vec![],
                fallback_dns: vec![],
                all_servers: false,
                fastest_addr: false,
                fastest_timeout: 0,
                cache_size: 0,
                cache_ttl_min: 0,
                cache_ttl_max: 0,
                cache_optimistic: false,
                upstream_mode: "".to_string(),
                use_private_ptr_resolvers: false,
                local_ptr_upstreams: vec![],
            },
            blocked_services: vec!["youtube".to_string()],
            rewrites: vec![],
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

        let mut state2 = state1.clone();
        state2.filtering.enabled = false;
        state2.dns.upstream_dns = vec!["8.8.8.8".to_string()];
        state2.blocked_services = vec!["facebook".to_string()];
        state2.filtering.user_rules = vec!["rule2".to_string()];

        let diff = state1.diff(&state2);
        assert!(diff.contains("Filtering: Enabled: false -> true"));
        assert!(diff.contains("DNS: Upstream DNS changed"));
        assert!(diff.contains("Blocked Services: Changed"));
        assert!(diff.contains("User Rules: Changed"));
    }

    #[tokio::test]
    async fn test_push_to_replica_partial_success() {
        use crate::adguard::AdGuardClient;
        use crate::config::AppConfig;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let mut config = AppConfig {
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
        config.validate().unwrap();
        let client = AdGuardClient::new(config.get_instance(None).unwrap().clone());

        let state = SyncState {
            metadata: None,
            filtering: FilteringConfig {
                enabled: true,
                interval: 1,
                filters: vec![],
                whitelist_filters: vec![],
                user_rules: vec![],
            },
            clients: vec![],
            dns: DnsConfig::default(),
            blocked_services: vec![],
            rewrites: vec![],
            access_list: AccessList::default(),
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

        // 1. Success Mock
        Mock::given(method("POST"))
            .and(path("/control/filtering/set_rules"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        // 2. Failure Mock
        Mock::given(method("POST"))
            .and(path("/control/blocked_services/set"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let result = state
            .push_to_replica(&client, "full-overwrite")
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.applied_modules.contains(&"User Rules".to_string()));
        assert!(
            result
                .failed_modules
                .contains(&"Blocked Services".to_string())
        );
        assert!(!result.errors.is_empty());
    }
}
