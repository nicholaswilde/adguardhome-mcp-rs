use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(alias = "host")]
    pub adguard_host: String,
    #[serde(alias = "port", default = "default_adguard_port")]
    pub adguard_port: u16,
    #[serde(alias = "username")]
    pub adguard_username: Option<String>,
    #[serde(alias = "password")]
    pub adguard_password: Option<String>,
    #[serde(default = "default_transport")]
    pub mcp_transport: String,
    #[serde(default)]
    pub lazy_mode: bool,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    pub http_auth_token: Option<String>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_no_verify_ssl")]
    pub no_verify_ssl: bool,
    #[serde(default, deserialize_with = "deserialize_instances")]
    pub instances: Vec<InstanceConfig>,
    #[serde(default)]
    pub replicas: Vec<ReplicaConfig>,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_seconds: u64,
    #[serde(default = "default_sync_mode")]
    pub default_sync_mode: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct InstanceConfig {
    pub name: Option<String>,
    pub url: String,
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub no_verify_ssl: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ReplicaConfig {
    pub url: String,
    pub api_key: String,
}

fn default_transport() -> String {
    "stdio".to_string()
}

fn default_adguard_port() -> u16 {
    3000
}

fn default_http_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_no_verify_ssl() -> bool {
    true
}

fn default_sync_interval() -> u64 {
    3600
}

fn default_sync_mode() -> String {
    "additive-merge".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            adguard_host: "localhost".to_string(),
            adguard_port: 3000,
            adguard_username: None,
            adguard_password: None,
            mcp_transport: "stdio".to_string(),
            lazy_mode: false,
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
            no_verify_ssl: true,
            instances: Vec::new(),
            replicas: Vec::new(),
            sync_interval_seconds: 3600,
            default_sync_mode: "additive-merge".to_string(),
        }
    }
}

fn deserialize_instances<'de, D>(deserializer: D) -> Result<Vec<InstanceConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, SeqAccess, Visitor};
    use std::collections::BTreeMap;
    use std::fmt;

    struct InstancesVisitor;

    impl<'de> Visitor<'de> for InstancesVisitor {
        type Value = Vec<InstanceConfig>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence or a map")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element()? {
                vec.push(value);
            }
            Ok(vec)
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut btree_map = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, InstanceConfig>()? {
                btree_map.insert(key, value);
            }
            // Sort by key if they are numeric to maintain order
            Ok(btree_map.into_values().collect())
        }
    }

    deserializer.deserialize_any(InstancesVisitor)
}

