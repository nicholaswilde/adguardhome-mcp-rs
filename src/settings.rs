use crate::error::{Error, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub adguard_url: String,
    pub adguard_username: Option<String>,
    pub adguard_password: Option<String>,
    pub lazy_mode: bool,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let adguard_url = env::var("ADGUARD_URL")
            .map_err(|_| Error::MissingEnvironmentVariable("ADGUARD_URL".to_string()))?;

        let adguard_username = env::var("ADGUARD_USERNAME").ok();
        let adguard_password = env::var("ADGUARD_PASSWORD").ok();
        let lazy_mode = env::var("LAZY_MODE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            adguard_url,
            adguard_username,
            adguard_password,
            lazy_mode,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_settings_from_env_missing_url() {
        let _guard = ENV_LOCK.lock().unwrap();
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
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("ADGUARD_URL", "http://localhost:8080");
            env::set_var("ADGUARD_USERNAME", "admin");
            env::set_var("ADGUARD_PASSWORD", "password");
            env::remove_var("LAZY_MODE");
        }

        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.adguard_url, "http://localhost:8080");
        assert_eq!(settings.adguard_username, Some("admin".to_string()));
        assert_eq!(settings.adguard_password, Some("password".to_string()));
        assert!(!settings.lazy_mode);
    }

    #[test]
    fn test_settings_lazy_mode() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("ADGUARD_URL", "http://localhost:8080");
            env::set_var("LAZY_MODE", "true");
        }
        let settings = Settings::from_env().unwrap();
        assert!(settings.lazy_mode);

        unsafe {
            env::set_var("LAZY_MODE", "false");
        }
        let settings = Settings::from_env().unwrap();
        assert!(!settings.lazy_mode);
    }
}
