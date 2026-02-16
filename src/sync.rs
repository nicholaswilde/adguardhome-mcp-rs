use crate::adguard::models::{AdGuardClientDevice, DnsConfig, DnsRewrite, FilteringConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncState {
    pub filtering: FilteringConfig,
    pub clients: Vec<AdGuardClientDevice>,
    pub dns: DnsConfig,
    pub blocked_services: Vec<String>,
    pub rewrites: Vec<DnsRewrite>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_serialization() {
        let state = SyncState {
            filtering: FilteringConfig {
                enabled: true,
                interval: 1,
                filters: Vec::new(),
                whitelist_filters: Vec::new(),
                user_rules: vec!["rule1".to_string()],
            },
            clients: Vec::new(),
            dns: DnsConfig {
                upstream_dns: vec!["1.1.1.1".to_string()],
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
            blocked_services: vec!["youtube".to_string()],
            rewrites: vec![DnsRewrite {
                domain: "example.com".to_string(),
                answer: "1.2.3.4".to_string(),
            }],
        };

        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: SyncState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.filtering.user_rules[0], "rule1");
        assert_eq!(deserialized.dns.upstream_dns[0], "1.1.1.1");
        assert_eq!(deserialized.blocked_services[0], "youtube");
        assert_eq!(deserialized.rewrites[0].domain, "example.com");
    }
}