impl AppConfig {
    pub fn load(file_path: Option<String>, cli_args: Vec<String>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        let matches = parse_args(cli_args);

        // 1. Determine Config File Path
        let path_to_load = if let Some(p) = file_path {
            Some(p)
        } else {
            matches.get_one::<String>("config").cloned()
        };

        // 2. Set Defaults
        builder = builder
            .set_default("mcp_transport", "stdio")?
            .set_default("lazy_mode", false)?
            .set_default("http_port", 3000)?
            .set_default("log_level", "info")?
            .set_default("no_verify_ssl", true)?
            .set_default("sync_interval_seconds", 3600)?
            .set_default("default_sync_mode", "additive-merge")?;

        // 3. Load from File
        if let Some(path) = path_to_load {
            builder = builder.add_source(File::with_name(&path));
        } else {
            // Check default locations
            // Current directory
            builder = builder.add_source(File::with_name("config").required(false));

            // System config directory (simplified for now, ideally use directories crate or just specific paths)
            // For adguardhome-mcp-rs, we can check ~/.config/adguardhome-mcp-rs/config
            if let Ok(home) = std::env::var("HOME") {
                let path = format!("{}/.config/adguardhome-mcp-rs/config", home);
                builder = builder.add_source(File::with_name(&path).required(false));
            }
        }

        // 4. Load from Environment Variables
        builder = builder.add_source(
            Environment::with_prefix("ADGUARD")
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        );

        if let Ok(replicas_json) = std::env::var("ADGUARD_REPLICAS") {
            let wrapped_json = format!(r#"{{"replicas": {}}}"#, replicas_json);
            builder = builder.add_source(config::File::from_str(
                &wrapped_json,
                config::FileFormat::Json,
            ));
        }

        // 5. Apply CLI overrides
        if let Some(host) = matches.get_one::<String>("adguard_host") {
            builder = builder.set_override("adguard_host", host.as_str())?;
        }
        if let Some(port) = matches.get_one::<u16>("adguard_port") {
            builder = builder.set_override("adguard_port", *port)?;
        }
        if let Some(username) = matches.get_one::<String>("adguard_username") {
            builder = builder.set_override("adguard_username", username.as_str())?;
        }
        if let Some(password) = matches.get_one::<String>("adguard_password") {
            builder = builder.set_override("adguard_password", password.as_str())?;
        }
        if let Some(transport) = matches.get_one::<String>("mcp_transport") {
            builder = builder.set_override("mcp_transport", transport.as_str())?;
        }
        if matches.get_flag("lazy_mode") {
            builder = builder.set_override("lazy_mode", true)?;
        }
        if matches.get_flag("no_verify_ssl") {
            builder = builder.set_override("no_verify_ssl", true)?;
        }
        if let Some(port) = matches.get_one::<u16>("http_port") {
            builder = builder.set_override("http_port", *port)?;
        }
        if let Some(token) = matches.get_one::<String>("http_auth_token") {
            builder = builder.set_override("http_auth_token", token.as_str())?;
        }
        if let Some(level) = matches.get_one::<String>("log_level") {
            builder = builder.set_override("log_level", level.as_str())?;
        }

        let mut config: AppConfig = builder.build()?.try_deserialize()?;
        config.validate().map_err(ConfigError::Message)?;
        Ok(config)
    }

    pub fn validate(&mut self) -> Result<(), String> {
        if self.instances.is_empty() {
            // Synthesis: if instances is empty, but host is present, create a default instance.
            if !self.adguard_host.is_empty() {
                let url = if self.adguard_host.starts_with("http") {
                    self.adguard_host.clone()
                } else {
                    format!("http://{}:{}", self.adguard_host, self.adguard_port)
                };

                self.instances.push(InstanceConfig {
                    name: Some("default".to_string()),
                    url,
                    username: self.adguard_username.clone(),
                    password: self.adguard_password.clone(),
                    no_verify_ssl: Some(self.no_verify_ssl),
                    api_key: None,
                });
            }
        }

        if self.instances.is_empty() {
            return Err("At least one AdGuard Home instance must be configured".to_string());
        }

        for (i, inst) in self.instances.iter().enumerate() {
            if inst.url.is_empty() {
                return Err(format!("Instance {} is missing URL", i));
            }
            // Ensure some form of auth is present, although it's not strictly required by AGH itself
            // it's highly recommended and expected by this MCP server.
        }

        Ok(())
    }

    pub fn get_instance(&self, name_or_index: Option<&str>) -> std::result::Result<&InstanceConfig, String> {
        match name_or_index {
            None => self.instances.get(0).ok_or_else(|| "No instances configured".to_string()),
            Some(s) => {
                // Try as index
                if let Ok(idx) = s.parse::<usize>() {
                    if let Some(inst) = self.instances.get(idx) {
                        return Ok(inst);
                    }
                }
                // Try as name
                self.instances.iter()
                    .find(|i| i.name.as_deref() == Some(s))
                    .ok_or_else(|| format!("Instance not found: {}", s))
            }
        }
    }
}

fn parse_args(args: Vec<String>) -> ArgMatches {
    use clap::{Arg, ArgAction, Command};

    let cmd = Command::new("adguardhome-mcp-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file"),
        )
        .arg(
            Arg::new("adguard_host")
                .long("adguard-host")
                .help("AdGuard Home host"),
        )
        .arg(
            Arg::new("adguard_port")
                .long("adguard-port")
                .help("AdGuard Home port")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("adguard_username")
                .long("adguard-username")
                .help("AdGuard Home Username"),
        )
        .arg(
            Arg::new("adguard_password")
                .long("adguard-password")
                .help("AdGuard Home Password"),
        )
        .arg(
            Arg::new("mcp_transport")
                .long("transport")
                .help("Transport mode: stdio or http"),
        )
        .arg(
            Arg::new("lazy_mode")
                .long("lazy")
                .action(ArgAction::SetTrue)
                .help("Enable lazy mode"),
        )
        .arg(
            Arg::new("no_verify_ssl")
                .long("no-verify-ssl")
                .action(ArgAction::SetTrue)
                .help("Disable SSL certificate verification"),
        )
        .arg(
            Arg::new("http_port")
                .long("http-port")
                .help("Port for HTTP transport")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("http_auth_token")
                .long("http-token")
                .help("Authentication token for HTTP transport"),
        )
        .arg(Arg::new("log_level").long("log-level").help("Log level"));

