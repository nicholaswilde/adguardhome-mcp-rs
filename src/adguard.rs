use crate::config::AppConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};

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

impl AdGuardClient {
    pub fn new(config: AppConfig) -> Self {
        let client = reqwest::Client::new();
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
}
