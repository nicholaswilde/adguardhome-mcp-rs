use super::models::*;
use crate::config::AppConfig;
use crate::error::Result;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct AdGuardClient {
    pub client: reqwest::Client,
    pub config: AppConfig,
}

impl AdGuardClient {
    pub fn new(config: AppConfig) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_verify_ssl)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, config }
    }

    fn add_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let (Some(user), Some(pass)) =
            (&self.config.adguard_username, &self.config.adguard_password)
        {
            request = request.basic_auth(user, Some(pass));
        }
        request
    }

    pub async fn get_version_info(&self) -> Result<VersionInfo> {
        // Fallback to get_status as control/version_info is often 404 in newer versions
        match self.get_status().await {
            Ok(status) => Ok(VersionInfo {
                version: status.version,
                announcement: "AdGuard Home Status".to_string(),
                announcement_url: "".to_string(),
                can_update: false,
                new_version: "".to_string(),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn update_adguard_home(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/update",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_query_log_config(&self) -> Result<QueryLogConfig> {
        let url = format!(
            "http://{}:{}/control/querylog/config",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<QueryLogConfig>().await?;
        Ok(config)
    }

    pub async fn set_query_log_config(&self, config: QueryLogConfig) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/querylog/config/update",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.put(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_safe_search_settings(&self) -> Result<SafeSearchConfig> {
        let url = format!(
            "http://{}:{}/control/safesearch/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let settings = response.json::<SafeSearchConfig>().await?;
        Ok(settings)
    }

    pub async fn set_safe_search_settings(&self, settings: SafeSearchConfig) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/safesearch/settings",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.put(&url).json(&settings));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_parental_settings(&self) -> Result<ParentalControlConfig> {
        let url = format!(
            "http://{}:{}/control/parental/status",
            self.config.adguard_host, self.config.adguard_port
        );
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
        let url = format!(
            "http://{}:{}/control/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

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

        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let log = response.json::<QueryLogResponse>().await?;
        Ok(log)
    }

    pub async fn list_rewrites(&self) -> Result<Vec<DnsRewrite>> {
        let url = format!(
            "http://{}:{}/control/rewrite/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let rewrites = response.json::<Vec<DnsRewrite>>().await?;
        Ok(rewrites)
    }

    pub async fn add_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/rewrite/add",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&rewrite));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_rewrite(&self, rewrite: DnsRewrite) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/rewrite/delete",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&rewrite));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_protection(&self, enabled: bool) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/protection",
            self.config.adguard_host, self.config.adguard_port
        );
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
        let url = format!(
            "http://{}:{}/control/safesearch/{}",
            self.config.adguard_host, self.config.adguard_port, path
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_safe_browsing(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!(
            "http://{}:{}/control/safebrowsing/{}",
            self.config.adguard_host, self.config.adguard_port, path
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn set_parental_control(&self, enabled: bool) -> Result<()> {
        let path = if enabled { "enable" } else { "disable" };
        let url = format!(
            "http://{}:{}/control/parental/{}",
            self.config.adguard_host, self.config.adguard_port, path
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_filters(&self) -> Result<FilteringConfig> {
        let url = format!(
            "http://{}:{}/control/filtering/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<FilteringConfig>().await?;
        Ok(config)
    }

    pub async fn add_filter(&self, name: String, url: String, whitelist: bool) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/add_url",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&endpoint).json(&AddFilterRequest {
            name,
            url,
            whitelist,
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn toggle_filter(&self, url: String, name: String, enabled: bool) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/set_url",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&endpoint).json(&SetFilterUrlRequest {
            url: url.clone(),
            data: SetFilterUrlData { enabled, name, url },
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_filter(&self, url: String, whitelist: bool) -> Result<()> {
        let endpoint = format!(
            "http://{}:{}/control/filtering/remove_url",
            self.config.adguard_host, self.config.adguard_port
        );
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
        let endpoint = format!(
            "http://{}:{}/control/filtering/set_url",
            self.config.adguard_host, self.config.adguard_port
        );

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
        let url = format!(
            "http://{}:{}/control/clients",
            self.config.adguard_host, self.config.adguard_port
        );
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
        let url = format!(
            "http://{}:{}/control/filtering/set_rules",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&SetRulesRequest { rules }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn list_all_services(&self) -> Result<Vec<BlockedService>> {
        let url = format!(
            "http://{}:{}/control/blocked_services/all",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let all_response = response.json::<BlockedServicesAllResponse>().await?;
        Ok(all_response.services)
    }

    pub async fn list_blocked_services(&self) -> Result<Vec<String>> {
        let url = format!(
            "http://{}:{}/control/blocked_services/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let blocked_ids = response.json::<Vec<String>>().await?;
        Ok(blocked_ids)
    }

    pub async fn set_blocked_services(&self, ids: Vec<String>) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/blocked_services/set",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&ids));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn add_client(&self, client: AdGuardClientDevice) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/clients/add",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&client));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn update_client(&self, old_name: String, client: AdGuardClientDevice) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/clients/update",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&UpdateClientRequest {
            name: old_name,
            data: client,
        }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_client(&self, name: String) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/clients/delete",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&DeleteClientRequest { name }));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_dhcp_status(&self) -> Result<DhcpStatus> {
        let url = format!(
            "http://{}:{}/control/dhcp/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<DhcpStatus>().await?;
        Ok(status)
    }

    pub async fn add_static_lease(&self, lease: StaticLease) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/dhcp/add_static_lease",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&lease));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn remove_static_lease(&self, lease: StaticLease) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/dhcp/remove_static_lease",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&lease));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_dns_info(&self) -> Result<DnsConfig> {
        let url = format!(
            "http://{}:{}/control/dns_info",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<DnsConfig>().await?;
        Ok(config)
    }

    pub async fn set_dns_config(&self, config: DnsConfig) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/dns_config",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_dns_cache(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/cache_clear",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_access_list(&self) -> Result<AccessList> {
        let url = format!(
            "http://{}:{}/control/access/list",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let list = response.json::<AccessList>().await?;
        Ok(list)
    }

    pub async fn set_access_list(&self, list: AccessList) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/access/set",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&list));

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

        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let result = response.json::<FilterCheckResponse>().await?;
        Ok(result)
    }

    pub async fn reset_stats(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/stats_reset",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn clear_query_log(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/querylog_clear",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn create_backup(&self) -> Result<PathBuf> {
        let url = format!(
            "http://{}:{}/control/backup",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url));

        let response = request.send().await?.error_for_status()?;
        let bytes = response.bytes().await?;

        // Ensure backups directory exists
        let backup_dir = PathBuf::from("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).await?;
        }

        let file_name = format!("adguard_backup_{}.tar.gz", uuid::Uuid::new_v4());
        let file_path = backup_dir.join(file_name);

        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&bytes).await?;

        Ok(file_path)
    }

    pub async fn restore_backup(&self, file_path: &str) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/restore",
            self.config.adguard_host, self.config.adguard_port
        );

        let bytes = fs::read(file_path).await?;
        let part = reqwest::multipart::Part::bytes(bytes).file_name("backup.tar.gz");
        let form = reqwest::multipart::Form::new().part("file", part);

        let request = self.add_auth(self.client.post(&url).multipart(form));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn restart_service(&self) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/restart",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_tls_status(&self) -> Result<TlsConfig> {
        let url = format!(
            "http://{}:{}/control/tls/status",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.get(&url));

        let response = request.send().await?.error_for_status()?;
        let config = response.json::<TlsConfig>().await?;
        Ok(config)
    }

    pub async fn configure_tls(&self, config: TlsConfig) -> Result<()> {
        let url = format!(
            "http://{}:{}/control/tls/configure",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&config));

        request.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn validate_tls(&self, config: TlsConfig) -> Result<TlsConfig> {
        let url = format!(
            "http://{}:{}/control/tls/validate",
            self.config.adguard_host, self.config.adguard_port
        );
        let request = self.add_auth(self.client.post(&url).json(&config));

        let response = request.send().await?.error_for_status()?;
        let result = response.json::<TlsConfig>().await?;
        Ok(result)
    }
}
