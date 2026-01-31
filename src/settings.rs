use crate::error::{Error, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub adguard_url: String,
    pub adguard_username: Option<String>,
    pub adguard_password: Option<String>,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let adguard_url = env::var("ADGUARD_URL")
            .map_err(|_| Error::MissingEnvironmentVariable("ADGUARD_URL".to_string()))?;

        let adguard_username = env::var("ADGUARD_USERNAME").ok();
        let adguard_password = env::var("ADGUARD_PASSWORD").ok();

        Ok(Self {
            adguard_url,
            adguard_username,
            adguard_password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_from_env_missing_url() {
        unsafe {
            env::remove_var("ADGUARD_URL");
        }
        let result = Settings::from_env();
        assert!(
            matches!(result, Err(Error::MissingEnvironmentVariable(ref v)) if v == "ADGUARD_URL")
        );
    }

    #[test]
    fn test_settings_from_env_success() {
        unsafe {
            env::set_var("ADGUARD_URL", "http://localhost:8080");
            env::set_var("ADGUARD_USERNAME", "admin");
            env::set_var("ADGUARD_PASSWORD", "password");
        }

        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.adguard_url, "http://localhost:8080");
        assert_eq!(settings.adguard_username, Some("admin".to_string()));
        assert_eq!(settings.adguard_password, Some("password".to_string()));
    }
}