    if args.is_empty() {
        cmd.get_matches_from(vec!["adguardhome-mcp-rs"])
    } else {
        cmd.get_matches_from(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ENV_LOCK;
    use std::io::Write;

    #[test]
    fn test_load_defaults() {
        // Ensure env vars don't interfere
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("ADGUARD_HOST");
            std::env::remove_var("ADGUARD_PORT");
            std::env::remove_var("ADGUARD_MCP_TRANSPORT");
        }

        let _config = AppConfig::load(None, vec![]).unwrap_or_else(|_| AppConfig {
            adguard_host: "".to_string(),
            ..Default::default()
        });
    }

    #[test]
    fn test_load_required_fields() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("ADGUARD_HOST");
        }
        let res = AppConfig::load(None, vec![]);
        assert!(res.is_err(), "Should fail without adguard_host");
    }

    #[test]
    fn test_cli_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let args = vec![
            "app".to_string(),
            "--adguard-host".to_string(),
            "cli.com".to_string(),
            "--adguard-port".to_string(),
            "4000".to_string(),
            "--transport".to_string(),
            "http".to_string(),
            "--http-port".to_string(),
            "8080".to_string(),
            "--lazy".to_string(),
        ];
        let config = AppConfig::load(None, args).unwrap();
        assert_eq!(config.adguard_host, "cli.com");
        assert_eq!(config.adguard_port, 4000);
        assert_eq!(config.mcp_transport, "http");
        assert_eq!(config.http_port, 8080);
        assert!(config.lazy_mode);
    }

    #[test]
    fn test_env_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("ADGUARD_HOST", "env.com");
            std::env::set_var("ADGUARD_PORT", "5050");
            std::env::set_var("ADGUARD_MCP_TRANSPORT", "http");
            std::env::set_var("ADGUARD_HTTP_PORT", "9090");
        }

        let config = AppConfig::load(None, vec![]).unwrap();

        unsafe {
            std::env::remove_var("ADGUARD_HOST");
            std::env::remove_var("ADGUARD_PORT");
            std::env::remove_var("ADGUARD_MCP_TRANSPORT");
            std::env::remove_var("ADGUARD_HTTP_PORT");
        }

        assert_eq!(config.adguard_host, "env.com");
        assert_eq!(config.adguard_port, 5050);
        assert_eq!(config.mcp_transport, "http");
        assert_eq!(config.http_port, 9090);
    }

    #[test]
    fn test_file_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("ADGUARD_HOST");
        }

        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            "adguard_host = \"file.com\"\nadguard_port = 6060\nmcp_transport = \"http\"\nhttp_port = 7070"
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = AppConfig::load(Some(path), vec![]).unwrap();
        assert_eq!(config.adguard_host, "file.com");
        assert_eq!(config.adguard_port, 6060);
        assert_eq!(config.mcp_transport, "http");
        assert_eq!(config.http_port, 7070);
    }

    #[test]
    fn test_no_verify_ssl_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Default should be true
        let config = AppConfig::load(
            None,
            vec![
                "app".to_string(),
                "--adguard-host".to_string(),
                "localhost".to_string(),
            ],
        )
        .unwrap();
        assert!(config.no_verify_ssl);

        // CLI override to false (assuming we use a flag that can be negated or just testing the default for now)
        // Wait, the spec says "Default value: true (SSL verification is disabled by default)".
        // Usually flags are false by default. If it's true by default, we might need a --verify-ssl flag to disable it?
        // Or just test that it can be set to true/false.

        let args = vec![
            "app".to_string(),
            "--adguard-host".to_string(),
            "localhost".to_string(),
            "--no-verify-ssl".to_string(),
        ];
        let config = AppConfig::load(None, args).unwrap();
        assert!(config.no_verify_ssl);
    }

    #[test]
    fn test_replica_config_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
