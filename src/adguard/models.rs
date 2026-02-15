use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn deserialize_null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilteringConfig {
    pub enabled: bool,
    pub interval: u32,
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
    pub filters: Vec<Filter>,
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
    pub whitelist_filters: Vec<Filter>,
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
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
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafeSearchConfig {
    pub enabled: bool,
    pub bing: bool,
    pub duckduckgo: bool,
    pub google: bool,
    pub pixabay: bool,
    pub yandex: bool,
    pub youtube: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParentalControlConfig {
    pub enabled: bool,
    pub sensitivity: Option<u32>, // Optional, as it might not be present in all versions or configs
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryLogConfig {
    pub enabled: bool,
    pub interval: u32, // retention interval in hours
    pub anonymize_client_ip: bool,
    #[serde(default)]
    pub allowed_clients: Vec<String>,
    #[serde(default)]
    pub disallowed_clients: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub version: String,
    pub announcement: String,
    pub announcement_url: String,
    pub can_update: bool,
    pub new_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TlsConfig {
    pub enabled: bool,
    pub server_name: String,
    pub force_https: bool,
    pub port_https: u16,
    pub port_dns_over_tls: u16,
    pub port_dns_over_quic: u16,
    pub certificate_chain: String,
    pub private_key: String,
    pub certificate_path: String,
    pub private_key_path: String,
    #[serde(default)]
    pub valid_cert: bool,
    #[serde(default)]
    pub valid_key: bool,
    #[serde(default)]
    pub valid_pair: bool,
}
