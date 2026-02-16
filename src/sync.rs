use crate::adguard::AdGuardClient;
use crate::adguard::models::{AdGuardClientDevice, DnsConfig, DnsRewrite, FilteringConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncState {
    pub filtering: FilteringConfig,
    pub clients: Vec<AdGuardClientDevice>,
    pub dns: DnsConfig,
    pub blocked_services: Vec<String>,
    pub rewrites: Vec<DnsRewrite>,
}

impl SyncState {
    pub async fn fetch(client: &AdGuardClient) -> Result<Self> {
        let filtering = client.list_filters().await?;
        let clients = client.list_clients().await?;
        let dns = client.get_dns_info().await?;
        let blocked_services = client.list_blocked_services().await?;
        let rewrites = client.list_rewrites().await?;

        Ok(Self {
            filtering,
            clients,
            dns,
            blocked_services,
            rewrites,
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

        // TODO: Implement other modules (Clients, DNS, Filters) in subsequent steps if needed,
        // but this covers the failing test case for rules, services, and rewrites.

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

        master_state
            .push_to_replica(&client, "full-overwrite")
            .await
            .unwrap();
    }
}