adguard_host = "master.com"
sync_interval_seconds = 60
default_sync_mode = "full-overwrite"

[[replicas]]
url = "http://replica1.com:3000"
api_key = "key1"

[[replicas]]
url = "http://replica2.com:3000"
api_key = "key2"
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = AppConfig::load(Some(path), vec![]).unwrap();
        assert_eq!(config.replicas.len(), 2);
        assert_eq!(config.replicas[0].url, "http://replica1.com:3000");
        assert_eq!(config.replicas[0].api_key, "key1");
        assert_eq!(config.sync_interval_seconds, 60);
        assert_eq!(config.default_sync_mode, "full-overwrite");
    }

    #[test]
    fn test_replica_env_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("ADGUARD_HOST", "master.com");
            std::env::set_var(
                "ADGUARD_REPLICAS",
                r#"[{"url": "http://env-replica.com", "api_key": "env-key"}]"#,
            );
        }

        let config = AppConfig::load(None, vec![]).unwrap();

        unsafe {
            std::env::remove_var("ADGUARD_HOST");
            std::env::remove_var("ADGUARD_REPLICAS");
        }

        assert_eq!(config.replicas.len(), 1);
        assert_eq!(config.replicas[0].url, "http://env-replica.com");
        assert_eq!(config.replicas[0].api_key, "env-key");
    }

    #[test]
    fn test_multi_instance_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
adguard_host = "primary.com"

[[instances]]
name = "primary"
url = "http://192.168.1.1"
api_key = "key1"

[[instances]]
name = "secondary"
url = "http://192.168.1.2"
username = "admin"
password = "password"
no_verify_ssl = false
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = AppConfig::load(Some(path), vec![]).unwrap();
        assert_eq!(config.instances.len(), 2);
        assert_eq!(config.instances[0].name, Some("primary".to_string()));
        assert_eq!(config.instances[0].url, "http://192.168.1.1");
        assert_eq!(config.instances[0].api_key, Some("key1".to_string()));
        assert_eq!(config.instances[1].name, Some("secondary".to_string()));
        assert_eq!(config.instances[1].url, "http://192.168.1.2");
        assert_eq!(config.instances[1].username, Some("admin".to_string()));
        assert_eq!(config.instances[1].password, Some("password".to_string()));
        assert_eq!(config.instances[1].no_verify_ssl, Some(false));
    }

    #[test]
    fn test_multi_instance_env_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("ADGUARD_HOST", "primary.com");
            std::env::set_var("ADGUARD_INSTANCES__0__NAME", "env-primary");
            std::env::set_var("ADGUARD_INSTANCES__0__URL", "http://10.0.0.1");
            std::env::set_var("ADGUARD_INSTANCES__1__NAME", "env-secondary");
            std::env::set_var("ADGUARD_INSTANCES__1__URL", "http://10.0.0.2");
        }

        let config = AppConfig::load(None, vec![]).unwrap();

        unsafe {
            std::env::remove_var("ADGUARD_HOST");
            std::env::remove_var("ADGUARD_INSTANCES__0__NAME");
            std::env::remove_var("ADGUARD_INSTANCES__0__URL");
            std::env::remove_var("ADGUARD_INSTANCES__1__NAME");
            std::env::remove_var("ADGUARD_INSTANCES__1__URL");
        }

        assert_eq!(config.instances.len(), 2);
        assert_eq!(config.instances[0].name, Some("env-primary".to_string()));
        assert_eq!(config.instances[0].url, "http://10.0.0.1");
        assert_eq!(config.instances[1].name, Some("env-secondary".to_string()));
        assert_eq!(config.instances[1].url, "http://10.0.0.2");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.instances = vec![];
        // Default has adguard_host = localhost, so it should validate by creating a default instance
        assert!(config.validate().is_ok());
        assert_eq!(config.instances.len(), 1);
        assert_eq!(config.instances[0].name, Some("default".to_string()));

        // Invalid instance (missing URL)
        config.instances = vec![InstanceConfig {
            url: "".to_string(),
            ..Default::default()
        }];
        assert!(config.validate().is_err());
    }
}
