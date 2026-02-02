use crate::config::AppConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AdGuardClient {
    pub client: reqwest::Client,
    pub config: AppConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub version: String,
    pub language: String,
    pub protection_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsRewrite {
    pub domain: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub num_dns_queries: u64,
    pub num_blocked_filtering: u64,
    pub num_replaced_safebrowsing: u64,
    pub num_replaced_safesearch: u64,
    pub num_replaced_parental: u64,
    pub avg_processing_time: f64,
    #[serde(default)]
    pub top_queried_domains: Vec<HashMap<String, u64>>,
    #[serde(default)]
    pub top_blocked_domains: Vec<HashMap<String, u64>>,
    #[serde(default)]
    pub top_clients: Vec<HashMap<String, u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLogEntry {
    pub client: String,
    pub elapsed_ms: String,
    pub reason: String,
    pub status: String,
    pub time: String,
    pub question: QueryLogQuestion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLogQuestion {
    pub name: String,
    #[serde(rename = "type")]
    pub qtype: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLogResponse {
    pub data: Vec<QueryLogEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filter {
    pub url: String,
    pub name: String,
    pub id: u64,
    pub enabled: bool,
    pub last_updated: Option<String>,
    pub rules_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilteringConfig {
    pub enabled: bool,
    pub interval: u32,
    pub filters: Vec<Filter>,
    pub whitelist_filters: Vec<Filter>,
    pub user_rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetRulesRequest {
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddFilterRequest {
    pub name: String,
    pub url: String,
    pub whitelist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetFilterUrlRequest {
    pub url: String,
    pub data: SetFilterUrlData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetFilterUrlData {
    pub enabled: bool,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFilterRequest {
    pub url: String,
    pub whitelist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFilterRequest {
    pub url: String,
    pub name: String,
    pub whitelist: bool,
    pub data: UpdateFilterData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFilterData {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdGuardClientDevice {
    pub name: String,
    pub ids: Vec<String>,
    pub use_global_settings: bool,
    pub filtering_enabled: bool,
    pub parental_enabled: bool,
    pub safebrowsing_enabled: bool,
    pub safesearch_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientsResponse {
    pub clients: Vec<AdGuardClientDevice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockedService {
    pub id: String,
    pub name: String,
    pub icon_svg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockedServicesAllResponse {
    pub services: Vec<BlockedService>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetBlockedServicesRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClientRequest {
    pub name: String,
    pub data: AdGuardClientDevice,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteClientRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DhcpLease {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub expires: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StaticLease {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsConfig {
    pub upstream_dns: Vec<String>,
    pub upstream_dns_file: String,
    pub bootstrap_dns: Vec<String>,
    pub fallback_dns: Vec<String>,
    pub all_servers: bool,
    pub fastest_addr: bool,
    pub fastest_timeout: u32,
    pub cache_size: u32,
    pub cache_ttl_min: u32,
    pub cache_ttl_max: u32,
    pub cache_optimistic: bool,
    pub upstream_mode: String,
    pub use_private_ptr_resolvers: bool,
    pub local_ptr_upstreams: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DhcpStatus {
    pub enabled: bool,
    pub interface_name: String,
    pub leases: Vec<DhcpLease>,
    pub static_leases: Vec<StaticLease>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessList {
    pub allowed_clients: Vec<String>,
    pub disallowed_clients: Vec<String>,
    pub blocked_hosts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterCheckResponse {
    pub reason: String,
    pub filter_id: Option<i64>,
    pub rule: Option<String>,
    pub rules: Option<Vec<FilterCheckMatchedRule>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterCheckMatchedRule {
    pub filter_id: i64,
    pub text: String,
}

impl AdGuardClient {
    pub fn new(config: AppConfig) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, config }
    }

    pub async fn get_status(&self) -> Result<Status> {
        let url = format!(
            "http://{}:{}/control/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<Status>().await?;
        Ok(status)
    }

    pub async fn get_stats(&self, time_period: Option<&str>) -> Result<Stats> {
        let mut url = format!(
            "http://{}:{}/control/stats",
            self.config.adguard_host, self.config.adguard_port
        );
        if let Some(period) = time_period {
            url.push_str(&format!("?time_period={}", period));
        }
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let stats = response.json::<Stats>().await?;
        Ok(stats)
    }

    pub async fn get_query_log(
        &self,
        search: Option<&str>,
        filter: Option<&str>,
        limit: Option<u32>,
    ) -> Result<QueryLogResponse> {
        let mut url = format!(
            "http://{}:{}/control/querylog",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut params = Vec::new();
        if let Some(s) = search {
            params.push(format!("search={}", s));
        }
        if let Some(f) = filter {
            params.push(format!("filter={}", f));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let log = response.json::<QueryLogResponse>().await?;
        Ok(log)
    }

    pub async fn list_rewrites(&self) -> Result<Vec<DnsRewrite>> {
        let url = format!(
            "http://{}:{}/control/rewrite/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let rewrites = response.json::<Vec<DnsRewrite>>().await?;
        Ok(rewrites)
    }

    pub async fn add_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/rewrite/add",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&rewrite);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/rewrite/delete",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&rewrite);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_protection(&self, enabled: bool) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/protection",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "enabled": enabled }));

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_safe_search(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!(
            "http://{}:{}/control/safesearch/{}",
            self.config.adguard_host, self.config.adguard_port, path
        );
        let mut request = self.client.post(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_safe_browsing(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!(
            "http://{}:{}/control/safebrowsing/{}",
            self.config.adguard_host, self.config.adguard_port, path
        );
        let mut request = self.client.post(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_parental_control(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!(
            "http://{}:{}/control/parental/{}",
            self.config.adguard_host, self.config.adguard_port, path
        );
        let mut request = self.client.post(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_filters(&self) -> Result<FilteringConfig> {
        let url = format!(
            "http://{}:{}/control/filtering/config",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<FilteringConfig>().await?;
        Ok(config)
    }

    pub async fn add_filter(&self, name: String, url: String, whitelist: bool) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/add_url",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&endpoint).json(&AddFilterRequest {
            name,
            url,
            whitelist,
        });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn toggle_filter(&self, url: String, name: String, enabled: bool) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/set_url",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&endpoint).json(&SetFilterUrlRequest {
            url: url.clone(),
            data: SetFilterUrlData { enabled, name, url },
        });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_filter(&self, url: String, whitelist: bool) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/remove_url",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&endpoint).json(&RemoveFilterRequest {
            url,
            whitelist,
        });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn update_filter(
        &self,
        current_url: String,
        new_url: String,
        name: String,
        whitelist: bool,
        enabled: bool,
    ) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/set_url",
            self.config.adguard_host, self.config.adguard_port
        );

        // Note: The AdGuard Home API uses the same endpoint for toggling (enable/disable) and editing (changing URL/Name).
        // However, the JSON structure might be slightly different or interpreted based on fields.
        // For editing, we usually need to provide the 'url' query param (identifying the old one) and the body with new data.
        // Let's verify if we need to implement a different logic or if the existing set_url is enough but with different payload.
        // Actually, looking at AdGuard Home API, /control/filtering/set_url is indeed used for editing.
        // The 'url' in the body is the NEW url, and the 'url' in the wrapper (if any) or query param identifies the filter.
        // Wait, the `SetFilterUrlRequest` struct I defined earlier has `url` and `data`.
        // The `url` field in `SetFilterUrlRequest` is what identifies the filter to change.
        // The `data` field contains the new properties.

        let mut request = self.client.post(&endpoint).json(&UpdateFilterRequest {
            url: current_url,
            name: name.clone(),
            whitelist,
            data: UpdateFilterData {
                name,
                url: new_url,
                enabled,
            },
        });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_clients(&self) -> Result<Vec<AdGuardClientDevice>> {
        let url = format!(
            "http://{}:{}/control/clients",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let clients_response = response.json::<ClientsResponse>().await?;
        Ok(clients_response.clients)
    }

    pub async fn get_client_info(&self, identifier: &str) -> Result<AdGuardClientDevice> {
        let clients = self.list_clients().await?;
        clients
            .into_iter()
            .find(|c| c.name == identifier || c.ids.iter().any(|id| id == identifier))
            .ok_or_else(|| {
                crate::error::Error::Generic(format!("Client not found: {}", identifier))
            })
    }

    pub async fn get_user_rules(&self) -> Result<Vec<String>> {
        let config = self.list_filters().await?;
        Ok(config.user_rules)
    }

    pub async fn set_user_rules(&self, rules: Vec<String>) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/filtering/set_rules",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&SetRulesRequest { rules });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_all_services(&self) -> Result<Vec<BlockedService>> {
        let url = format!(
            "http://{}:{}/control/blocked_services/all",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let all_response = response.json::<BlockedServicesAllResponse>().await?;
        Ok(all_response.services)
    }

    pub async fn list_blocked_services(&self) -> Result<Vec<String>> {
        let url = format!(
            "http://{}:{}/control/blocked_services/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let blocked_ids = response.json::<Vec<String>>().await?;
        Ok(blocked_ids)
    }

    pub async fn set_blocked_services(&self, ids: Vec<String>) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/blocked_services/set",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&ids);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn add_client(&self, client: AdGuardClientDevice) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/clients/add",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&client);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn update_client(&self, old_name: String, client: AdGuardClientDevice) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/clients/update",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&UpdateClientRequest {
            name: old_name,
            data: client,
        });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_client(&self, name: String) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/clients/delete",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&DeleteClientRequest { name });

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_dhcp_status(&self) -> Result<DhcpStatus> {
        let url = format!(
            "http://{}:{}/control/dhcp/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<DhcpStatus>().await?;
        Ok(status)
    }

    pub async fn add_static_lease(&self, lease: StaticLease) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/dhcp/add_static_lease",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&lease);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_static_lease(&self, lease: StaticLease) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/dhcp/remove_static_lease",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&lease);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_dns_info(&self) -> Result<DnsConfig> {
        let url = format!(
            "http://{}:{}/control/dns_info",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<DnsConfig>().await?;
        Ok(config)
    }

    pub async fn set_dns_config(&self, config: DnsConfig) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/dns_config",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&config);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_dns_cache(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/cache_clear",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_access_list(&self) -> Result<AccessList> {
        let url = format!(
            "http://{}:{}/control/access/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let list = response.json::<AccessList>().await?;
        Ok(list)
    }

    pub async fn set_access_list(&self, list: AccessList) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/access/set",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url).json(&list);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn check_host(
        &self,
        name: &str,
        client: Option<&str>,
    ) -> Result<FilterCheckResponse> {
        let mut url = format!(
            "http://{}:{}/control/filtering/check_host",
            self.config.adguard_host, self.config.adguard_port
        );
        url.push_str(&format!("?name={}", name));
        if let Some(c) = client {
            url.push_str(&format!("&client={}", c));
        }

        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let result = response.json::<FilterCheckResponse>().await?;
        Ok(result)
    }

    pub async fn reset_stats(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/stats_reset",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_query_log(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/querylog_clear",
            self.config.adguard_host, self.config.adguard_port
        );
        let mut request = self.client.post(&url);

        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }

        request.send().await?.error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(host: String, port: u16) -> AppConfig {
        AppConfig {
            adguard_host: host,
            adguard_port: port,
            adguard_username: None,
            adguard_password: None,
            mcp_transport: "stdio".to_string(),
            lazy_mode: false,
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
            no_verify_ssl: true,
        }
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
            .and(path("/control/filtering/config"))
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
            .and(path("/control/filtering/config"))
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
}
