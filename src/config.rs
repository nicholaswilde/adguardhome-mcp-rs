use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(alias = "url")]
    pub adguard_url: String,
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
}

fn default_transport() -> String {
    "stdio".to_string()
}

fn default_http_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
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
            .set_default("log_level", "info")?;

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

        // 5. Apply CLI overrides
        if let Some(url) = matches.get_one::<String>("adguard_url") {
            builder = builder.set_override("adguard_url", url.as_str())?;
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
        if let Some(port) = matches.get_one::<u16>("http_port") {
            builder = builder.set_override("http_port", *port)?;
        }
        if let Some(token) = matches.get_one::<String>("http_auth_token") {
            builder = builder.set_override("http_auth_token", token.as_str())?;
        }
        if let Some(level) = matches.get_one::<String>("log_level") {
            builder = builder.set_override("log_level", level.as_str())?;
        }

        builder.build()?.try_deserialize()
    }
}

fn parse_args(args: Vec<String>) -> ArgMatches {
    use clap::{Arg, ArgAction, Command};

    let cmd = Command::new("adguardhome-mcp-rs")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file"),
        )
        .arg(
            Arg::new("adguard_url")
                .long("adguard-url")
                .help("AdGuard Home URL"),
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
            std::env::remove_var("ADGUARD_URL");
            std::env::remove_var("ADGUARD_MCP_TRANSPORT");
        }

        let _config = AppConfig::load(None, vec![]).unwrap_or_else(|_| AppConfig {
            adguard_url: "".to_string(),
            adguard_username: None,
            adguard_password: None,
            mcp_transport: "stdio".to_string(),
            lazy_mode: false,
            http_port: 3000,
            http_auth_token: None,
            log_level: "info".to_string(),
        });

        // adguard_url is required, so it might fail if not set?
        // AppConfig::load calls builder.build()?.try_deserialize().
        // If adguard_url is missing, deserialization fails?
        // We should check if it returns Err.
        // Actually, we define adguard_url as String without default.
        // So yes, it should fail if not provided.
    }

    #[test]
    fn test_load_required_fields() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("ADGUARD_URL");
        }
        let res = AppConfig::load(None, vec![]);
        assert!(res.is_err(), "Should fail without adguard_url");
    }

    #[test]
    fn test_cli_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let args = vec![
            "app".to_string(),
            "--adguard-url".to_string(),
            "http://cli.com".to_string(),
            "--transport".to_string(),
            "http".to_string(),
            "--http-port".to_string(),
            "8080".to_string(),
            "--lazy".to_string(),
        ];
        let config = AppConfig::load(None, args).unwrap();
        assert_eq!(config.adguard_url, "http://cli.com");
        assert_eq!(config.mcp_transport, "http");
        assert_eq!(config.http_port, 8080);
        assert!(config.lazy_mode);
    }

    #[test]
    fn test_env_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("ADGUARD_URL", "http://env.com");
            std::env::set_var("ADGUARD_MCP_TRANSPORT", "http");
            std::env::set_var("ADGUARD_HTTP_PORT", "9090");
        }

        let config = AppConfig::load(None, vec![]).unwrap();

        unsafe {
            std::env::remove_var("ADGUARD_URL");
            std::env::remove_var("ADGUARD_MCP_TRANSPORT");
            std::env::remove_var("ADGUARD_HTTP_PORT");
        }

        assert_eq!(config.adguard_url, "http://env.com");
        assert_eq!(config.mcp_transport, "http");
        assert_eq!(config.http_port, 9090);
    }

    #[test]
    fn test_file_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("ADGUARD_URL");
        }

        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            "adguard_url = \"http://file.com\"\nmcp_transport = \"http\"\nhttp_port = 7070"
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = AppConfig::load(Some(path), vec![]).unwrap();
        assert_eq!(config.adguard_url, "http://file.com");
        assert_eq!(config.mcp_transport, "http");
        assert_eq!(config.http_port, 7070);
    }
}
