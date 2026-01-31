use crate::error::Result;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AdGuardClient {
    pub client: reqwest::Client,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub version: String,
    pub language: String,
    pub protection_enabled: bool,
}

impl AdGuardClient {
    pub fn new(settings: Settings) -> Self {
        let client = reqwest::Client::new();
        Self { client, settings }
    }

    pub async fn get_status(&self) -> Result<Status> {
        let url = format!(
            "{}/control/status",
            self.settings.adguard_url.trim_end_matches('/')
        );
        let mut request = self.client.get(&url);

        if let (Some(user), Some(pass)) = (
            &self.settings.adguard_username,
            &self.settings.adguard_password,
        ) {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request.send().await?.error_for_status()?;
        let status = response.json::<Status>().await?;
        Ok(status)
    }
}
