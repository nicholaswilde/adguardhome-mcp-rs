use super::models::*;
use crate::config::InstanceConfig;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct AdGuardClient {
    pub client: reqwest::Client,
    pub config: InstanceConfig,
}

impl AdGuardClient {
    pub fn new(config: InstanceConfig) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl.unwrap_or(true))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, config }
    }

    fn add_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = &self.config.api_key {
            request = request.header("X-API-Key", api_key);
        } else if let (Some(user), Some(pass)) = (&self.config.username, &self.config.password) {
            request = request.basic_auth(user, Some(pass));
        }
        request
    }

    pub async fn get_version_info(&self) -> Result<VersionInfo> {
        let url = format!("{}/control/version_info", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        match request.send().await {
            Ok(resp) if resp.status().is_success() => Ok(resp.json::<VersionInfo>().await?),
            _ => {
                // Fallback to get_status as control/version_info is often 404 in newer versions
                let status = self.get_status().await?;
                Ok(VersionInfo {
                    version: status.version,
                    announcement: "AdGuard Home Status".to_string(),
                    announcement_url: "".to_string(),
                    can_update: false,
                    new_version: "".to_string(),
                })
            }
        }
    }

    pub async fn update_adguard_home(&self) -> Result<()> {
        let info = self.get_version_info().await?;
        if !info.can_update {
            return Err(crate::error::Error::Generic(
                "/update request isn't allowed now: no update available".to_string(),
            ));
        }

        let url = format!("{}/control/update", self.config.url);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_query_log_config(&self) -> Result<QueryLogConfig> {
        let url = format!("{}/control/querylog/config", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<QueryLogConfig>().await?;
        Ok(config)
    }

    pub async fn set_query_log_config(&self, config: QueryLogConfig) -> Result<()> {
        let url = format!("{}/control/querylog/config/update", self.config.url);
        let request = self.add_auth(self.client.put(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_safe_search_settings(&self) -> Result<SafeSearchConfig> {
        let url = format!("{}/control/safesearch/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let settings = response.json::<SafeSearchConfig>().await?;
        Ok(settings)
    }

    pub async fn set_safe_search_settings(&self, settings: SafeSearchConfig) -> Result<()> {
        let url = format!("{}/control/safesearch/settings", self.config.url);
        let request = self.add_auth(self.client.put(&url).json(&settings));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_parental_settings(&self) -> Result<ParentalControlConfig> {
        let url = format!("{}/control/parental/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let settings = response.json::<ParentalControlConfig>().await?;
        Ok(settings)
    }

    pub async fn set_parental_settings(&self, settings: ParentalControlConfig) -> Result<()> {
        if settings.enabled {
            self.set_parental_control(true).await?;
        } else {
            self.set_parental_control(false).await?;
        }
        Ok(())
    }

    pub async fn get_status(&self) -> Result<Status> {
        let url = format!("{}/control/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<Status>().await?;
        Ok(status)
    }

    pub async fn get_stats(&self, time_period: Option<&str>) -> Result<Stats> {
        let mut url = format!("{}/control/stats", self.config.url);
        if let Some(period) = time_period {
            url.push_str(&format!("?time_period={}", period));
        }
        let request = self.add_auth(self.client.get(&url));

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
        let mut url = format!("{}/control/querylog", self.config.url);
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

        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let log = response.json::<QueryLogResponse>().await?;
        Ok(log)
    }

    pub async fn list_rewrites(&self) -> Result<Vec<DnsRewrite>> {
        let url = format!("{}/control/rewrite/list", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let rewrites = response.json::<Vec<DnsRewrite>>().await?;
        Ok(rewrites)
    }

    pub async fn add_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!("{}/control/rewrite/add", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&rewrite));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!("{}/control/rewrite/delete", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&rewrite));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_protection(&self, enabled: bool) -> Result<()> {
        let url = format!("{}/control/protection", self.config.url);
        let request = self.add_auth(
            self.client
                .post(&url)
                .json(&serde_json::json!({ "enabled": enabled })),
        );

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_safe_search(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!("{}/control/safesearch/{}", self.config.url, path);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_safe_browsing(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!("{}/control/safebrowsing/{}", self.config.url, path);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_parental_control(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!("{}/control/parental/{}", self.config.url, path);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_filters(&self) -> Result<FilteringConfig> {
        let url = format!("{}/control/filtering/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<FilteringConfig>().await?;
        Ok(config)
    }

    pub async fn add_filter(&self, name: String, url: String, whitelist: bool) -> Result<()> {
        let endpoint = format!("{}/control/filtering/add_url", self.config.url);
        let request = self.add_auth(self.client.post(&endpoint).json(&AddFilterRequest {
            name,
            url,
            whitelist,
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn toggle_filter(&self, url: String, name: String, enabled: bool) -> Result<()> {
        let endpoint = format!("{}/control/filtering/set_url", self.config.url);
        let request = self.add_auth(self.client.post(&endpoint).json(&SetFilterUrlRequest {
            url: url.clone(),
            data: SetFilterUrlData { enabled, name, url },
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_filter(&self, url: String, whitelist: bool) -> Result<()> {
        let endpoint = format!("{}/control/filtering/remove_url", self.config.url);
        let request = self.add_auth(
            self.client
                .post(&endpoint)
                .json(&RemoveFilterRequest { url, whitelist }),
        );

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
        let endpoint = format!("{}/control/filtering/set_url", self.config.url);

        let request = self.add_auth(self.client.post(&endpoint).json(&UpdateFilterRequest {
            url: current_url,
            name: name.clone(),
            whitelist,
            data: UpdateFilterData {
                name,
                url: new_url,
                enabled,
            },
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_clients(&self) -> Result<Vec<AdGuardClientDevice>> {
        let url = format!("{}/control/clients", self.config.url);
        let request = self.add_auth(self.client.get(&url));

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
        let url = format!("{}/control/filtering/set_rules", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&SetRulesRequest { rules }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_all_services(&self) -> Result<Vec<BlockedService>> {
        let url = format!("{}/control/blocked_services/all", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let all_response = response.json::<BlockedServicesAllResponse>().await?;
        Ok(all_response.services)
    }

    pub async fn list_blocked_services(&self) -> Result<Vec<String>> {
        let url = format!("{}/control/blocked_services/list", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let blocked_ids = response.json::<Vec<String>>().await?;
        Ok(blocked_ids)
    }

    pub async fn set_blocked_services(&self, ids: Vec<String>) -> Result<()> {
        let url = format!("{}/control/blocked_services/set", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&ids));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn add_client(&self, client: AdGuardClientDevice) -> Result<()> {
        let url = format!("{}/control/clients/add", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&client));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn update_client(&self, old_name: String, client: AdGuardClientDevice) -> Result<()> {
        let url = format!("{}/control/clients/update", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&UpdateClientRequest {
            name: old_name,
            data: client,
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_client(&self, name: String) -> Result<()> {
        let url = format!("{}/control/clients/delete", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&DeleteClientRequest { name }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_dhcp_status(&self) -> Result<DhcpStatus> {
        let url = format!("{}/control/dhcp/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<DhcpStatus>().await?;
        Ok(status)
    }

    pub async fn set_dhcp_config(&self, config: DhcpStatus) -> Result<()> {
        let url = format!("{}/control/dhcp/set_config", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_profile_info(&self) -> Result<ProfileInfo> {
        let url = format!("{}/control/profile", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let profile = response.json::<ProfileInfo>().await?;
        Ok(profile)
    }

    pub async fn set_profile_info(&self, profile: ProfileInfo) -> Result<()> {
        let url = format!("{}/control/profile/update", self.config.url);
        let request = self.add_auth(self.client.put(&url).json(&profile));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn add_static_lease(&self, lease: StaticLease) -> Result<()> {
        let url = format!("{}/control/dhcp/add_static_lease", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&lease));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_static_lease(&self, lease: StaticLease) -> Result<()> {
        let url = format!("{}/control/dhcp/remove_static_lease", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&lease));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_dns_info(&self) -> Result<DnsConfig> {
        let url = format!("{}/control/dns_info", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<DnsConfig>().await?;
        Ok(config)
    }

    pub async fn set_dns_config(&self, config: DnsConfig) -> Result<()> {
        let url = format!("{}/control/dns_config", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_dns_cache(&self) -> Result<()> {
        let url = format!("{}/control/cache_clear", self.config.url);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_access_list(&self) -> Result<AccessList> {
        let url = format!("{}/control/access/list", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let list = response.json::<AccessList>().await?;
        Ok(list)
    }

    pub async fn set_access_list(&self, list: AccessList) -> Result<()> {
        let url = format!("{}/control/access/set", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&list));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn check_host(
        &self,
        name: &str,
        client: Option<&str>,
    ) -> Result<FilterCheckResponse> {
        let mut url = format!("{}/control/filtering/check_host", self.config.url);
        url.push_str(&format!("?name={}", name));
        if let Some(c) = client {
            url.push_str(&format!("&client={}", c));
        }

        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let result = response.json::<FilterCheckResponse>().await?;
        Ok(result)
    }

    pub async fn reset_stats(&self) -> Result<()> {
        let url = format!("{}/control/stats_reset", self.config.url);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_query_log(&self) -> Result<()> {
        let url = format!("{}/control/querylog_clear", self.config.url);
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn restart_service(&self, hard: bool) -> Result<()> {
        if hard {
            let url = format!("{}/control/restart", self.config.url);
            let request = self.add_auth(self.client.post(&url));
            // We ignore error_for_status here because /restart often closes the connection
            // before returning a response, causing a "connection closed" error.
            let _ = request.send().await;
        } else {
            // Soft restart (refresh filters)
            let url = format!("{}/control/filtering/refresh", self.config.url);
            let request = self.add_auth(
                self.client
                    .post(&url)
                    .json(&serde_json::json!({ "whitelist": false })),
            );
            request.send().await?.error_for_status()?;
        }
        Ok(())
    }

    pub async fn get_tls_status(&self) -> Result<TlsConfig> {
        let url = format!("{}/control/tls/status", self.config.url);
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<TlsConfig>().await?;
        Ok(config)
    }

    pub async fn configure_tls(&self, config: TlsConfig) -> Result<()> {
        let url = format!("{}/control/tls/configure", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn validate_tls(&self, config: TlsConfig) -> Result<TlsConfig> {
        let url = format!("{}/control/tls/validate", self.config.url);
        let request = self.add_auth(self.client.post(&url).json(&config));

        let response = request.send().await?.error_for_status()?;
        let result = response.json::<TlsConfig>().await?;
        Ok(result)
    }
}
